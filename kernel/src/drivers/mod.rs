//! Hardware device drivers for the kernel

pub mod ps2_keyboard;
pub mod ps2_mouse;
pub mod vga_enhanced;
pub mod pci;
pub mod pit;
pub mod rtc;

use crate::errors::KernelError;
use crate::serial_println;

/// Initialize all drivers
pub fn init() -> Result<(), KernelError> {
    serial_println!("DEBUG: Initializing device drivers");
    
    // Initialize PCI first to discover hardware
    pci::init()?;
    
    // Initialize display driver
    vga_enhanced::init()?;
    
    // Initialize input devices
    ps2_keyboard::init()?;
    ps2_mouse::init()?;
    
    // Initialize timers
    pit::init(100)?;
    rtc::init()?;
    
    serial_println!("DEBUG: Device drivers initialized");
    Ok(())
}

/// Driver interface trait for consistent driver management
pub trait Driver {
    /// Initialize the driver
    fn init(&mut self) -> Result<(), KernelError>;
    
    /// Name of the driver
    fn name(&self) -> &str;
    
    /// Shut down the driver
    fn shutdown(&mut self) -> Result<(), KernelError>;
} 