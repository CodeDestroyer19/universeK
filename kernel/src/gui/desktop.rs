//! Desktop module for UniverseK OS GUI
//! Manages the desktop environment, including icons, taskbar, and windows

use crate::drivers::vga_enhanced::{self, Color};
use crate::serial_println;
use crate::errors::KernelError;
use crate::gui::window::{Window, WindowHandle};
use crate::gui::app::AppIcon;
use alloc::string::ToString;
use alloc::vec::Vec;
use spin::Mutex;
use lazy_static::lazy_static;

/// Colors for the desktop environment
pub const DESKTOP_BACKGROUND: Color = Color::Blue;
pub const DESKTOP_TEXT: Color = Color::White;
pub const TASKBAR_BACKGROUND: Color = Color::LightGray;
pub const TASKBAR_TEXT: Color = Color::Black;
pub const ICON_BACKGROUND: Color = Color::Cyan;
pub const ICON_TEXT: Color = Color::Black;

/// Desktop state
lazy_static! {
    pub static ref DESKTOP: Mutex<Desktop> = Mutex::new(Desktop::new());
}

/// Desktop representation
pub struct Desktop {
    /// Icons on the desktop
    icons: Vec<AppIcon>,
    /// Currently open windows
    windows: Vec<WindowHandle>,
    /// Active window index
    active_window: Option<usize>,
    /// Mouse position
    mouse_x: usize,
    mouse_y: usize,
    /// Taskbar information
    start_menu_open: bool,
    taskbar_height: usize,
    // Exit flag
    exit_requested: bool,
}

impl Desktop {
    /// Create a new desktop
    pub fn new() -> Self {
        Self {
            icons: Vec::new(),
            windows: Vec::new(),
            active_window: None,
            mouse_x: 0,
            mouse_y: 0,
            start_menu_open: false,
            taskbar_height: 2,
            exit_requested: false,
        }
    }
    
    /// Add an icon to the desktop
    pub fn add_icon(&mut self, icon: AppIcon) {
        self.icons.push(icon);
    }
    
    /// Add a window to the desktop
    pub fn add_window(&mut self, window: Mutex<Window>) -> WindowHandle {
        let handle = WindowHandle::new(window);
        self.windows.push(handle.clone());
        self.active_window = Some(self.windows.len() - 1);
        handle
    }
    
    /// Set the mouse position
    pub fn set_mouse_position(&mut self, x: usize, y: usize) {
        self.mouse_x = x;
        self.mouse_y = y;
    }
    
    /// Get the mouse position
    pub fn mouse_position(&self) -> (usize, usize) {
        (self.mouse_x, self.mouse_y)
    }
    
    /// Check if a point is inside the taskbar
    pub fn is_in_taskbar(&self, x: usize, y: usize) -> bool {
        y >= 25 - self.taskbar_height && y < 25
    }
    
    /// Check if a point is inside the start button
    pub fn is_in_start_button(&self, x: usize, y: usize) -> bool {
        self.is_in_taskbar(x, y) && x < 8
    }
    
    /// Toggle the start menu
    pub fn toggle_start_menu(&mut self) {
        self.start_menu_open = !self.start_menu_open;
    }
    
    /// Request exit from the GUI
    pub fn request_exit(&mut self) {
        self.exit_requested = true;
    }
    
    /// Check if exit has been requested
    pub fn should_exit(&self) -> bool {
        self.exit_requested
    }
    
    /// Get the active window index
    pub fn get_active_window(&self) -> Option<usize> {
        self.active_window
    }
    
    /// Get a reference to the windows list
    pub fn get_windows(&self) -> &Vec<WindowHandle> {
        &self.windows
    }
}

/// Initialize the desktop
pub fn init() -> Result<(), KernelError> {
    serial_println!("DEBUG: Initializing desktop");
    // No additional initialization needed yet
    Ok(())
}

/// Draw the desktop environment
pub fn draw() -> Result<(), KernelError> {
    serial_println!("DEBUG: Drawing desktop");
    
    // Clear the screen with desktop background
    vga_enhanced::clear_screen();
    
    // Draw the taskbar
    draw_taskbar()?;
    
    // Draw the desktop icons
    let desktop = DESKTOP.lock();
    for (i, icon) in desktop.icons.iter().enumerate() {
        let x = 2 + (i % 4) * 15;
        let y = 2 + (i / 4) * 4;
        draw_icon(icon, x, y)?;
    }
    
    // Draw windows
    for (i, window) in desktop.windows.iter().enumerate() {
        let is_active = desktop.active_window.map_or(false, |active| active == i);
        let mut window = window.lock();
        window.draw(is_active)?;
    }
    
    // Draw the mouse cursor
    draw_mouse_cursor(desktop.mouse_x, desktop.mouse_y)?;
    
    Ok(())
}

/// Refresh the desktop display
pub fn refresh() -> Result<(), KernelError> {
    // Just redraw everything for now
    draw()
}

/// Draw the taskbar at the bottom of the screen
fn draw_taskbar() -> Result<(), KernelError> {
    // Draw taskbar background
    for y in 23..25 {
        for x in 0..80 {
            vga_enhanced::write_at(y, x, " ", TASKBAR_TEXT, TASKBAR_BACKGROUND);
        }
    }
    
    // Draw start button
    vga_enhanced::write_at(24, 1, "START", Color::White, Color::Green);
    
    // Draw taskbar divider
    vga_enhanced::write_at(24, 8, "|", TASKBAR_TEXT, TASKBAR_BACKGROUND);
    
    // Draw clock on the right
    vga_enhanced::write_at(24, 70, "12:00 PM", TASKBAR_TEXT, TASKBAR_BACKGROUND);
    
    Ok(())
}

/// Draw a desktop icon
fn draw_icon(icon: &AppIcon, x: usize, y: usize) -> Result<(), KernelError> {
    // Draw icon background
    for iy in 0..3 {
        for ix in 0..10 {
            vga_enhanced::write_at(y + iy, x + ix, " ", ICON_TEXT, ICON_BACKGROUND);
        }
    }
    
    // Draw icon label
    let name = if icon.name.len() > 8 {
        // Truncate with ...
        let mut short = icon.name[..5].to_string();
        short.push_str("...");
        short
    } else {
        icon.name.clone()
    };
    
    let padding = (10 - name.len()) / 2;
    vga_enhanced::write_at(y + 1, x + padding, &name, ICON_TEXT, ICON_BACKGROUND);
    
    Ok(())
}

/// Draw the mouse cursor
fn draw_mouse_cursor(x: usize, y: usize) -> Result<(), KernelError> {
    // Simple cursor representation
    if x < 80 && y < 25 {
        let current_char = vga_enhanced::read_char_at(y, x);
        vga_enhanced::write_at(y, x, "X", Color::White, Color::Red);
    }
    
    Ok(())
}

/// Handle a mouse click on the desktop
pub fn handle_mouse_click(x: usize, y: usize) -> Result<(), KernelError> {
    let mut desktop = DESKTOP.lock();
    
    // Check if click is on start button
    if desktop.is_in_start_button(x, y) {
        desktop.toggle_start_menu();
        return Ok(());
    }
    
    // Check if click is on desktop icon
    for (i, icon) in desktop.icons.iter().enumerate() {
        let icon_x = 2 + (i % 4) * 15;
        let icon_y = 2 + (i / 4) * 4;
        
        if x >= icon_x && x < icon_x + 10 && y >= icon_y && y < icon_y + 3 {
            // Click on icon - launch app
            serial_println!("DEBUG: Launching app: {}", icon.name);
            
            // Create an instance of the app
            if let Some(ref create_fn) = icon.create_fn {
                let handle = create_fn()?;
                let mut desktop = DESKTOP.lock();
                desktop.active_window = Some(desktop.windows.len() - 1);
                return Ok(());
            }
        }
    }
    
    // Store windows in a temporary vec to avoid borrowing issues
    let mut windows_to_check = Vec::new();
    for (i, window) in desktop.windows.iter().enumerate() {
        windows_to_check.push((i, window.clone()));
    }
    
    // Drop the desktop lock before processing windows
    drop(desktop);
    
    // Check if click is on a window
    for (i, window) in windows_to_check {
        let window_guard = window.lock();
        if window_guard.contains_point(x, y) {
            // Set as active window
            let mut desktop = DESKTOP.lock();
            desktop.active_window = Some(i);
            
            // Check if click is on close button
            if window_guard.is_on_close_button(x, y) {
                // Close window logic
                drop(window_guard); // Release the lock
                let _ = desktop.windows.remove(i);
                if desktop.windows.is_empty() {
                    desktop.active_window = None;
                } else {
                    desktop.active_window = Some(desktop.windows.len() - 1);
                }
                return Ok(());
            }
            
            // Pass click to the window
            drop(window_guard); // Release the lock
            drop(desktop);     // Release desktop lock
            let mut window = window.lock();
            window.handle_click(x, y)?;
            return Ok(());
        }
    }
    
    Ok(())
}

/// Add an icon to the desktop
pub fn add_icon(icon: AppIcon) -> Result<(), KernelError> {
    let mut desktop = DESKTOP.lock();
    desktop.add_icon(icon);
    Ok(())
}

/// Get the exit flag
pub fn should_exit() -> bool {
    let desktop = DESKTOP.lock();
    desktop.should_exit()
}

/// Request exit from the GUI
pub fn request_exit() {
    let mut desktop = DESKTOP.lock();
    desktop.request_exit();
} 