use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use alloc::string::ToString;
use crate::errors::{KernelError, DeviceError};
use crate::device::{Device, DeviceType, DeviceStatus};
use core::sync::atomic::{AtomicU64, Ordering};
use x86_64::instructions::port::{Port, PortWriteOnly, PortReadOnly};

/// PIO-based ATA driver for IDE disks
///
/// This is a simplistic driver that uses PIO mode for ATA/IDE disks.
/// It only supports the primary channel and does not use interrupts.
pub struct AtaDevice {
    id: u64,
    name: String,
    status: DeviceStatus,
    
    // Port I/O addresses for ATA controller (primary bus, master drive)
    data_port: Port<u16>,
    error_port: PortReadOnly<u8>,
    sector_count_port: Port<u8>,
    lba_low_port: Port<u8>,
    lba_mid_port: Port<u8>,
    lba_high_port: Port<u8>,
    device_port: Port<u8>,
    command_port: Port<u8>,
    control_port: PortWriteOnly<u8>,
    
    // Disk information
    sector_size: usize,
    sector_count: u64,
    
    // Flags for driver state
    initialized: bool,
}

// ATA controller port addresses
const ATA_PRIMARY_DATA: u16 = 0x1F0;
const ATA_PRIMARY_ERROR: u16 = 0x1F1;
const ATA_PRIMARY_SECTOR_COUNT: u16 = 0x1F2;
const ATA_PRIMARY_LBA_LOW: u16 = 0x1F3;
const ATA_PRIMARY_LBA_MID: u16 = 0x1F4;
const ATA_PRIMARY_LBA_HIGH: u16 = 0x1F5;
const ATA_PRIMARY_DEVICE: u16 = 0x1F6;
const ATA_PRIMARY_COMMAND: u16 = 0x1F7;
const ATA_PRIMARY_CONTROL: u16 = 0x3F6;

// ATA commands
const ATA_CMD_READ_SECTORS: u8 = 0x20;
const ATA_CMD_WRITE_SECTORS: u8 = 0x30;
const ATA_CMD_IDENTIFY: u8 = 0xEC;

// ATA status register bits
const ATA_SR_BSY: u8 = 0x80; // Busy
const ATA_SR_DRDY: u8 = 0x40; // Drive ready
const ATA_SR_DF: u8 = 0x20; // Drive fault
const ATA_SR_DSC: u8 = 0x10; // Drive seek complete
const ATA_SR_DRQ: u8 = 0x08; // Data request ready
const ATA_SR_CORR: u8 = 0x04; // Corrected data
const ATA_SR_IDX: u8 = 0x02; // Index
const ATA_SR_ERR: u8 = 0x01; // Error

// ATA error register bits
const ATA_ER_BBK: u8 = 0x80; // Bad block
const ATA_ER_UNC: u8 = 0x40; // Uncorrectable data
const ATA_ER_MC: u8 = 0x20; // Media changed
const ATA_ER_IDNF: u8 = 0x10; // ID not found
const ATA_ER_MCR: u8 = 0x08; // Media change request
const ATA_ER_ABRT: u8 = 0x04; // Command aborted
const ATA_ER_TK0NF: u8 = 0x02; // Track 0 not found
const ATA_ER_AMNF: u8 = 0x01; // No address mark

// Default sector size
const DEFAULT_SECTOR_SIZE: usize = 512;

impl AtaDevice {
    /// Create a new ATA device for the primary channel, master drive
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        
        AtaDevice {
            id,
            name: "ata0-master".to_string(),
            status: DeviceStatus::Uninitialized,
            
            // Create port objects for the ATA controller
            data_port: Port::new(ATA_PRIMARY_DATA),
            error_port: PortReadOnly::new(ATA_PRIMARY_ERROR),
            sector_count_port: Port::new(ATA_PRIMARY_SECTOR_COUNT),
            lba_low_port: Port::new(ATA_PRIMARY_LBA_LOW),
            lba_mid_port: Port::new(ATA_PRIMARY_LBA_MID),
            lba_high_port: Port::new(ATA_PRIMARY_LBA_HIGH),
            device_port: Port::new(ATA_PRIMARY_DEVICE),
            command_port: Port::new(ATA_PRIMARY_COMMAND),
            control_port: PortWriteOnly::new(ATA_PRIMARY_CONTROL),
            
            sector_size: DEFAULT_SECTOR_SIZE,
            sector_count: 0,
            initialized: false,
        }
    }
    
    /// Wait until the disk is not busy
    fn wait_not_busy(&mut self) -> Result<(), KernelError> {
        // Read the status port up to 10000 times before giving up
        for _ in 0..10000 {
            let status = unsafe { self.command_port.read() };
            if status & ATA_SR_BSY == 0 {
                return Ok(());
            }
        }
        
        Err(KernelError::DeviceTimeout)
    }
    
    /// Wait until the disk is ready to transfer data
    fn wait_drq(&mut self) -> Result<(), KernelError> {
        // Read the status port up to 10000 times before giving up
        for _ in 0..10000 {
            let status = unsafe { self.command_port.read() };
            if status & ATA_SR_DRQ != 0 {
                return Ok(());
            }
            if status & ATA_SR_ERR != 0 || status & ATA_SR_DF != 0 {
                return Err(KernelError::DeviceError(DeviceError::InitFailed));
            }
        }
        
        Err(KernelError::DeviceTimeout)
    }
    
    /// Select the drive (master)
    fn select_drive(&mut self) {
        unsafe {
            // 0xE0 selects the master drive with LBA addressing
            self.device_port.write(0xE0);
        }
    }
    
    /// Read the identify data from the disk
    fn identify(&mut self) -> Result<Vec<u16>, KernelError> {
        self.select_drive();
        
        // Reset sector counts
        unsafe {
            self.sector_count_port.write(0);
            self.lba_low_port.write(0);
            self.lba_mid_port.write(0);
            self.lba_high_port.write(0);
        }
        
        // Send the IDENTIFY command
        unsafe {
            self.command_port.write(ATA_CMD_IDENTIFY);
        }
        
        // Read the status port to see if the disk is present
        let status = unsafe { self.command_port.read() };
        if status == 0 {
            return Err(KernelError::DeviceNotFound);
        }
        
        // Wait for the BSY flag to clear
        self.wait_not_busy()?;
        
        // Check if disk is ATA
        let lba_mid = unsafe { self.lba_mid_port.read() };
        let lba_high = unsafe { self.lba_high_port.read() };
        if lba_mid != 0 || lba_high != 0 {
            return Err(KernelError::UnsupportedFeature);
        }
        
        // Wait for the DRQ flag to set or error to be indicated
        self.wait_drq()?;
        
        // Read the identification data (256 words)
        let mut data = Vec::with_capacity(256);
        for _ in 0..256 {
            let value = unsafe { self.data_port.read() };
            data.push(value);
        }
        
        Ok(data)
    }
    
    /// Process identify data to extract disk information
    fn process_identify_data(&mut self, data: &[u16]) -> Result<(), KernelError> {
        if data.len() < 256 {
            return Err(KernelError::InvalidData);
        }
        
        // Extract sector count from LBA48 or LBA28 fields
        let lba28_sectors = ((data[60] as u32) | ((data[61] as u32) << 16)) as u64;
        let lba48_sectors = if data[83] & (1 << 10) != 0 {
            // LBA48 supported, use the 48-bit sector count
            (data[100] as u64) | 
             ((data[101] as u64) << 16) |
             ((data[102] as u64) << 32) |
             ((data[103] as u64) << 48)
        } else {
            0
        };
        
        // Use the larger of the two
        self.sector_count = if lba48_sectors > lba28_sectors {
            lba48_sectors
        } else {
            lba28_sectors
        };
        
        // Extract the model string (words 27-46)
        let mut model = String::new();
        for i in 27..47 {
            let word = data[i];
            model.push((((word >> 8) & 0xFF) as u8) as char);
            model.push(((word & 0xFF) as u8) as char);
        }
        
        // Trim whitespace and update name
        let model = model.trim();
        self.name = format!("ata0-master ({})", model);
        
        // Successfully processed identify data
        Ok(())
    }
    
    /// Read sectors from the disk using LBA28 addressing
    pub fn read_sectors(&mut self, lba: u32, count: u8, buffer: &mut [u8]) -> Result<(), KernelError> {
        if !self.initialized {
            return Err(KernelError::DeviceNotInitialized);
        }
        
        // Check if LBA is within range
        if (lba as u64) + (count as u64) > self.sector_count {
            return Err(KernelError::InvalidParameter);
        }
        
        // Check if buffer is large enough
        if buffer.len() < (count as usize) * self.sector_size {
            return Err(KernelError::BufferTooSmall);
        }
        
        self.select_drive();
        
        // Send the sector count and LBA
        unsafe {
            self.sector_count_port.write(count);
            self.lba_low_port.write((lba & 0xFF) as u8);
            self.lba_mid_port.write(((lba >> 8) & 0xFF) as u8);
            self.lba_high_port.write(((lba >> 16) & 0xFF) as u8);
            
            // Upper 4 bits of LBA go in device register (bits 0-3)
            let device_bits = 0xE0 | (((lba >> 24) & 0x0F) as u8);
            self.device_port.write(device_bits);
            
            // Send the READ SECTORS command
            self.command_port.write(ATA_CMD_READ_SECTORS);
        }
        
        // Read the requested sectors
        for sector in 0..count {
            // Wait for the disk to be ready
            self.wait_not_busy()?;
            self.wait_drq()?;
            
            // Read one sector of data (256 words = 512 bytes)
            let start = (sector as usize) * self.sector_size;
            let end = start + self.sector_size;
            
            // Read 16-bit words into the buffer
            for i in (start..end).step_by(2) {
                let value = unsafe { self.data_port.read() };
                buffer[i] = (value & 0xFF) as u8;
                buffer[i + 1] = ((value >> 8) & 0xFF) as u8;
            }
        }
        
        Ok(())
    }
    
    /// Write sectors to the disk using LBA28 addressing
    pub fn write_sectors(&mut self, lba: u32, count: u8, buffer: &[u8]) -> Result<(), KernelError> {
        if !self.initialized {
            return Err(KernelError::DeviceNotInitialized);
        }
        
        // Check if LBA is within range
        if (lba as u64) + (count as u64) > self.sector_count {
            return Err(KernelError::InvalidParameter);
        }
        
        // Check if buffer is large enough
        if buffer.len() < (count as usize) * self.sector_size {
            return Err(KernelError::BufferTooSmall);
        }
        
        self.select_drive();
        
        // Send the sector count and LBA
        unsafe {
            self.sector_count_port.write(count);
            self.lba_low_port.write((lba & 0xFF) as u8);
            self.lba_mid_port.write(((lba >> 8) & 0xFF) as u8);
            self.lba_high_port.write(((lba >> 16) & 0xFF) as u8);
            
            // Upper 4 bits of LBA go in device register (bits 0-3)
            let device_bits = 0xE0 | (((lba >> 24) & 0x0F) as u8);
            self.device_port.write(device_bits);
            
            // Send the WRITE SECTORS command
            self.command_port.write(ATA_CMD_WRITE_SECTORS);
        }
        
        // Write the requested sectors
        for sector in 0..count {
            // Wait for the disk to be ready
            self.wait_not_busy()?;
            self.wait_drq()?;
            
            // Write one sector of data (256 words = 512 bytes)
            let start = (sector as usize) * self.sector_size;
            let end = start + self.sector_size;
            
            // Write 16-bit words from the buffer
            for i in (start..end).step_by(2) {
                let value = (buffer[i] as u16) | ((buffer[i + 1] as u16) << 8);
                unsafe {
                    self.data_port.write(value);
                }
            }
        }
        
        // Flush cache (could add a flush cache command here)
        
        Ok(())
    }
    
    /// Check if a drive is present
    pub fn is_present(&mut self) -> bool {
        self.select_drive();
        
        // Reset sector counts
        unsafe {
            self.sector_count_port.write(0);
            self.lba_low_port.write(0);
            self.lba_mid_port.write(0);
            self.lba_high_port.write(0);
        }
        
        // Read status
        let status = unsafe { self.command_port.read() };
        status != 0 && status != 0xff
    }
}

impl Device for AtaDevice {
    fn id(&self) -> u64 {
        self.id
    }
    
    fn device_type(&self) -> DeviceType {
        DeviceType::Block
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn status(&self) -> DeviceStatus {
        self.status
    }
    
    fn initialize(&mut self) -> Result<(), KernelError> {
        // Check if the device is present
        if !self.is_present() {
            self.status = DeviceStatus::NotResponding;
            return Err(KernelError::DeviceNotFound);
        }
        
        // Identify the device
        match self.identify() {
            Ok(data) => {
                self.process_identify_data(&data)?;
                self.initialized = true;
                self.status = DeviceStatus::Initialized;
                Ok(())
            }
            Err(e) => {
                self.status = DeviceStatus::Error;
                Err(e)
            }
        }
    }
    
    fn reset(&mut self) -> Result<(), KernelError> {
        // Write to the control register (bit 2 = soft reset)
        unsafe {
            self.control_port.write(0x04);
        }
        
        // Wait a bit
        for _ in 0..1000 {
            // Delay
        }
        
        // Clear the reset bit
        unsafe {
            self.control_port.write(0x00);
        }
        
        // Wait for the drive to be ready
        self.wait_not_busy()?;
        
        self.status = DeviceStatus::Initialized;
        Ok(())
    }
    
    fn suspend(&mut self) -> Result<(), KernelError> {
        // Nothing to do for now
        self.status = DeviceStatus::Suspended;
        Ok(())
    }
    
    fn resume(&mut self) -> Result<(), KernelError> {
        self.status = DeviceStatus::Initialized;
        Ok(())
    }
    
    fn debug_info(&self) -> String {
        format!(
            "ATA Drive: {} (ID: {})\n\
             Status: {:?}\n\
             Sector Size: {} bytes\n\
             Sector Count: {}\n\
             Capacity: {} MB",
            self.name, self.id, self.status, self.sector_size, self.sector_count,
            (self.sector_count * self.sector_size as u64) / (1024 * 1024)
        )
    }
    
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn core::any::Any {
        self
    }
}

// Implement BlockDevice trait from the device module
impl crate::device::BlockDevice for AtaDevice {
    fn block_size(&self) -> usize {
        self.sector_size
    }
    
    fn block_count(&self) -> usize {
        self.sector_count as usize
    }
    
    fn read_block(&self, block_id: usize, buffer: &mut [u8]) -> Result<(), KernelError> {
        if !self.initialized {
            return Err(KernelError::DeviceNotInitialized);
        }
        
        // This is a bit of a hack since read_sectors requires a mutable reference
        // In a real implementation, we'd use interior mutability or another approach
        let mut mutable_self = AtaDevice {
            id: self.id,
            name: self.name.clone(),
            status: self.status,
            data_port: Port::new(ATA_PRIMARY_DATA),
            error_port: PortReadOnly::new(ATA_PRIMARY_ERROR),
            sector_count_port: Port::new(ATA_PRIMARY_SECTOR_COUNT),
            lba_low_port: Port::new(ATA_PRIMARY_LBA_LOW),
            lba_mid_port: Port::new(ATA_PRIMARY_LBA_MID),
            lba_high_port: Port::new(ATA_PRIMARY_LBA_HIGH),
            device_port: Port::new(ATA_PRIMARY_DEVICE),
            command_port: Port::new(ATA_PRIMARY_COMMAND),
            control_port: PortWriteOnly::new(ATA_PRIMARY_CONTROL),
            sector_size: self.sector_size,
            sector_count: self.sector_count,
            initialized: self.initialized,
        };
        
        mutable_self.read_sectors(block_id as u32, 1, buffer)
    }
    
    fn write_block(&mut self, block_id: usize, buffer: &[u8]) -> Result<(), KernelError> {
        self.write_sectors(block_id as u32, 1, buffer)
    }
    
    fn flush(&mut self) -> Result<(), KernelError> {
        // ATA flush cache command could be implemented here
        Ok(())
    }
}

// Make block device methods public on AtaDevice
impl AtaDevice {
    /// Get the block size in bytes
    pub fn block_size(&self) -> usize {
        self.sector_size
    }
    
    /// Get the total number of blocks
    pub fn block_count(&self) -> usize {
        self.sector_count as usize
    }
} 