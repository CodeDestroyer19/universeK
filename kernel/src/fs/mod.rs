pub mod block_device;
pub mod block_adapter;
pub mod ramdisk;
pub mod simple_fs;
pub mod tempfs;
pub mod vfs;
pub mod fat;
pub mod fd;

use crate::serial_println;
use crate::errors::KernelError;
use alloc::sync::Arc;
use spin::Mutex;

/// Initialize the file system subsystem.
/// This sets up the VFS and mounts the initial file systems.
pub fn init() -> Result<(), KernelError> {
    serial_println!("DEBUG: Initializing file system subsystem (standard mode)");
    
    // Initialize the Virtual File System
    // VFS itself should be safe to initialize regardless of mode.
    match vfs::init() {
        Ok(_) => serial_println!("DEBUG: VFS initialized"),
        Err(e) => {
            serial_println!("DEBUG: CRITICAL: VFS initialization failed: {:?}", e);
            return Err(e); // VFS is critical
        }
    }
    
    // Initialize the file descriptor system
    // FD system should also be safe.
    match fd::init() {
        Ok(_) => serial_println!("DEBUG: File descriptor system initialized"),
        Err(e) => {
            serial_println!("DEBUG: CRITICAL: File descriptor system initialization failed: {:?}", e);
            return Err(e); // FD system is also critical
        }
    }
    
    // Try to initialize device-based file system first
    serial_println!("DEBUG: Attempting to initialize device-based file system.");
    if let Err(e) = init_device_fs() {
        serial_println!("DEBUG: Device-based file system init failed: {:?}. Falling back to RAM-based FS.", e);
        // Fallback uses the non-forcing init_ram_fs
        init_ram_fs()?;
    }
    
    Ok(())
}

/// Initialize a file system based on hardware devices
fn init_device_fs() -> Result<(), KernelError> {
    serial_println!("DEBUG: init_device_fs() called.");
    // Check if we have any block devices available
    let block_devices = crate::device::get_block_devices();
    
    if block_devices.is_empty() {
        serial_println!("DEBUG: No block devices found.");
        return Err(KernelError::DeviceNotFound);
    }
    
    serial_println!("DEBUG: Found {} block devices", block_devices.len());
    
    // Create an adapter for the first device
    let block_adapter = block_adapter::DeviceBlockAdapter::new_first_available()?;
    serial_println!("DEBUG: Using block device: {}", block_adapter.name());
    
    // Try to create a FAT file system on top of the device
    // This could fail if the device isn't formatted as FAT
    match fat::FatFileSystem::new(Arc::new(Mutex::new(block_adapter))) {
        Ok(fat_fs) => {
            let fs = Arc::new(Mutex::new(fat_fs));
            
            // Mount the FAT file system
            let vfs = vfs::get_vfs_manager().ok_or(KernelError::NotInitialized)?;
            vfs.mount("/", fs.clone())?;
            
            serial_println!("DEBUG: FAT file system mounted at /");
            
            // Store the mounted filesystem
            unsafe {
                GLOBAL_FS = Some(fs);
            }
            
            Ok(())
        }
        Err(e) => {
            serial_println!("DEBUG: Failed to create FAT file system: {:?}", e);
            Err(e)
        }
    }
}

/// Initialize a RAM-based file system
fn init_ram_fs() -> Result<(), KernelError> {
    serial_println!("DEBUG: Initializing RAM-based filesystem");
    
    // Default to TempFS as per original logic
    let use_tempfs_resolved = true; 
    serial_println!("DEBUG: RAM-based FS config: Using TempFS: {}", use_tempfs_resolved);
    
    if use_tempfs_resolved {
        serial_println!("DEBUG: Creating TempFS in-memory filesystem");
        let tempfs = tempfs::TempFs::new("root");
        let fs = Arc::new(Mutex::new(tempfs));
        
        // Mount the TempFS
        serial_println!("DEBUG: Getting VFS manager");
        let vfs = match vfs::get_vfs_manager() {
            Some(manager) => {
                serial_println!("DEBUG: Got VFS manager successfully");
                manager
            },
            None => {
                serial_println!("DEBUG: VFS manager not initialized");
                return Err(KernelError::NotInitialized);
            }
        };
        
        serial_println!("DEBUG: Preparing to mount TempFS at /");
        serial_println!("DEBUG: Creating filesystem Arc and Mutex");
        let tempfs_mutex = fs.clone();
        serial_println!("DEBUG: Arc and Mutex created");

        serial_println!("DEBUG: Calling vfs.mount()");
        if let Err(e) = vfs.mount("/", tempfs_mutex.clone()) {
            serial_println!("DEBUG: Failed to mount TempFS: {:?}", e);
            return Err(e);
        }

        serial_println!("DEBUG: Mount successful, TempFS mounted at /");
        
        // Store the mounted filesystem
        serial_println!("DEBUG: Storing filesystem in global variable");
        unsafe {
            GLOBAL_FS = Some(tempfs_mutex.clone());
            serial_println!("DEBUG: Global filesystem stored successfully");
        }
        
        serial_println!("DEBUG: RAM filesystem initialization complete");
    } else {
        // This branch is currently unlikely to be hit due to use_tempfs_resolved logic
        serial_println!("DEBUG: Creating FAT RamDisk filesystem");
        use crate::fs::fat::FatFileSystem;
        use crate::fs::ramdisk::RamDisk;
        
        // Create a RamDisk
        serial_println!("DEBUG: Attempting to create RamDisk for FAT");
        let ramdisk = match RamDisk::new() {
            Ok(disk) => {
                serial_println!("DEBUG: RamDisk created successfully");
                disk
            },
            Err(e) => {
                serial_println!("DEBUG: Failed to create RamDisk: {}", e);
                return Err(KernelError::from(e));
            }
        };
        
        serial_println!("DEBUG: Created RAM disk with {} blocks of size {} bytes",
            ramdisk.block_count(), ramdisk.block_size());
        
        // Create a FatFileSystem
        let fatfs = FatFileSystem::new(Arc::new(Mutex::new(ramdisk)))?;
        let fs = Arc::new(Mutex::new(fatfs));
        
        // Mount the FatFileSystem
        let vfs = vfs::get_vfs_manager().ok_or(KernelError::NotInitialized)?;
        vfs.mount("/", fs.clone())?;
        
        serial_println!("DEBUG: FAT filesystem mounted at /");
        
        // Store the mounted filesystem
        unsafe {
            GLOBAL_FS = Some(fs);
        }
    }
    
    Ok(())
}

/// Create some initial files and directories in the filesystem
fn create_initial_files() -> Result<(), KernelError> {
    let vfs = vfs::get_vfs_manager().ok_or(KernelError::NotInitialized)?;
    
    // Create /bin directory
    vfs.create_directory("/bin")?;
    
    // Create /home directory
    vfs.create_directory("/home")?;
    
    // Create /etc directory
    vfs.create_directory("/etc")?;
    
    // Create /tmp directory
    vfs.create_directory("/tmp")?;
    
    serial_println!("DEBUG: Created initial directory structure");
    
    Ok(())
}

/// Global file system instance (primary file system)
static mut GLOBAL_FS: Option<Arc<Mutex<dyn vfs::FileSystem>>> = None;

/// Get a reference to the global file system, if initialized.
pub fn get_fs() -> Option<Arc<Mutex<dyn vfs::FileSystem>>> {
    unsafe { GLOBAL_FS.as_ref().map(|fs| fs.clone()) }
}

// Re-exports for convenience
pub use block_device::BlockDevice;
pub use ramdisk::RamDisk;
pub use simple_fs::SimpleFileSystem;

impl From<&'static str> for KernelError {
    fn from(err: &'static str) -> Self {
        KernelError::GenericError(err)
    }
}

/// Test the file system functionality
fn test_filesystem() {
    serial_println!("DEBUG: Testing file system...");
    
    // Get the VFS manager
    if let Some(vfs) = vfs::get_vfs_manager() {
        // List the root directory
        match vfs.read_dir("/") {
            Ok(entries) => {
                serial_println!("DEBUG: Root directory contents:");
                for entry in entries {
                    let type_str = match entry.node_type {
                        vfs::NodeType::Directory => "DIR",
                        vfs::NodeType::File => "FILE",
                        _ => "OTHER",
                    };
                    serial_println!("DEBUG:   {} [{}]", entry.name, type_str);
                }
            },
            Err(e) => {
                serial_println!("DEBUG: Error listing root directory: {:?}", e);
            }
        }
        
        // Try to create a test file
        match vfs.create_file("/test.txt") {
            Ok(_) => {
                serial_println!("DEBUG: Created test file: /test.txt");
                
                // Open the file for writing
                match fd::open("/test.txt", vfs::file_flags::WRITE) {
                    Ok(fd) => {
                        serial_println!("DEBUG: Opened file for writing with FD: {}", fd);
                        
                        // Write to the file
                        let data = b"Hello, File System!";
                        match fd::write(fd, data) {
                            Ok(bytes) => {
                                serial_println!("DEBUG: Wrote {} bytes to file", bytes);
                                
                                // Close the file
                                if let Err(e) = fd::close(fd) {
                                    serial_println!("DEBUG: Error closing file: {:?}", e);
                                }
                                
                                // Reopen for reading
                                match fd::open("/test.txt", vfs::file_flags::READ) {
                                    Ok(fd) => {
                                        serial_println!("DEBUG: Reopened file for reading with FD: {}", fd);
                                        
                                        // Read from the file
                                        let mut buffer = [0u8; 20];
                                        match fd::read(fd, &mut buffer) {
                                            Ok(bytes) => {
                                                let text = core::str::from_utf8(&buffer[0..bytes])
                                                    .unwrap_or("(invalid UTF-8)");
                                                serial_println!("DEBUG: Read {} bytes from file: '{}'", bytes, text);
                                            },
                                            Err(e) => {
                                                serial_println!("DEBUG: Error reading from file: {:?}", e);
                                            }
                                        }
                                        
                                        // Close the file
                                        if let Err(e) = fd::close(fd) {
                                            serial_println!("DEBUG: Error closing file: {:?}", e);
                                        }
                                    },
                                    Err(e) => {
                                        serial_println!("DEBUG: Error reopening file for reading: {:?}", e);
                                    }
                                }
                            },
                            Err(e) => {
                                serial_println!("DEBUG: Error writing to file: {:?}", e);
                            }
                        }
                    },
                    Err(e) => {
                        serial_println!("DEBUG: Error opening file: {:?}", e);
                    }
                }
            },
            Err(e) => {
                serial_println!("DEBUG: Error creating test file: {:?}", e);
            }
        }
    } else {
        serial_println!("DEBUG: VFS manager not initialized");
    }
    
    serial_println!("DEBUG: File system test complete");
}

/// Directly write data to a file, bypassing the file descriptor system
/// This is a workaround for issues with the FD system
pub fn direct_write_file(path: &str, data: &[u8]) -> Result<usize, KernelError> {
    serial_println!("DEBUG: direct_write_file - Starting for path: {}", path);
    
    // Get the VFS manager
    let vfs = vfs::get_vfs_manager().ok_or(KernelError::NotInitialized)?;
    serial_println!("DEBUG: direct_write_file - Got VFS manager");
    
    // Find the filesystem that contains this path
    let fs = vfs.find_fs(path)?;
    serial_println!("DEBUG: direct_write_file - Found filesystem for path");
    
    // Lock the filesystem and write directly
    let mut fs_guard = fs.lock();
    serial_println!("DEBUG: direct_write_file - Acquired filesystem lock");
    
    // Write at position 0
    let result = fs_guard.write_at(path, 0, data);
    
    match &result {
        Ok(bytes) => serial_println!("DEBUG: direct_write_file - Successfully wrote {} bytes", bytes),
        Err(e) => serial_println!("DEBUG: direct_write_file - Error: {:?}", e),
    }
    
    result
}

/// Directly read data from a file, bypassing the file descriptor system
pub fn direct_read_file(path: &str, buffer: &mut [u8]) -> Result<usize, KernelError> {
    serial_println!("DEBUG: direct_read_file - Starting for path: {}", path);
    
    // Get the VFS manager
    let vfs = vfs::get_vfs_manager().ok_or(KernelError::NotInitialized)?;
    serial_println!("DEBUG: direct_read_file - Got VFS manager");
    
    // Find the filesystem that contains this path
    let fs = vfs.find_fs(path)?;
    serial_println!("DEBUG: direct_read_file - Found filesystem for path");
    
    // Lock the filesystem and read directly
    let fs_guard = fs.lock();
    serial_println!("DEBUG: direct_read_file - Acquired filesystem lock");
    
    // Read from position 0
    let result = fs_guard.read_at(path, 0, buffer);
    
    match &result {
        Ok(bytes) => serial_println!("DEBUG: direct_read_file - Successfully read {} bytes", bytes),
        Err(e) => serial_println!("DEBUG: direct_read_file - Error: {:?}", e),
    }
    
    result
}

/// Directly create a directory, bypassing the VFS layer
/// SAFETY: Only for use during initial filesystem setup
pub fn direct_create_directory(path: &str) -> Result<(), KernelError> {
    serial_println!("DEBUG: fs::direct_create_directory - Starting for path: {}", path);
    
    // Find the global filesystem
    let fs_opt = unsafe { GLOBAL_FS.as_ref() };
    let fs = match fs_opt {
        Some(fs) => fs,
        None => {
            serial_println!("DEBUG: fs::direct_create_directory - Global FS not found");
            return Err(KernelError::NotInitialized);
        }
    };
    
    // Check if it's a TempFS and use the direct method if available
    let mut fs_guard = fs.lock();
    
    if fs_guard.is_tempfs() {
        serial_println!("DEBUG: fs::direct_create_directory - Found TempFS, using direct creation");
        // Convert to TempFS
        if let Some(tempfs) = tempfs::as_tempfs(&mut *fs_guard) {
            return tempfs.direct_create_directory(path);
        }
    }
    
    // Fallback to standard directory creation
    serial_println!("DEBUG: fs::direct_create_directory - Using standard creation");
    fs_guard.create_directory(path)
}