// kernel/src/errors.rs
//! Error handling and validation utilities for the kernel

use crate::serial_println;
use core::fmt;

/// Represents different types of kernel errors
#[derive(Debug)]
pub enum KernelError {
    MemoryError(MemoryError),
    DeviceError(DeviceError),
    FilesystemError(FilesystemError),
    TaskError(TaskError),
    ValidationError(&'static str),
    GenericError(&'static str),
    NotImplemented,
    NotInitialized,
    InvalidParameter,
    InvalidHandle,
    NotFound,
    AlreadyExists,
    NotADirectory,
    NotAFile,
    DirectoryNotEmpty,
    IoError,
    ReadError,
    WriteError,
    BufferTooSmall,
    InvalidData,
    UnsupportedFeature,
    DeviceNotFound,
    DeviceNotInitialized,
    DeviceTimeout,
    IsADirectory,
    DirectoryFull,
    InvalidOperation,
    InitializationFailed,
    OutOfMemory,
}

#[derive(Debug)]
pub enum MemoryError {
    HeapInitFailed,
    AllocationFailed,
    PageMappingFailed,
    InvalidFrameAllocation,
}

#[derive(Debug)]
pub enum DeviceError {
    InitFailed,
    NotResponding,
    Timeout,
    InvalidOperation,
    DriverNotFound,
}

#[derive(Debug)]
pub enum FilesystemError {
    MountFailed,
    FileNotFound,
    PermissionDenied,
    ReadError,
    WriteError,
    FormatError,
    DirectoryNotEmpty,
    NotADirectory,
    NotAFile,
}

#[derive(Debug)]
pub enum TaskError {
    SchedulerInitFailed,
    InvalidTaskState,
    TaskCreationFailed,
    DeadlockDetected,
}

impl KernelError {
    /// Get a string representation of the error
    pub fn to_str(&self) -> &'static str {
        match self {
            KernelError::MemoryError(_) => "Memory error",
            KernelError::DeviceError(_) => "Device error",
            KernelError::FilesystemError(_) => "Filesystem error",
            KernelError::TaskError(_) => "Task error",
            KernelError::ValidationError(_) => "Validation error",
            KernelError::GenericError(_) => "Generic error",
            KernelError::NotImplemented => "Not implemented",
            KernelError::NotInitialized => "Not initialized",
            KernelError::InvalidParameter => "Invalid parameter",
            KernelError::InvalidHandle => "Invalid handle",
            KernelError::NotFound => "Not found",
            KernelError::AlreadyExists => "Already exists",
            KernelError::NotADirectory => "Not a directory",
            KernelError::NotAFile => "Not a file",
            KernelError::DirectoryNotEmpty => "Directory not empty",
            KernelError::IoError => "I/O error",
            KernelError::ReadError => "Read error",
            KernelError::WriteError => "Write error",
            KernelError::BufferTooSmall => "Buffer too small",
            KernelError::InvalidData => "Invalid data",
            KernelError::UnsupportedFeature => "Unsupported feature",
            KernelError::DeviceNotFound => "Device not found",
            KernelError::DeviceNotInitialized => "Device not initialized",
            KernelError::DeviceTimeout => "Device timeout",
            KernelError::IsADirectory => "Is a directory",
            KernelError::DirectoryFull => "Directory full",
            KernelError::InvalidOperation => "Invalid operation",
            KernelError::InitializationFailed => "Initialization failed",
            KernelError::OutOfMemory => "Out of memory",
        }
    }
}

impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KernelError::MemoryError(e) => write!(f, "Memory Error: {:?}", e),
            KernelError::DeviceError(e) => write!(f, "Device Error: {:?}", e),
            KernelError::FilesystemError(e) => write!(f, "Filesystem Error: {:?}", e),
            KernelError::TaskError(e) => write!(f, "Task Error: {:?}", e),
            KernelError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
            KernelError::GenericError(msg) => write!(f, "Error: {}", msg),
            _other => write!(f, "{}", self.to_str()),
        }
    }
}

/// Reports an error to the system log and console
pub fn report_error(error: &KernelError, critical: bool) {
    let prefix = if critical { "CRITICAL" } else { "ERROR" };
    serial_println!("{}: {}", prefix, error);
    
    // For critical errors, we might want to panic or take more drastic action
    if critical {
        panic!("CRITICAL ERROR: {}", error);
    }
}

/// Validation functions for kernel subsystems

/// Validates memory subsystem state
pub fn validate_memory_subsystem() -> Result<(), KernelError> {
    // This would contain actual checks in a full implementation
    // For now, just a placeholder for the structure
    
    // Example check: verify heap is initialized
    if !crate::allocator::is_heap_initialized() {
        return Err(KernelError::MemoryError(MemoryError::HeapInitFailed));
    }
    
    Ok(())
}

/// Validates interrupt subsystem state
pub fn validate_interrupt_system() -> Result<(), KernelError> {
    // Example: Check if the IDT is loaded
    // This would be expanded in a real implementation
    
    // For now just return success
    Ok(())
}

/// Validates overall system state before completing initialization
pub fn perform_system_checks() -> Result<(), KernelError> {
    serial_println!("DEBUG: errors::perform_system_checks - Start");
    // Run all validation checks
    serial_println!("DEBUG: errors::perform_system_checks - Calling validate_memory_subsystem()");
    validate_memory_subsystem()?;
    serial_println!("DEBUG: errors::perform_system_checks - Calling validate_interrupt_system()");
    validate_interrupt_system()?;

    // All checks passed
    serial_println!("DEBUG: errors::perform_system_checks - End (Success)");
    Ok(())
} 