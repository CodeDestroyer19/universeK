use crate::errors::KernelError;
use crate::fs::vfs::FileHandle;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, Ordering};
use spin::Mutex;
use crate::serial_println;

/// Unique file descriptor counter
static NEXT_FD: AtomicU32 = AtomicU32::new(3); // Start at 3 (after stdin, stdout, stderr)

/// A file descriptor is a simple handle to an open file
pub struct FileDescriptor {
    pub fd: u32,
    pub handle: Box<FileHandle>,
}
impl FileDescriptor {
    fn new(handle: FileHandle) -> Self {
        // Avoid printing handle properties directly
        serial_println!("DEBUG: FileDescriptor::new - Creating new FD");

        // Get next available FD number
        let fd = NEXT_FD.fetch_add(1, Ordering::Relaxed);
        serial_println!("DEBUG: FileDescriptor::new - Got FD: {}", fd);

        // Create a local copy of flags before boxing, to avoid accessing handle after boxing
        let flags = handle.flags;
        
        // Try to skip the Box altogether
        return Self {
            fd,
            handle: Box::new(handle),
        };
    }
}

/// File descriptor table for managing open files
pub struct FdTable {
    descriptors: Vec<FileDescriptor>,
}

impl FdTable {
    pub fn new() -> Self {
        Self {
            descriptors: Vec::new(),
        }
    }
    
    /// Open a file and return a file descriptor
    pub fn open(&mut self, path: &str, flags: u8) -> Result<u32, KernelError> {
        serial_println!("DEBUG: FdTable::open - Starting for path '{}', flags={}", path, flags);
        
        // Get the VFS manager
        let vfs_manager = match crate::fs::vfs::get_vfs_manager() {
            Some(manager) => {
                serial_println!("DEBUG: FdTable::open - Got VFS manager");
                manager
            },
            None => {
                serial_println!("DEBUG: FdTable::open - ERROR: VFS manager not initialized");
                return Err(KernelError::NotInitialized);
            }
        };
        
        // Open the file
        serial_println!("DEBUG: FdTable::open - Calling vfs_manager.open() for path '{}'", path);
        let handle = match vfs_manager.open(path, flags) {
            Ok(h) => {
                serial_println!("DEBUG: FdTable::open - vfs_manager.open() successful for path '{}'", h.path);
                h
            },
            Err(e) => {
                serial_println!("DEBUG: FdTable::open - vfs_manager.open() FAILED for path '{}': {:?}", path, e);
                return Err(e);
            }
        };
        
        // Create a file descriptor
        let fd_entry;
        serial_println!("DEBUG: FdTable::open - Calling FileDescriptor::new() for path '{}'", handle.path);
        fd_entry = FileDescriptor::new(handle);
        serial_println!("DEBUG: FdTable::open - FileDescriptor::new() successful, fd={}", fd_entry.fd);
        let fd = fd_entry.fd;
        
        // Add to the table
        serial_println!("DEBUG: FdTable::open - Pushing fd_entry (fd={}) to descriptors vector", fd);
        self.descriptors.push(fd_entry);
        serial_println!("DEBUG: FdTable::open - Push successful. Current descriptor count: {}", self.descriptors.len());
        
        serial_println!("DEBUG: FdTable::open - Returning Ok(fd={}) for path '{}'", fd, path);
        Ok(fd)
    }
    
    /// Close a file descriptor
    pub fn close(&mut self, fd: u32) -> Result<(), KernelError> {
        let index = self.descriptors.iter()
            .position(|desc| desc.fd == fd)
            .ok_or(KernelError::InvalidHandle)?;
        
        // Remove the descriptor from the table
        let mut fd_entry = self.descriptors.remove(index);
        
        // Close the file handle
        fd_entry.handle.close()
    }
    
    /// Read from a file descriptor
    pub fn read(&mut self, fd: u32, buffer: &mut [u8]) -> Result<usize, KernelError> {
        let fd_entry = self.get_fd_mut(fd)?;
        fd_entry.handle.read(buffer)
    }
    
    /// Write to a file descriptor
    pub fn write(&mut self, fd: u32, buffer: &[u8]) -> Result<usize, KernelError> {
        use crate::serial_println;
        serial_println!("DEBUG: FdTable::write - Starting with fd={}, buffer.len={}", fd, buffer.len());
        
        // Get the file descriptor entry
        let fd_entry = match self.get_fd_mut(fd) {
            Ok(entry) => {
                serial_println!("DEBUG: FdTable::write - Found FD entry");
                entry
            },
            Err(e) => {
                serial_println!("DEBUG: FdTable::write - FD not found: {:?}", e);
                return Err(e);
            }
        };
        
        serial_println!("DEBUG: FdTable::write - Calling handle.write()");
        let result = fd_entry.handle.write(buffer);
        
        match &result {
            Ok(bytes) => serial_println!("DEBUG: FdTable::write - Completed, wrote {} bytes", bytes),
            Err(e) => serial_println!("DEBUG: FdTable::write - Error: {:?}", e),
        }
        
        result
    }
    
    /// Seek to a position in a file
    pub fn seek(&mut self, fd: u32, position: u64) -> Result<(), KernelError> {
        let fd_entry = self.get_fd_mut(fd)?;
        fd_entry.handle.seek(position)
    }
    
    /// Get the current position in a file
    pub fn tell(&self, fd: u32) -> Result<u64, KernelError> {
        let fd_entry = self.get_fd(fd)?;
        Ok(fd_entry.handle.position)
    }
    
    /// Get a file descriptor entry
    fn get_fd(&self, fd: u32) -> Result<&FileDescriptor, KernelError> {
        self.descriptors.iter()
            .find(|desc| desc.fd == fd)
            .ok_or(KernelError::InvalidHandle)
    }
    
    /// Get a mutable file descriptor entry
    fn get_fd_mut(&mut self, fd: u32) -> Result<&mut FileDescriptor, KernelError> {
        self.descriptors.iter_mut()
            .find(|desc| desc.fd == fd)
            .ok_or(KernelError::InvalidHandle)
    }
}

/// Global file descriptor table - initialized in init()
static mut FD_TABLE: Option<Mutex<FdTable>> = None;

/// Initialize standard file descriptors (stdin, stdout, stderr)
pub fn init() -> Result<(), KernelError> {
    serial_println!("DEBUG: fd::init - Initializing file descriptor system");
    
    // Initialize the FD table safely
    unsafe {
        if FD_TABLE.is_none() {
            FD_TABLE = Some(Mutex::new(FdTable::new()));
            serial_println!("DEBUG: fd::init - Created new FdTable");
        }
    }
    
    serial_println!("DEBUG: fd::init - File descriptor system initialized");
    // In a real implementation, we would set up stdin, stdout, and stderr
    Ok(())
}

/// Get the global FD table or initialize it if needed
fn get_fd_table() -> &'static Mutex<FdTable> {
    unsafe {
        if let Some(table) = &FD_TABLE {
            table
        } else {
            serial_println!("DEBUG: get_fd_table - FD_TABLE not initialized, creating it now");
            FD_TABLE = Some(Mutex::new(FdTable::new()));
            FD_TABLE.as_ref().unwrap()
        }
    }
}

/// Open a file and return a file descriptor
pub fn open(path: &str, flags: u8) -> Result<u32, KernelError> {
    serial_println!("DEBUG: fd::open - Opening file '{}'", path);
    let table = get_fd_table();
    let mut table_guard = table.lock();
    let fd = table_guard.open(path, flags)?;
    serial_println!("DEBUG: fd::open - File opened with fd={}", fd);
    Ok(fd)
}

/// Close a file descriptor
pub fn close(fd: u32) -> Result<(), KernelError> {
    serial_println!("DEBUG: fd::close - Closing fd={}", fd);
    let table = get_fd_table();
    let mut table_guard = table.lock();
    table_guard.close(fd)
}

/// Read from a file descriptor
pub fn read(fd: u32, buffer: &mut [u8]) -> Result<usize, KernelError> {
    serial_println!("DEBUG: fd::read - Reading from fd={}", fd);
    let table = get_fd_table();
    let mut table_guard = table.lock();
    table_guard.read(fd, buffer)
}

/// Write to a file descriptor
pub fn write(fd: u32, buffer: &[u8]) -> Result<usize, KernelError> {
    serial_println!("DEBUG: fd::write - Starting with fd={}, buffer.len={}", fd, buffer.len());
    let table = get_fd_table();
    
    serial_println!("DEBUG: fd::write - Getting lock on fd_table");
    let mut table_guard = table.lock();
    
    serial_println!("DEBUG: fd::write - Got lock, calling table.write()");
    let result = table_guard.write(fd, buffer);
    
    match &result {
        Ok(bytes) => serial_println!("DEBUG: fd::write - Completed, wrote {} bytes", bytes),
        Err(e) => serial_println!("DEBUG: fd::write - Error: {:?}", e),
    }
    
    result
}

/// Seek to a position in a file
pub fn seek(fd: u32, position: u64) -> Result<(), KernelError> {
    serial_println!("DEBUG: fd::seek - Seeking fd={} to position {}", fd, position);
    let table = get_fd_table();
    let mut table_guard = table.lock();
    table_guard.seek(fd, position)
}

/// Get the current position in a file
pub fn tell(fd: u32) -> Result<u64, KernelError> {
    serial_println!("DEBUG: fd::tell - Getting position for fd={}", fd);
    let table = get_fd_table();
    let table_guard = table.lock();
    table_guard.tell(fd)
} 