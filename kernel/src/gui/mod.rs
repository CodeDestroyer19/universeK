//! GUI subsystem for UniverseK OS
//! Implements a Windows-like GUI with a desktop, windows, and clickable applications

pub mod window;
pub mod desktop;
pub mod app;
pub mod events;

use crate::drivers::vga_enhanced::{self, Color};
use crate::drivers::ps2_mouse;
use crate::serial_println;
use crate::errors::KernelError;
use alloc::vec::Vec;
use alloc::sync::Arc;
use spin::Mutex;

/// Initialize the GUI subsystem
pub fn init() -> Result<(), KernelError> {
    serial_println!("DEBUG: Initializing GUI subsystem");
    
    // Initialize the desktop
    desktop::init()?;
    
    // Add basic applications to the desktop
    app::register_default_apps()?;
    
    serial_println!("DEBUG: GUI subsystem initialized successfully");
    Ok(())
}

/// Start the GUI (blocking)
pub fn run() -> Result<(), KernelError> {
    serial_println!("DEBUG: Starting GUI main loop");
    
    // Draw the desktop
    desktop::draw()?;
    
    // Main GUI loop
    let mut loop_count = 0;
    
    loop {
        // Check for mouse events
        if let Some(event) = ps2_mouse::get_event() {
            // Handle mouse event
            events::handle_mouse_event(event)?;
        }
        
        // Check for keyboard events
        if let Some(event) = crate::drivers::ps2_keyboard::get_event() {
            // Handle keyboard event
            events::handle_keyboard_event(event)?;
        }
        
        // Periodic redraw
        if loop_count % 10_000_000 == 0 {
            desktop::refresh()?;
        }
        
        // Check for exit request
        if events::should_exit() {
            break;
        }
        
        // Use HLT to save CPU when possible
        if loop_count % 1000 == 0 {
            x86_64::instructions::hlt();
        }
        
        loop_count += 1;
    }
    
    serial_println!("DEBUG: GUI main loop exited");
    Ok(())
} 