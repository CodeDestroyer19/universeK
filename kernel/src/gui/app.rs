//! App module for UniverseK OS GUI
//! Defines and manages applications for the graphical user interface

use crate::drivers::vga_enhanced::{self, Color};
use crate::serial_println;
use crate::errors::KernelError;
use crate::gui::window::{Window, WindowHandle, create_window};
use crate::gui::desktop;
use alloc::string::String;
use alloc::string::ToString;
use alloc::boxed::Box;
use alloc::format;
use alloc::vec::Vec;
use alloc::rc::Rc;
use spin::Mutex;

/// Callback type for creating an app window
pub type AppCreateFn = Box<dyn Fn() -> Result<WindowHandle, KernelError> + Send + Sync>;

/// Represents an application icon on the desktop
pub struct AppIcon {
    /// Application name
    pub name: String,
    /// Function to create an instance of the app
    pub create_fn: Option<AppCreateFn>,
}

impl AppIcon {
    /// Create a new application icon
    pub fn new(name: &str, create_fn: AppCreateFn) -> Self {
        Self {
            name: name.to_string(),
            create_fn: Some(create_fn),
        }
    }
}

/// Register the default applications for the GUI
pub fn register_default_apps() -> Result<(), KernelError> {
    serial_println!("DEBUG: Registering default applications");
    
    // Register Terminal app
    desktop::add_icon(AppIcon::new("Terminal", Box::new(create_terminal_app)))?;
    
    // Register About app
    desktop::add_icon(AppIcon::new("About", Box::new(create_about_app)))?;
    
    // Register File Explorer app
    desktop::add_icon(AppIcon::new("Files", Box::new(create_files_app)))?;
    
    // Register Settings app
    desktop::add_icon(AppIcon::new("Settings", Box::new(create_settings_app)))?;
    
    Ok(())
}

/// Create a terminal app window
fn create_terminal_app() -> Result<WindowHandle, KernelError> {
    serial_println!("DEBUG: Creating terminal app window");
    
    // Create a terminal window
    let window_handle = create_window("Terminal", 10, 2, 60, 18);
    
    // Clone for the closure
    let window_handle_for_closure = window_handle.clone();
    
    // Set up terminal functionality
    {
        let mut window = window_handle.lock();
        window.add_text("UniverseK OS Terminal\n");
        window.add_text("Type 'help' for a list of commands\n");
        
        // Set up input handling with a closure that owns its own copy of the handle
        window.enable_input(Box::new(move |input| {
            // Simple command handling logic
            match input.trim() {
                "help" => {
                    let mut window = window_handle_for_closure.lock();
                    window.add_text("Available commands:\n");
                    window.add_text("  help - Display this help message\n");
                    window.add_text("  clear - Clear the screen\n");
                    window.add_text("  exit - Close this terminal\n");
                    window.add_text("  about - Display system information\n");
                }
                "clear" => {
                    let mut window = window_handle_for_closure.lock();
                    window.clear();
                }
                "exit" => {
                    // Will be handled by desktop by closing window
                    return Ok(());
                }
                "about" => {
                    let mut window = window_handle_for_closure.lock();
                    window.add_text("UniverseK OS v0.1.0\n");
                    window.add_text("A simple operating system for learning\n");
                }
                "" => {}
                _ => {
                    let mut window = window_handle_for_closure.lock();
                    window.add_text(&format!("Unknown command: {}\n", input));
                }
            }
            
            Ok(())
        }));
    }
    
    // Add window to desktop
    let desktop = desktop::DESKTOP.lock();
    // We're returning the window handle directly, no need to add it here
    // The desktop management will need to be refactored later
    
    Ok(window_handle)
}

/// Create an about app window
fn create_about_app() -> Result<WindowHandle, KernelError> {
    serial_println!("DEBUG: Creating about app window");
    
    // Create an about window
    let window_handle = create_window("About UniverseK OS", 15, 5, 50, 15);
    
    {
        let mut window = window_handle.lock();
        window.add_text("UniverseK OS v0.1.0\n\n");
        window.add_text("A simple operating system for learning.\n");
        window.add_text("Features:\n");
        window.add_text("- Custom bootloader\n");
        window.add_text("- Memory management\n");
        window.add_text("- File system support\n");
        window.add_text("- Simple GUI environment\n\n");
        window.add_text("Created as a learning project.\n");
    }
    
    Ok(window_handle)
}

/// Create a file explorer app window
fn create_files_app() -> Result<WindowHandle, KernelError> {
    serial_println!("DEBUG: Creating file explorer app window");
    
    // Create a file explorer window
    let window_handle = create_window("File Explorer", 5, 3, 55, 16);
    
    {
        let mut window = window_handle.lock();
        window.add_text("File Explorer\n\n");
        
        // Show root directory contents
        window.add_text("Contents of /:\n");
        
        // Try to read the directory if file system is available
        match crate::fs::vfs::get_vfs_manager() {
            Some(vfs) => {
                match vfs.read_dir("/") {
                    Ok(entries) => {
                        if entries.is_empty() {
                            window.add_text("  (empty directory)\n");
                        } else {
                            for entry in entries {
                                let type_indicator = match entry.node_type {
                                    crate::fs::vfs::NodeType::Directory => "/",
                                    crate::fs::vfs::NodeType::File => "",
                                    _ => "?",
                                };
                                window.add_text(&format!("  {}{}\n", entry.name, type_indicator));
                            }
                        }
                    },
                    Err(e) => {
                        window.add_text(&format!("Error reading directory: {:?}\n", e));
                    }
                }
            },
            None => {
                window.add_text("File system not initialized.\n");
            }
        }
    }
    
    Ok(window_handle)
}

/// Create a settings app window
fn create_settings_app() -> Result<WindowHandle, KernelError> {
    serial_println!("DEBUG: Creating settings app window");
    
    // Create a settings window
    let window_handle = create_window("System Settings", 20, 4, 40, 14);
    
    {
        let mut window = window_handle.lock();
        window.add_text("System Settings\n\n");
        window.add_text("Note: This is a mock settings app. Features are not\n");
        window.add_text("implemented yet.\n\n");
        window.add_text("[ ] Enable advanced CPU features\n");
        window.add_text("[ ] Use hardware acceleration\n");
        window.add_text("[ ] Show system stats in taskbar\n");
        window.add_text("[ ] Enable power saving\n");
    }
    
    Ok(window_handle)
} 