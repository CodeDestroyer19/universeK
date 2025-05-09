//! Hardware device drivers for the kernel

pub mod vga_enhanced;
pub mod ps2_keyboard;
pub mod ps2_mouse;
pub mod pit;
pub mod rtc;
pub mod pci;

use crate::errors::KernelError;
use crate::serial_println;

/// Initialize essential device drivers
pub fn init() -> Result<(), KernelError> {
    serial_println!("DEBUG: Initializing device drivers");
    
    // Initialize PS/2 keyboard
    serial_println!("DEBUG: Initializing PS/2 keyboard");
    if let Err(e) = ps2_keyboard::init() {
        serial_println!("WARNING: Failed to initialize PS/2 keyboard: {:?}", e);
        // Continue even if keyboard init fails
    } else {
        serial_println!("DEBUG: PS/2 keyboard initialized successfully");
    }
    
    // Initialize PS/2 mouse
    serial_println!("DEBUG: Initializing PS/2 mouse");
    if let Err(e) = ps2_mouse::init() {
        serial_println!("WARNING: Failed to initialize PS/2 mouse: {:?}", e);
        // Continue even if mouse init fails
    } else {
        serial_println!("DEBUG: PS/2 mouse initialized successfully");
    }
    
    // Initialize PIT for system timer
    serial_println!("DEBUG: Initializing PIT timer");
    if let Err(e) = pit::init(100) { // 100 Hz timer frequency
        serial_println!("WARNING: Failed to initialize PIT timer: {:?}", e);
        // Continue even if timer init fails
    } else {
        serial_println!("DEBUG: PIT timer initialized successfully");
    }
    
    serial_println!("DEBUG: Essential drivers initialized");
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