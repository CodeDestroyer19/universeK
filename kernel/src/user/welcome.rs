//! Welcome screen for new users
//! Displays a user-friendly interface after system boot

use alloc::format;
use crate::alloc::string::ToString;
use crate::drivers::vga_enhanced::{self, Color};
use crate::user;
use crate::errors::KernelError;
use crate::fs;

/// Display the welcome screen
pub fn show() -> Result<(), KernelError> {
    // Clear the screen
    vga_enhanced::clear_screen();
    
    // Draw the main window
    vga_enhanced::draw_shadowed_box(5, 1, 70, 22);
    
    // Get current user info
    let username = user::USER_MANAGER.lock().get_current_user()
        .map_or("Guest".to_string(), |u| u.username.clone());
    
    // Draw title bar
    vga_enhanced::write_at(0, 24, " ".repeat(80).as_str(),
        Color::White, Color::Blue);
    
    vga_enhanced::write_at(0, 2, &format!(" UniverseK OS - Welcome {} ", username),
        Color::White, Color::Blue);
    
    vga_enhanced::write_at(0, 69, " [ESC] Exit ",
        Color::White, Color::Blue);
    
    // Draw system information
    draw_system_info()?;
    
    // Draw user information
    draw_user_info()?;
    
    // Draw options menu
    draw_options_menu();
    
    // Wait for keyboard input
    wait_for_keypress();
    
    Ok(())
}

/// Draw system information section
fn draw_system_info() -> Result<(), KernelError> {
    vga_enhanced::write_at(3, 8, "System Information:", 
        Color::Yellow, Color::Black);
    
    // Memory information
    let heap_size = crate::allocator::HEAP_SIZE / 1024;
    vga_enhanced::write_at(4, 10, &format!("Memory: {} KB available", heap_size),
        Color::White, Color::Black);
    
    // Filesystem information
    let vfs = fs::vfs::get_vfs_manager().ok_or(KernelError::NotInitialized)?;
    let root_entries = vfs.read_dir("/")?;
    
    vga_enhanced::write_at(5, 10, &format!("Filesystem: {} root entries", root_entries.len()),
        Color::White, Color::Black);
    
    // Get date/time
    let now = crate::drivers::rtc::get_datetime();
    vga_enhanced::write_at(6, 10, &format!("Date/Time: {}", now.format()),
        Color::White, Color::Black);
    
    Ok(())
}

/// Draw user information section
fn draw_user_info() -> Result<(), KernelError> {
    vga_enhanced::write_at(9, 8, "User Information:", 
        Color::Yellow, Color::Black);
    
    // Get current user or display guest message
    if let Some(current_user) = user::USER_MANAGER.lock().get_current_user() {
        vga_enhanced::write_at(10, 10, &format!("Username: {}", current_user.username),
            Color::White, Color::Black);
        
        vga_enhanced::write_at(11, 10, &format!("Full Name: {}", current_user.full_name),
            Color::White, Color::Black);
        
        vga_enhanced::write_at(12, 10, &format!("Home Directory: {}", current_user.home_dir),
            Color::White, Color::Black);
        
        // List files in home directory
        let vfs = fs::vfs::get_vfs_manager().ok_or(KernelError::NotInitialized)?;
        match vfs.read_dir(&current_user.home_dir) {
            Ok(entries) => {
                let mut file_info = format!("Home contains {} items", entries.len());
                if entries.len() <= 3 {
                    file_info.push_str(": ");
                    for (i, entry) in entries.iter().enumerate() {
                        if i > 0 {
                            file_info.push_str(", ");
                        }
                        file_info.push_str(&entry.name);
                    }
                }
                
                vga_enhanced::write_at(13, 10, &file_info,
                    Color::LightCyan, Color::Black);
            },
            Err(e) => {
                vga_enhanced::write_at(13, 10, &format!("Error reading home dir: {:?}", e),
                    Color::LightRed, Color::Black);
            }
        }
    } else {
        vga_enhanced::write_at(10, 10, "No user is currently logged in.",
            Color::LightRed, Color::Black);
        
        vga_enhanced::write_at(11, 10, "Please create a user account to continue.",
            Color::White, Color::Black);
    }
    
    Ok(())
}

/// Draw options menu
fn draw_options_menu() {
    vga_enhanced::write_at(16, 8, "Options:", 
        Color::Yellow, Color::Black);
    
    vga_enhanced::write_at(17, 10, "1. Create New User",
        Color::White, Color::Black);
    
    vga_enhanced::write_at(18, 10, "2. Explore Files",
        Color::White, Color::Black);
    
    vga_enhanced::write_at(19, 10, "3. System Settings",
        Color::White, Color::Black);
    
    vga_enhanced::write_at(20, 10, "4. Restart System",
        Color::White, Color::Black);
    
    vga_enhanced::write_at(22, 8, "Press a number key to select an option...",
        Color::LightCyan, Color::Black);
}

/// Wait for a keypress
fn wait_for_keypress() {
    // Wait for a key press - for now just a placeholder
    // In a real implementation, this would wait for a key and take action
}

/// Handle new user creation
fn create_new_user() -> Result<(), KernelError> {
    // Clear options area
    for i in 16..23 {
        vga_enhanced::write_at(i, 8, " ".repeat(60).as_str(),
            Color::Black, Color::Black);
    }
    
    vga_enhanced::write_at(16, 8, "Create New User:", 
        Color::Yellow, Color::Black);
    
    vga_enhanced::write_at(17, 10, "Username: _____________",
        Color::White, Color::Black);
    
    vga_enhanced::write_at(18, 10, "Full Name: _____________",
        Color::White, Color::Black);
    
    // In a real implementation, this would handle text input
    // For now, just create a default user
    
    let username = "user1";
    let full_name = "User One";
    
    match user::create_user(username, full_name) {
        Ok(_) => {
            vga_enhanced::write_at(20, 10, &format!("User '{}' created successfully!", username),
                Color::LightGreen, Color::Black);
        },
        Err(e) => {
            vga_enhanced::write_at(20, 10, &format!("Error creating user: {:?}", e),
                Color::LightRed, Color::Black);
        }
    }
    
    Ok(())
}

/// Explore files interface
fn explore_files() -> Result<(), KernelError> {
    // Clear options area
    for i in 16..23 {
        vga_enhanced::write_at(i, 8, " ".repeat(60).as_str(),
            Color::Black, Color::Black);
    }
    
    vga_enhanced::write_at(16, 8, "File Explorer:", 
        Color::Yellow, Color::Black);
    
    // Get root directory listing
    let vfs = fs::vfs::get_vfs_manager().ok_or(KernelError::NotInitialized)?;
    let entries = vfs.read_dir("/")?;
    
    vga_enhanced::write_at(17, 10, &format!("Root directory: {} items", entries.len()),
        Color::White, Color::Black);
    
    // Display up to 4 entries
    for (i, entry) in entries.iter().take(4).enumerate() {
        let type_str = match entry.node_type {
            fs::vfs::NodeType::Directory => "[DIR]",
            fs::vfs::NodeType::File => "[FILE]",
            _ => "[OTHER]",
        };
        
        vga_enhanced::write_at(18 + i, 12, &format!("{} {}", type_str, entry.name),
            Color::White, Color::Black);
    }
    
    // If there are more entries, show a message
    if entries.len() > 4 {
        vga_enhanced::write_at(22, 12, &format!("... and {} more items", entries.len() - 4),
            Color::LightCyan, Color::Black);
    }
    
    Ok(())
}

/// Run the welcome screen with interactive menu
pub fn run() -> Result<(), KernelError> {
    show()?;
    
    // In a real implementation, this would handle user input and call the appropriate functions
    // For now, let's just call one of them to demonstrate
    explore_files()?;
    
    Ok(())
} 