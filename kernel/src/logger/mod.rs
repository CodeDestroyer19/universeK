//! Logging system for UniverseK OS
//! Provides different log levels and output targets

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use core::fmt;
use spin::Mutex;
use lazy_static::lazy_static;
use crate::serial_println;
use crate::drivers::vga_enhanced::{self, Color};

/// Log levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl LogLevel {
    /// Convert log level to string
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARNING",
            LogLevel::Error => "ERROR",
            LogLevel::Critical => "CRITICAL",
        }
    }
    
    /// Get color for log level
    pub fn color(&self) -> Color {
        match self {
            LogLevel::Debug => Color::LightCyan,
            LogLevel::Info => Color::White,
            LogLevel::Warning => Color::Yellow,
            LogLevel::Error => Color::LightRed,
            LogLevel::Critical => Color::Red,
        }
    }
}

/// Log target
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogTarget {
    Serial,
    Screen,
    Both,
    Memory,
}

/// Log entry
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Log level
    pub level: LogLevel,
    /// Module name
    pub module: String,
    /// Log message
    pub message: String,
    /// Timestamp (seconds since boot)
    pub timestamp: u64,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(level: LogLevel, module: &str, message: &str) -> Self {
        // Get timestamp (for now just use a placeholder)
        // In a real implementation, we would get this from a timer
        let timestamp = 0; // Placeholder
        
        Self {
            level,
            module: module.to_string(),
            message: message.to_string(),
            timestamp,
        }
    }
    
    /// Format the log entry
    pub fn format(&self) -> String {
        format!("[{:04}.{:03}] {:<8} {}: {}", 
            self.timestamp / 1000, 
            self.timestamp % 1000, 
            self.level.as_str(), 
            self.module, 
            self.message)
    }
}

/// Logger state
pub struct Logger {
    /// Minimum log level to display
    min_level: LogLevel,
    /// Log target
    target: LogTarget,
    /// In-memory log buffer (for viewing later)
    log_buffer: Vec<LogEntry>,
    /// Maximum log buffer size
    max_buffer_size: usize,
}

impl Logger {
    /// Create a new logger
    pub fn new() -> Self {
        Self {
            min_level: LogLevel::Info,
            target: LogTarget::Both,
            log_buffer: Vec::new(),
            max_buffer_size: 1000,
        }
    }
    
    /// Set minimum log level
    pub fn set_min_level(&mut self, level: LogLevel) {
        self.min_level = level;
    }
    
    /// Set log target
    pub fn set_target(&mut self, target: LogTarget) {
        self.target = target;
    }
    
    /// Log a message
    pub fn log(&mut self, level: LogLevel, module: &str, message: &str) {
        // Skip if below minimum level
        let should_skip = match (level, self.min_level) {
            (LogLevel::Debug, LogLevel::Info) => true,
            (LogLevel::Debug, LogLevel::Warning) => true,
            (LogLevel::Debug, LogLevel::Error) => true,
            (LogLevel::Debug, LogLevel::Critical) => true,
            
            (LogLevel::Info, LogLevel::Warning) => true,
            (LogLevel::Info, LogLevel::Error) => true,
            (LogLevel::Info, LogLevel::Critical) => true,
            
            (LogLevel::Warning, LogLevel::Error) => true,
            (LogLevel::Warning, LogLevel::Critical) => true,
            
            (LogLevel::Error, LogLevel::Critical) => true,
            
            _ => false,
        };
        
        if should_skip {
            return;
        }
        
        // Create log entry
        let entry = LogEntry::new(level, module, message);
        let formatted = entry.format();
        
        // Output to selected targets
        match self.target {
            LogTarget::Serial => {
                serial_println!("{}", formatted);
            },
            LogTarget::Screen => {
                self.log_to_screen(&entry);
            },
            LogTarget::Both => {
                serial_println!("{}", formatted);
                self.log_to_screen(&entry);
            },
            LogTarget::Memory => {
                // Just store in buffer
            },
        }
        
        // Store in memory buffer
        self.log_buffer.push(entry);
        
        // Trim buffer if needed
        if self.log_buffer.len() > self.max_buffer_size {
            self.log_buffer.remove(0);
        }
    }
    
    /// Output to screen
    fn log_to_screen(&self, entry: &LogEntry) {
        // For now, just write to bottom of screen
        // In a real implementation, this would scroll a log area
        
        // Clear the log line area
        for i in 0..80 {
            vga_enhanced::write_at(24, i, " ", Color::White, Color::Black);
        }
        
        // Write the log message, trimmed to fit on one line
        let mut msg = entry.format();
        if msg.len() > 79 {
            msg.truncate(76);
            msg.push_str("...");
        }
        
        vga_enhanced::write_at(24, 0, &msg, entry.level.color(), Color::Black);
    }
    
    /// Get all log entries
    pub fn get_entries(&self) -> &[LogEntry] {
        &self.log_buffer
    }
    
    /// Clear the log buffer
    pub fn clear(&mut self) {
        self.log_buffer.clear();
    }
}

// Global logger instance
lazy_static! {
    static ref LOGGER: Mutex<Logger> = Mutex::new(Logger::new());
}

/// Initialize the logging system
pub fn init() -> Result<(), crate::errors::KernelError> {
    serial_println!("Initializing logging system");
    
    // No special initialization needed
    
    Ok(())
}

/// Log a debug message
pub fn debug(module: &str, message: &str) {
    LOGGER.lock().log(LogLevel::Debug, module, message);
}

/// Log an info message
pub fn info(module: &str, message: &str) {
    LOGGER.lock().log(LogLevel::Info, module, message);
}

/// Log a warning message
pub fn warning(module: &str, message: &str) {
    LOGGER.lock().log(LogLevel::Warning, module, message);
}

/// Log an error message
pub fn error(module: &str, message: &str) {
    LOGGER.lock().log(LogLevel::Error, module, message);
}

/// Log a critical message
pub fn critical(module: &str, message: &str) {
    LOGGER.lock().log(LogLevel::Critical, module, message);
}

// Implement for format! support
impl fmt::Write for Logger {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.log(LogLevel::Info, "fmt", s);
        Ok(())
    }
} 