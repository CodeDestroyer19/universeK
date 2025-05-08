//! PCI bus driver
//! Provides PCI device enumeration and configuration

use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::format;
use spin::Mutex;
use lazy_static::lazy_static;
use x86_64::instructions::port::{Port, PortWriteOnly};
use crate::errors::KernelError;
use crate::serial_println;

// PCI configuration space ports
const PCI_CONFIG_ADDR: u16 = 0xCF8;
const PCI_CONFIG_DATA: u16 = 0xCFC;

// PCI registers
const PCI_VENDOR_ID: u8 = 0x00;
const PCI_DEVICE_ID: u8 = 0x02;
const PCI_COMMAND: u8 = 0x04;
const PCI_STATUS: u8 = 0x06;
const PCI_REVISION_ID: u8 = 0x08;
const PCI_PROG_IF: u8 = 0x09;
const PCI_SUBCLASS: u8 = 0x0A;
const PCI_CLASS_CODE: u8 = 0x0B;
const PCI_CACHE_LINE_SIZE: u8 = 0x0C;
const PCI_LATENCY_TIMER: u8 = 0x0D;
const PCI_HEADER_TYPE: u8 = 0x0E;
const PCI_BIST: u8 = 0x0F;
const PCI_BAR0: u8 = 0x10;
const PCI_BAR1: u8 = 0x14;
const PCI_BAR2: u8 = 0x18;
const PCI_BAR3: u8 = 0x1C;
const PCI_BAR4: u8 = 0x20;
const PCI_BAR5: u8 = 0x24;
const PCI_INTERRUPT_LINE: u8 = 0x3C;
const PCI_INTERRUPT_PIN: u8 = 0x3D;

// PCI device information
#[derive(Debug, Clone)]
pub struct PciDeviceInfo {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub revision: u8,
    pub header_type: u8,
    pub interrupt_line: u8,
    pub interrupt_pin: u8,
    pub bar: [u32; 6],
}

impl PciDeviceInfo {
    pub fn new(bus: u8, device: u8, function: u8) -> Self {
        Self {
            bus,
            device,
            function,
            vendor_id: 0,
            device_id: 0,
            class_code: 0,
            subclass: 0,
            prog_if: 0,
            revision: 0,
            header_type: 0,
            interrupt_line: 0,
            interrupt_pin: 0,
            bar: [0; 6],
        }
    }
    
    pub fn device_type(&self) -> String {
        match (self.class_code, self.subclass) {
            (0x00, 0x00) => "Non-VGA-Compatible Unclassified Device".to_string(),
            (0x00, 0x01) => "VGA-Compatible Unclassified Device".to_string(),
            (0x01, 0x00) => "SCSI Bus Controller".to_string(),
            (0x01, 0x01) => "IDE Controller".to_string(),
            (0x01, 0x02) => "Floppy Disk Controller".to_string(),
            (0x01, 0x03) => "IPI Bus Controller".to_string(),
            (0x01, 0x04) => "RAID Controller".to_string(),
            (0x01, 0x05) => "ATA Controller".to_string(),
            (0x01, 0x06) => "SATA Controller".to_string(),
            (0x01, 0x07) => "Serial Attached SCSI Controller".to_string(),
            (0x01, 0x08) => "Non-Volatile Memory Controller".to_string(),
            (0x01, _) => format!("Mass Storage Controller (subclass = 0x{:02x})", self.subclass),
            (0x02, 0x00) => "Ethernet Controller".to_string(),
            (0x02, 0x01) => "Token Ring Controller".to_string(),
            (0x02, 0x02) => "FDDI Controller".to_string(),
            (0x02, 0x03) => "ATM Controller".to_string(),
            (0x02, 0x04) => "ISDN Controller".to_string(),
            (0x02, 0x05) => "WorldFip Controller".to_string(),
            (0x02, 0x06) => "PICMG Controller".to_string(),
            (0x02, 0x07) => "Infiniband Controller".to_string(),
            (0x02, 0x08) => "Fabric Controller".to_string(),
            (0x02, _) => format!("Network Controller (subclass = 0x{:02x})", self.subclass),
            (0x03, 0x00) => "VGA Compatible Controller".to_string(),
            (0x03, 0x01) => "XGA Controller".to_string(),
            (0x03, 0x02) => "3D Controller".to_string(),
            (0x03, _) => format!("Display Controller (subclass = 0x{:02x})", self.subclass),
            (0x04, 0x00) => "Multimedia Video Controller".to_string(),
            (0x04, 0x01) => "Multimedia Audio Controller".to_string(),
            (0x04, 0x02) => "Computer Telephony Device".to_string(),
            (0x04, 0x03) => "Audio Device".to_string(),
            (0x04, _) => format!("Multimedia Controller (subclass = 0x{:02x})", self.subclass),
            (0x05, 0x00) => "RAM Controller".to_string(),
            (0x05, 0x01) => "Flash Controller".to_string(),
            (0x05, _) => format!("Memory Controller (subclass = 0x{:02x})", self.subclass),
            (0x06, 0x00) => "Host Bridge".to_string(),
            (0x06, 0x01) => "ISA Bridge".to_string(),
            (0x06, 0x02) => "EISA Bridge".to_string(),
            (0x06, 0x03) => "MCA Bridge".to_string(),
            (0x06, 0x04) => "PCI-to-PCI Bridge".to_string(),
            (0x06, 0x05) => "PCMCIA Bridge".to_string(),
            (0x06, 0x06) => "NuBus Bridge".to_string(),
            (0x06, 0x07) => "CardBus Bridge".to_string(),
            (0x06, 0x08) => "RACEway Bridge".to_string(),
            (0x06, 0x09) => "PCI-to-PCI Bridge".to_string(),
            (0x06, 0x0A) => "InfiniBand-to-PCI Host Bridge".to_string(),
            (0x06, _) => format!("Bridge Device (subclass = 0x{:02x})", self.subclass),
            (0x07, 0x00) => "Serial Controller".to_string(),
            (0x07, 0x01) => "Parallel Controller".to_string(),
            (0x07, 0x02) => "Multiport Serial Controller".to_string(),
            (0x07, 0x03) => "Modem".to_string(),
            (0x07, 0x04) => "GPIB Controller".to_string(),
            (0x07, 0x05) => "Smart Card Controller".to_string(),
            (0x07, _) => format!("Simple Communication Controller (subclass = 0x{:02x})", self.subclass),
            (0x08, 0x00) => "PIC".to_string(),
            (0x08, 0x01) => "DMA Controller".to_string(),
            (0x08, 0x02) => "Timer".to_string(),
            (0x08, 0x03) => "RTC Controller".to_string(),
            (0x08, 0x04) => "PCI Hot-Plug Controller".to_string(),
            (0x08, 0x05) => "SD Host Controller".to_string(),
            (0x08, 0x06) => "IOMMU".to_string(),
            (0x08, _) => format!("Base System Peripheral (subclass = 0x{:02x})", self.subclass),
            (0x09, 0x00) => "Keyboard Controller".to_string(),
            (0x09, 0x01) => "Digitizer Pen".to_string(),
            (0x09, 0x02) => "Mouse Controller".to_string(),
            (0x09, 0x03) => "Scanner Controller".to_string(),
            (0x09, 0x04) => "Gameport Controller".to_string(),
            (0x09, _) => format!("Input Device Controller (subclass = 0x{:02x})", self.subclass),
            (0x0A, 0x00) => "Generic Docking Station".to_string(),
            (0x0A, _) => format!("Docking Station (subclass = 0x{:02x})", self.subclass),
            (0x0B, 0x00) => "386 Processor".to_string(),
            (0x0B, 0x01) => "486 Processor".to_string(),
            (0x0B, 0x02) => "Pentium Processor".to_string(),
            (0x0B, 0x03) => "Pentium Pro Processor".to_string(),
            (0x0B, 0x10) => "Alpha Processor".to_string(),
            (0x0B, 0x20) => "PowerPC Processor".to_string(),
            (0x0B, 0x30) => "MIPS Processor".to_string(),
            (0x0B, 0x40) => "Co-Processor".to_string(),
            (0x0B, _) => format!("Processor (subclass = 0x{:02x})", self.subclass),
            (0x0C, 0x00) => "FireWire (IEEE 1394) Controller".to_string(),
            (0x0C, 0x01) => "ACCESS Bus Controller".to_string(),
            (0x0C, 0x02) => "SSA Controller".to_string(),
            (0x0C, 0x03) => "USB Controller".to_string(),
            (0x0C, 0x04) => "Fibre Channel Controller".to_string(),
            (0x0C, 0x05) => "SMBus Controller".to_string(),
            (0x0C, 0x06) => "InfiniBand Controller".to_string(),
            (0x0C, 0x07) => "IPMI Interface".to_string(),
            (0x0C, 0x08) => "SERCOS Interface".to_string(),
            (0x0C, 0x09) => "CANBUS Controller".to_string(),
            (0x0C, _) => format!("Serial Bus Controller (subclass = 0x{:02x})", self.subclass),
            (0x0D, 0x00) => "IRDA Controller".to_string(),
            (0x0D, 0x01) => "Consumer IR Controller".to_string(),
            (0x0D, 0x10) => "RF Controller".to_string(),
            (0x0D, 0x11) => "Bluetooth Controller".to_string(),
            (0x0D, 0x12) => "Broadband Controller".to_string(),
            (0x0D, 0x20) => "Ethernet Controller (802.1a)".to_string(),
            (0x0D, 0x21) => "Ethernet Controller (802.1b)".to_string(),
            (0x0D, _) => format!("Wireless Controller (subclass = 0x{:02x})", self.subclass),
            (0x0E, 0x00) => "I2O Controller".to_string(),
            (0x0F, 0x01) => "Satellite TV Controller".to_string(),
            (0x0F, 0x02) => "Satellite Audio Controller".to_string(),
            (0x0F, 0x03) => "Satellite Voice Controller".to_string(),
            (0x0F, 0x04) => "Satellite Data Controller".to_string(),
            (0x10, 0x00) => "Network and Computing Encryption/Decryption".to_string(),
            (0x10, 0x10) => "Entertainment Encryption/Decryption".to_string(),
            (0x11, 0x00) => "DPIO Controller".to_string(),
            (0x11, 0x01) => "Performance Counters".to_string(),
            (0x11, 0x10) => "Communications Synchronization Controller".to_string(),
            (0x11, 0x20) => "Management Card".to_string(),
            (0x12, 0x00) => "Data Acquisition Controller".to_string(),
            (0x13, _) => "Reserved".to_string(),
            (_, _) => format!("Unknown Device (class = 0x{:02x}, subclass = 0x{:02x})", 
                              self.class_code, self.subclass),
        }
    }
    
    pub fn description(&self) -> String {
        format!("PCI {}.{}.{}: {}: {:04x}:{:04x} ({})",
            self.bus, self.device, self.function,
            self.device_type(),
            self.vendor_id, self.device_id,
            match self.vendor_id {
                0x1234 => "QEMU",
                0x8086 => "Intel",
                0x1022 => "AMD",
                0x10DE => "NVIDIA",
                0x1AF4 => "Virtio",
                0x1B36 => "QEMU Virtual",
                _ => "Unknown Vendor"
            }
        )
    }
}

// PCI driver structure
struct PciDriver {
    config_addr: PortWriteOnly<u32>,
    config_data: Port<u32>,
    devices: Vec<PciDeviceInfo>,
}

impl PciDriver {
    fn new() -> Self {
        Self {
            config_addr: PortWriteOnly::new(PCI_CONFIG_ADDR),
            config_data: Port::new(PCI_CONFIG_DATA),
            devices: Vec::new(),
        }
    }
    
    fn init(&mut self) -> Result<(), KernelError> {
        // Scan for PCI devices
        self.scan_bus();
        Ok(())
    }
    
    fn scan_bus(&mut self) {
        serial_println!("Scanning PCI bus for devices...");
        
        // Scan all buses, devices, and functions
        for bus in 0..256 {
            for device in 0..32 {
                for function in 0..8 {
                    if let Some(dev_info) = self.probe_device(bus as u8, device as u8, function as u8) {
                        serial_println!("Found PCI device: {}", dev_info.description());
                        self.devices.push(dev_info);
                    }
                }
            }
        }
        
        serial_println!("PCI scan complete. Found {} devices.", self.devices.len());
    }
    
    fn probe_device(&mut self, bus: u8, device: u8, function: u8) -> Option<PciDeviceInfo> {
        // Read vendor ID
        let vendor_id = self.read_config_u16(bus, device, function, PCI_VENDOR_ID);
        
        // Check if the device exists (vendor ID != 0xFFFF)
        if vendor_id == 0xFFFF {
            return None;
        }
        
        // Read device information
        let mut dev_info = PciDeviceInfo::new(bus, device, function);
        dev_info.vendor_id = vendor_id;
        dev_info.device_id = self.read_config_u16(bus, device, function, PCI_DEVICE_ID);
        dev_info.class_code = self.read_config_u8(bus, device, function, PCI_CLASS_CODE);
        dev_info.subclass = self.read_config_u8(bus, device, function, PCI_SUBCLASS);
        dev_info.prog_if = self.read_config_u8(bus, device, function, PCI_PROG_IF);
        dev_info.revision = self.read_config_u8(bus, device, function, PCI_REVISION_ID);
        dev_info.header_type = self.read_config_u8(bus, device, function, PCI_HEADER_TYPE);
        dev_info.interrupt_line = self.read_config_u8(bus, device, function, PCI_INTERRUPT_LINE);
        dev_info.interrupt_pin = self.read_config_u8(bus, device, function, PCI_INTERRUPT_PIN);
        
        // Read BARs
        dev_info.bar[0] = self.read_config_u32(bus, device, function, PCI_BAR0);
        dev_info.bar[1] = self.read_config_u32(bus, device, function, PCI_BAR1);
        dev_info.bar[2] = self.read_config_u32(bus, device, function, PCI_BAR2);
        dev_info.bar[3] = self.read_config_u32(bus, device, function, PCI_BAR3);
        dev_info.bar[4] = self.read_config_u32(bus, device, function, PCI_BAR4);
        dev_info.bar[5] = self.read_config_u32(bus, device, function, PCI_BAR5);
        
        Some(dev_info)
    }
    
    fn read_config_u8(&mut self, bus: u8, device: u8, function: u8, offset: u8) -> u8 {
        let address = self.get_address(bus, device, function, offset);
        unsafe {
            self.config_addr.write(address);
            (self.config_data.read() >> ((offset & 3) * 8)) as u8
        }
    }
    
    fn read_config_u16(&mut self, bus: u8, device: u8, function: u8, offset: u8) -> u16 {
        let address = self.get_address(bus, device, function, offset);
        unsafe {
            self.config_addr.write(address);
            (self.config_data.read() >> ((offset & 2) * 8)) as u16
        }
    }
    
    fn read_config_u32(&mut self, bus: u8, device: u8, function: u8, offset: u8) -> u32 {
        let address = self.get_address(bus, device, function, offset);
        unsafe {
            self.config_addr.write(address);
            self.config_data.read()
        }
    }
    
    fn write_config_u32(&mut self, bus: u8, device: u8, function: u8, offset: u8, value: u32) {
        let address = self.get_address(bus, device, function, offset);
        unsafe {
            self.config_addr.write(address);
            self.config_data.write(value);
        }
    }
    
    fn get_address(&self, bus: u8, device: u8, function: u8, offset: u8) -> u32 {
        // Format:
        // Bit 31    : Enable bit (1)
        // Bit 30-24 : Reserved (0)
        // Bit 23-16 : Bus number
        // Bit 15-11 : Device number
        // Bit 10-8  : Function number
        // Bit 7-2   : Register number
        // Bit 1-0   : Always 0
        0x80000000 | 
        ((bus as u32) << 16) |
        ((device as u32) << 11) |
        ((function as u32) << 8) |
        ((offset as u32) & 0xFC)
    }
    
    /// Find a PCI device by its class and subclass
    fn find_device_by_class(&self, class_code: u8, subclass: u8) -> Option<&PciDeviceInfo> {
        self.devices.iter().find(|dev| 
            dev.class_code == class_code && dev.subclass == subclass
        )
    }
    
    /// Find a PCI device by vendor ID and device ID
    fn find_device_by_id(&self, vendor_id: u16, device_id: u16) -> Option<&PciDeviceInfo> {
        self.devices.iter().find(|dev| 
            dev.vendor_id == vendor_id && dev.device_id == device_id
        )
    }
}

lazy_static! {
    static ref PCI: Mutex<PciDriver> = Mutex::new(PciDriver::new());
}

/// Initialize the PCI driver
pub fn init() -> Result<(), KernelError> {
    serial_println!("Initializing PCI driver");
    
    // Initialize the PCI driver
    PCI.lock().init()?;
    
    Ok(())
}

/// Find a PCI device by class and subclass
pub fn find_device_by_class(class_code: u8, subclass: u8) -> Option<PciDeviceInfo> {
    PCI.lock().find_device_by_class(class_code, subclass).cloned()
}

/// Find a PCI device by vendor ID and device ID
pub fn find_device_by_id(vendor_id: u16, device_id: u16) -> Option<PciDeviceInfo> {
    PCI.lock().find_device_by_id(vendor_id, device_id).cloned()
}

/// Get all discovered PCI devices
pub fn get_devices() -> Vec<PciDeviceInfo> {
    PCI.lock().devices.clone()
}

/// Check if a specific device type exists
pub fn has_device_type(class_code: u8, subclass: u8) -> bool {
    PCI.lock().find_device_by_class(class_code, subclass).is_some()
}

/// Check for common device types
pub fn has_network_card() -> bool {
    has_device_type(0x02, 0x00) // Ethernet controller
}

pub fn has_usb_controller() -> bool {
    has_device_type(0x0C, 0x03) // USB controller
}

pub fn has_sata_controller() -> bool {
    has_device_type(0x01, 0x06) // SATA controller
}

pub fn has_vga_controller() -> bool {
    has_device_type(0x03, 0x00) // VGA controller
} 