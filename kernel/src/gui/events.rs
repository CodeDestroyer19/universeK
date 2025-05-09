//! Events module for UniverseK OS GUI
//! Handles user input events for the graphical user interface

use crate::serial_println;
use crate::errors::KernelError;
use crate::drivers::ps2_keyboard::{KeyCode, KeyEvent, KeyState};
use crate::drivers::ps2_mouse::{MouseEvent, MouseButtons};
use crate::gui::desktop;

/// Handle a mouse event
pub fn handle_mouse_event(event: MouseEvent) -> Result<(), KernelError> {
    serial_println!("DEBUG: GUI received mouse event: x={}, y={}, btn_left={}, btn_right={}",
        event.x, event.y, event.buttons.left, event.buttons.right);
    
    // Update mouse position in desktop
    let x = (event.x / 8) as usize; // Convert to character coordinates
    let y = (event.y / 16) as usize;
    
    // Ensure coordinates are in bounds
    let x = x.min(79);
    let y = y.min(24);
    
    {
        let mut desktop_guard = desktop::DESKTOP.lock();
        desktop_guard.set_mouse_position(x, y);
    }
    
    // Handle mouse button clicks
    if event.buttons.left {
        // Left button - trigger click event
        desktop::handle_mouse_click(x, y)?;
    }
    
    // Refresh the display to show the new mouse position
    desktop::refresh()?;
    
    Ok(())
}

/// Handle a keyboard event
pub fn handle_keyboard_event(event: KeyEvent) -> Result<(), KernelError> {
    serial_println!("DEBUG: GUI received keyboard event: code={:?}, state={:?}",
        event.code, event.state);
    
    // Only process key press events
    if event.state != KeyState::Pressed {
        return Ok(());
    }
    
    // Check for global keyboard shortcuts first
    match event.code {
        KeyCode::Escape if event.ctrl => {
            // Ctrl+Esc - toggle start menu
            let mut desktop_guard = desktop::DESKTOP.lock();
            desktop_guard.toggle_start_menu();
            drop(desktop_guard);
            desktop::refresh()?;
            return Ok(());
        },
        KeyCode::Q if event.ctrl && event.alt => {
            // Ctrl+Alt+Q - quit GUI
            desktop::request_exit();
            return Ok(());
        },
        _ => {}
    }
    
    // Refresh display - we'll only handle global keys for now
    // until we add proper accessor methods for the desktop's active window
    desktop::refresh()?;
    
    Ok(())
}

/// Check if the GUI should exit
pub fn should_exit() -> bool {
    desktop::should_exit()
} 