use crate::device;
use crate::device::BlockDevice as DeviceBlockDevice;
use crate::fs::block_device::BlockDevice;
use alloc::sync::Arc;
use alloc::string::String;
use alloc::string::ToString;
use spin::Mutex;
use crate::errors::KernelError;

/// Adapter to use a device::BlockDevice as a fs::BlockDevice
pub struct DeviceBlockAdapter {
    device: Arc<Mutex<dyn device::Device>>,
    name: String,
}

impl DeviceBlockAdapter {
    /// Create a new adapter for a device
    pub fn new(device: Arc<Mutex<dyn device::Device>>) -> Self {
        let name = {
            let device_guard = device.lock();
            device_guard.name().to_string()
        };
        
        Self { 
            device,
            name
        }
    }
    
    /// Create a new adapter for the first available block device
    pub fn new_first_available() -> Result<Self, KernelError> {
        // Get all block devices
        let block_devices = device::get_block_devices();
        
        if block_devices.is_empty() {
            return Err(KernelError::DeviceNotFound);
        }
        
        // Use the first device
        let device = block_devices[0].clone();
        
        Ok(Self::new(device))
    }
    
    /// Get the name of the underlying device
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl BlockDevice for DeviceBlockAdapter {
    fn block_size(&self) -> usize {
        let device_guard = self.device.lock();
        
        // Try to downcast to BlockDevice
        if let Some(block_device) = device_guard.as_any().downcast_ref::<device::ata::AtaDevice>() {
            // Need to use the trait method through DeviceBlockDevice trait
            return block_device.block_size();
        }
        
        // Default size if not a known block device
        512
    }
    
    fn block_count(&self) -> u64 {
        let device_guard = self.device.lock();
        
        // Try to downcast to BlockDevice
        if let Some(block_device) = device_guard.as_any().downcast_ref::<device::ata::AtaDevice>() {
            return block_device.block_count() as u64;
        }
        
        // Default size if not a known block device
        0
    }
    
    fn read_block(&self, block_id: u64, buffer: &mut [u8]) -> Result<(), &'static str> {
        let device_guard = self.device.lock();
        
        // Try to downcast to BlockDevice
        if let Some(block_device) = device_guard.as_any().downcast_ref::<device::ata::AtaDevice>() {
            return block_device.read_block(block_id as usize, buffer)
                .map_err(|e| e.to_str());
        }
        
        Err("Device is not a block device")
    }
    
    fn write_block(&mut self, block_id: u64, buffer: &[u8]) -> Result<(), &'static str> {
        let mut device_guard = self.device.lock();
        
        // Try to downcast to BlockDevice
        if let Some(block_device) = device_guard.as_any_mut().downcast_mut::<device::ata::AtaDevice>() {
            return block_device.write_block(block_id as usize, buffer)
                .map_err(|e| e.to_str());
        }
        
        Err("Device is not a block device")
    }
}

impl crate::fs::block_device::BlockDeviceMarker for DeviceBlockAdapter {} 