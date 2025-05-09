// kernel/src/device/mod.rs
//! Device driver framework and device management
//! This module defines the interface for all hardware device drivers.

pub mod ata; // ATA/IDE disk driver

use core::any::Any;
use core::fmt;
use core::sync::atomic::{AtomicU64, Ordering};
use alloc::string::String;
use alloc::vec::Vec;
use alloc::sync::Arc;
use spin::Mutex;
use crate::errors::KernelError;
use crate::serial_println;

/// Unique ID generator for devices
static NEXT_DEVICE_ID: AtomicU64 = AtomicU64::new(1);

/// Generates a unique device ID
fn generate_device_id() -> u64 {
    NEXT_DEVICE_ID.fetch_add(1, Ordering::SeqCst)
}

/// DeviceType categorizes different classes of devices
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Block,
    Character,
    Network,
    Input,
    Timer,
    Interrupt,
    Other,
}

/// DeviceStatus represents the current operational state of a device
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceStatus {
    Uninitialized,
    Initialized,
    Running,
    Suspended,
    Error,
    NotResponding,
}

/// The core trait that all device drivers must implement
pub trait Device: Send + Sync + Any {
    /// Returns the device's unique ID
    fn id(&self) -> u64;
    
    /// Returns the device type
    fn device_type(&self) -> DeviceType;
    
    /// Returns a human-readable name for the device
    fn name(&self) -> &str;
    
    /// Returns the current status of the device
    fn status(&self) -> DeviceStatus;
    
    /// Initializes the device. This must be called before the device can be used.
    fn initialize(&mut self) -> Result<(), KernelError>;
    
    /// Resets the device to a known good state
    fn reset(&mut self) -> Result<(), KernelError>;
    
    /// Suspends device operation (if supported)
    fn suspend(&mut self) -> Result<(), KernelError>;
    
    /// Resumes device operation after suspension
    fn resume(&mut self) -> Result<(), KernelError>;
    
    /// Provides debug info for the device
    fn debug_info(&self) -> String;
    
    /// Type-specific device operations (to be used with downcast)
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl fmt::Debug for dyn Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Device {{ id: {}, name: \"{}\", type: {:?}, status: {:?} }}",
              self.id(), self.name(), self.device_type(), self.status())
    }
}

/// BlockDevice extends the Device trait for block-oriented storage devices
pub trait BlockDevice: Device {
    /// Returns the size of each block in bytes
    fn block_size(&self) -> usize;
    
    /// Returns the total number of blocks
    fn block_count(&self) -> usize;
    
    /// Returns the total capacity in bytes
    fn capacity(&self) -> u64 {
        (self.block_size() as u64) * (self.block_count() as u64)
    }
    
    /// Reads a block into the provided buffer
    fn read_block(&self, block_id: usize, buffer: &mut [u8]) -> Result<(), KernelError>;
    
    /// Writes a block from the provided buffer
    fn write_block(&mut self, block_id: usize, buffer: &[u8]) -> Result<(), KernelError>;
    
    /// Flushes any cached data to the underlying device
    fn flush(&mut self) -> Result<(), KernelError>;
}

/// CharacterDevice extends the Device trait for byte-stream oriented devices
pub trait CharacterDevice: Device {
    /// Reads a single byte from the device, blocking if necessary
    fn read_byte(&self) -> Result<u8, KernelError>;
    
    /// Writes a single byte to the device
    fn write_byte(&mut self, byte: u8) -> Result<(), KernelError>;
    
    /// Reads multiple bytes into a buffer, returns number of bytes read
    fn read(&self, buffer: &mut [u8]) -> Result<usize, KernelError>;
    
    /// Writes multiple bytes from a buffer, returns number of bytes written
    fn write(&mut self, buffer: &[u8]) -> Result<usize, KernelError>;
}

// Device registry to maintain a list of all registered devices
type DeviceRegistry = Vec<Arc<Mutex<dyn Device>>>;
static mut DEVICE_REGISTRY: Option<DeviceRegistry> = None;

/// Initialize the device registry
pub fn init() -> Result<(), KernelError> {
    unsafe {
        DEVICE_REGISTRY = Some(Vec::new());
    }
    
    // Initialize hardware drivers first
    serial_println!("DEBUG: Initializing hardware drivers through drivers module");
    if let Err(e) = crate::drivers::init() {
        serial_println!("DEBUG: Warning: Some driver initialization failed: {:?}", e);
        // Continue despite driver errors
    }
    
    // Initialize and register storage devices
    probe_storage_devices()?;
    
    Ok(())
}

/// Register a device with the system
pub fn register_device(device: Arc<Mutex<dyn Device>>) -> Result<u64, KernelError> {
    let registry = unsafe {
        DEVICE_REGISTRY.as_mut().ok_or(KernelError::NotInitialized)?
    };
    
    let id = {
        let device_guard = device.lock();
        device_guard.id()
    };
    
    registry.push(device.clone());
    
    Ok(id)
}

/// Probe for storage devices and register them
fn probe_storage_devices() -> Result<(), KernelError> {
    serial_println!("DEBUG: Probing for storage devices...");
    
    // Create a new ATA device
    let ata_device = Arc::new(Mutex::new(ata::AtaDevice::new()));
    
    // Try to initialize it
    {
        let mut device_guard = ata_device.lock();
        match device_guard.initialize() {
            Ok(_) => {
                serial_println!("DEBUG: ATA device initialized successfully");
                serial_println!("DEBUG: {}", device_guard.debug_info());
            }
            Err(e) => {
                serial_println!("DEBUG: Failed to initialize ATA device: {:?}", e);
                // We'll still register it, just in an uninitialized state
            }
        }
    }
    
    // Register the device
    register_device(ata_device)?;
    
    Ok(())
}

/// Get all devices of a specific type
pub fn get_devices_by_type(device_type: DeviceType) -> Vec<Arc<Mutex<dyn Device>>> {
    let registry = unsafe {
        match DEVICE_REGISTRY.as_ref() {
            Some(reg) => reg,
            None => return Vec::new(),
        }
    };
    
    registry.iter()
        .filter(|dev| {
            let device_guard = dev.lock();
            device_guard.device_type() == device_type
        })
        .cloned()
        .collect()
}

/// Get a specific device by ID
pub fn get_device_by_id(id: u64) -> Option<Arc<Mutex<dyn Device>>> {
    let registry = unsafe {
        match DEVICE_REGISTRY.as_ref() {
            Some(reg) => reg,
            None => return None,
        }
    };
    
    registry.iter()
        .find(|dev| {
            let device_guard = dev.lock();
            device_guard.id() == id
        })
        .cloned()
}

/// Get all block devices
pub fn get_block_devices() -> Vec<Arc<Mutex<dyn Device>>> {
    get_devices_by_type(DeviceType::Block)
} 