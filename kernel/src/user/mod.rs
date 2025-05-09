//! User management for the kernel.
//! Handles user accounts, home directories, and permissions.

pub mod welcome; // Welcome screen module

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use spin::Mutex;
use lazy_static::lazy_static;
use crate::errors::KernelError;
use crate::fs;
use crate::fs::vfs::get_vfs_manager;
use crate::serial_println;

/// Basic user structure
#[derive(Debug, Clone)]
pub struct User {
    /// Username (login name)
    pub username: String,
    /// User ID
    pub uid: u32,
    /// Group ID
    pub gid: u32,
    /// Home directory path
    pub home_dir: String,
    /// Full name
    pub full_name: String,
}

impl User {
    /// Create a new user with default settings
    pub fn new(username: &str, uid: u32) -> Self {
        Self {
            username: username.to_string(),
            uid,
            gid: uid, // By default, primary group = user id
            home_dir: format!("/Users/{}", username),
            full_name: username.to_string(), // Default to username
        }
    }
}

/// User management system
pub struct UserManager {
    users: Vec<User>,
    next_uid: u32,
    current_user: Option<u32>,
}

impl UserManager {
    /// Create a new user manager
    pub fn new() -> Self {
        Self {
            users: Vec::new(),
            next_uid: 1000, // Start UIDs at 1000 for regular users
            current_user: None,
        }
    }
    
    /// Add a new user
    pub fn add_user(&mut self, username: &str, full_name: &str) -> Result<&User, KernelError> {
        // Check if username already exists
        if self.users.iter().any(|u| u.username == username) {
            return Err(KernelError::AlreadyExists);
        }
        
        // Create the new user
        let uid = self.next_uid;
        self.next_uid += 1;
        
        let mut user = User::new(username, uid);
        user.full_name = full_name.to_string();
        
        // Create user directories
        self.create_user_directories(&user)?;
        
        // Add to our list
        self.users.push(user);
        
        Ok(self.users.last().unwrap())
    }
    
    /// Create the standard directories for a user
    fn create_user_directories(&self, user: &User) -> Result<(), KernelError> {
        // Get the VFS
        let vfs = get_vfs_manager().ok_or(KernelError::NotInitialized)?;
        
        // Create home directory
        serial_println!("Creating home directory: {}", user.home_dir);
        vfs.create_directory(&user.home_dir)?;
        
        // Create standard subdirectories
        let dirs = [
            "Documents",
            "Downloads",
            "Desktop",
            "Pictures",
            "Music",
            "Movies",
            "Library",
            "Library/Preferences",
            "Library/Application Support",
        ];
        
        for dir in dirs.iter() {
            let path = format!("{}/{}", user.home_dir, dir);
            serial_println!("Creating directory: {}", path);
            vfs.create_directory(&path)?;
        }
        
        Ok(())
    }
    
    /// Set the current active user
    pub fn set_current_user(&mut self, uid: u32) -> Result<(), KernelError> {
        if self.users.iter().any(|u| u.uid == uid) {
            self.current_user = Some(uid);
            Ok(())
        } else {
            Err(KernelError::NotFound)
        }
    }
    
    /// Get the current active user
    pub fn get_current_user(&self) -> Option<&User> {
        if let Some(uid) = self.current_user {
            self.users.iter().find(|u| u.uid == uid)
        } else {
            None
        }
    }
    
    /// Get user by username
    pub fn get_user_by_name(&self, username: &str) -> Option<&User> {
        self.users.iter().find(|u| u.username == username)
    }
    
    /// Get user by ID
    pub fn get_user_by_id(&self, uid: u32) -> Option<&User> {
        self.users.iter().find(|u| u.uid == uid)
    }
}

lazy_static! {
    pub static ref USER_MANAGER: Mutex<UserManager> = Mutex::new(UserManager::new());
}

/// Initialize the user management system
pub fn init() -> Result<(), KernelError> {
    serial_println!("Initializing user management system");
    
    // Create system users
    let mut manager = USER_MANAGER.lock();
    
    // Add root user
    let root = User {
        username: "root".to_string(),
        uid: 0,
        gid: 0,
        home_dir: "/root".to_string(),
        full_name: "System Administrator".to_string(),
    };
    manager.users.push(root);
    
    // Add system user
    let system = User {
        username: "system".to_string(),
        uid: 1,
        gid: 1,
        home_dir: "/System".to_string(),
        full_name: "System Services".to_string(),
    };
    manager.users.push(system);
    
    Ok(())
}

/// Set up initial file system for a new system
pub fn setup_filesystem() -> Result<(), KernelError> {
    serial_println!("Setting up initial file system structure");
    
    // Get VFS manager
    let vfs = get_vfs_manager().ok_or(KernelError::NotInitialized)?;

    // Create essential top-level directories only
    serial_println!("Creating only essential top-level directories - avoiding deep nesting");
    let top_dirs = [
        "/System",
        "/Library", 
        "/Applications",
        "/Users",
        "/root",
        "/tmp"
    ];

    // Create only top-level directories to avoid the problematic paths
    for dir in top_dirs.iter() {
        serial_println!("Creating top-level directory: {}", dir);
        match vfs.create_directory(dir) {
            Ok(_) => serial_println!("Successfully created directory: {}", dir),
            Err(KernelError::AlreadyExists) => serial_println!("Directory already exists: {}", dir),
            Err(e) => {
                serial_println!("ERROR creating directory {}: {:?}", dir, e);
                // Continue despite errors
            },
        }
        
        // Add delay between directory creations
        for _ in 0..5000 { core::hint::spin_loop(); }
    }

    // AVOID creating System/Library/Frameworks which causes the hang
    serial_println!("IMPORTANT: Skipping creation of /System/Library/Frameworks and other deep paths");
    serial_println!("Those paths will be created on demand if needed");

    // Create welcome message in root directory
    serial_println!("Creating welcome.txt file");
    let welcome_message = b"Welcome to UniverseK OS!\nThis is a basic Unix-like operating system.\n";
    match vfs.create_file("/welcome.txt") {
        Ok(_) => {
            // Write to the file
            serial_println!("Writing welcome message to welcome.txt");
            match fs::direct_write_file("/welcome.txt", welcome_message) {
                Ok(bytes) => serial_println!("Successfully wrote {} bytes to welcome.txt", bytes),
                Err(e) => serial_println!("ERROR writing to welcome.txt: {:?}", e),
            }
        },
        Err(KernelError::AlreadyExists) => serial_println!("File already exists: /welcome.txt"),
        Err(e) => serial_println!("ERROR creating welcome.txt: {:?}", e),
    }
    
    serial_println!("Filesystem setup completed - skipped problematic paths");
    Ok(())
}

/// Create a new user with default settings
pub fn create_user(username: &str, full_name: &str) -> Result<(), KernelError> {
    USER_MANAGER.lock().add_user(username, full_name)?;
    Ok(())
}

/// Interactive user setup for a new system
pub fn run_new_user_setup() -> Result<(), KernelError> {
    serial_println!("Running new user setup");
    
    // For now, just create a default user since we don't have proper input yet
    // In a real implementation, this would prompt for username, full name, etc.
    let username = "user";
    let full_name = "Default User";
    
    // Create the user
    match create_user(username, full_name) {
        Ok(_) => {
            serial_println!("Created default user: {} ({})", full_name, username);
            // Set as current user
            USER_MANAGER.lock().set_current_user(1000)?;
        },
        Err(KernelError::AlreadyExists) => {
            serial_println!("Default user already exists");
        },
        Err(e) => return Err(e),
    }
    
    Ok(())
} 