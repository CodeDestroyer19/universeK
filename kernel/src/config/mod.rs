//! Configuration management for UniverseK OS
//! Handles system configuration settings and boot options

use alloc::string::{String, ToString};
use alloc::collections::BTreeMap;
use alloc::format;
use spin::Mutex;
use lazy_static::lazy_static;
use crate::serial_println;
use crate::fs;
use crate::errors::KernelError;

/// A single configuration value
#[derive(Debug, Clone)]
pub enum ConfigValue {
    /// String value
    String(String),
    /// Integer value
    Integer(i64),
    /// Boolean value
    Boolean(bool),
}

impl ConfigValue {
    /// Create a string value
    pub fn string(value: &str) -> Self {
        ConfigValue::String(value.to_string())
    }
    
    /// Create an integer value
    pub fn integer(value: i64) -> Self {
        ConfigValue::Integer(value)
    }
    
    /// Create a boolean value
    pub fn boolean(value: bool) -> Self {
        ConfigValue::Boolean(value)
    }
    
    /// Convert to string
    pub fn as_string(&self) -> String {
        match self {
            ConfigValue::String(s) => s.clone(),
            ConfigValue::Integer(i) => i.to_string(),
            ConfigValue::Boolean(b) => b.to_string(),
        }
    }
    
    /// Try to get as string
    pub fn try_as_string(&self) -> Option<&String> {
        match self {
            ConfigValue::String(s) => Some(s),
            _ => None,
        }
    }
    
    /// Try to get as integer
    pub fn try_as_integer(&self) -> Option<i64> {
        match self {
            ConfigValue::Integer(i) => Some(*i),
            _ => None,
        }
    }
    
    /// Try to get as boolean
    pub fn try_as_boolean(&self) -> Option<bool> {
        match self {
            ConfigValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

/// Configuration manager
pub struct ConfigManager {
    /// Configuration values
    values: BTreeMap<String, ConfigValue>,
    /// Whether configuration has been modified
    modified: bool,
    /// Path to the config file
    config_file: String,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Self {
        Self {
            values: BTreeMap::new(),
            modified: false,
            config_file: "/System/Library/config.ini".to_string(),
        }
    }
    
    /// Get a configuration value
    pub fn get(&self, key: &str) -> Option<&ConfigValue> {
        self.values.get(key)
    }
    
    /// Set a configuration value
    pub fn set(&mut self, key: &str, value: ConfigValue) {
        self.values.insert(key.to_string(), value);
        self.modified = true;
    }
    
    /// Remove a configuration value
    pub fn remove(&mut self, key: &str) -> Option<ConfigValue> {
        let value = self.values.remove(key);
        if value.is_some() {
            self.modified = true;
        }
        value
    }
    
    /// Load configuration from the default file
    pub fn load(&mut self) -> Result<(), KernelError> {
        let config_file = self.config_file.clone();
        self.load_from_file(&config_file)
    }
    
    /// Load configuration from a file
    pub fn load_from_file(&mut self, path: &str) -> Result<(), KernelError> {
        // Check if file exists
        let vfs = fs::vfs::get_vfs_manager().ok_or(KernelError::NotInitialized)?;
        
        // Try to get metadata (to check if file exists)
        if let Err(KernelError::NotFound) = vfs.metadata(path) {
            // File doesn't exist, use defaults
            serial_println!("Config file not found, using defaults");
            self.set_defaults();
            return Ok(());
        }
        
        // Read the file
        let mut buffer = [0u8; 1024]; // Limit config file size to 1KB
        let bytes_read = fs::direct_read_file(path, &mut buffer)?;
        
        if bytes_read == 0 {
            // Empty file, use defaults
            serial_println!("Config file is empty, using defaults");
            self.set_defaults();
            return Ok(());
        }
        
        // Parse the file content
        let content = core::str::from_utf8(&buffer[0..bytes_read])
            .map_err(|_| KernelError::InvalidData)?;
        
        // Clear existing configuration
        self.values.clear();
        
        // Parse lines
        for line in content.lines() {
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Parse key=value
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim();
                let value = line[pos+1..].trim();
                
                if key.is_empty() {
                    continue;
                }
                
                // Parse value
                if value.eq_ignore_ascii_case("true") {
                    self.set(key, ConfigValue::boolean(true));
                } else if value.eq_ignore_ascii_case("false") {
                    self.set(key, ConfigValue::boolean(false));
                } else if let Ok(int_value) = value.parse::<i64>() {
                    self.set(key, ConfigValue::integer(int_value));
                } else {
                    self.set(key, ConfigValue::string(value));
                }
            }
        }
        
        self.modified = false;
        Ok(())
    }
    
    /// Save configuration to the default file
    pub fn save(&mut self) -> Result<(), KernelError> {
        let config_file = self.config_file.clone();
        self.save_to_file(&config_file)
    }
    
    /// Save configuration to a file
    pub fn save_to_file(&mut self, path: &str) -> Result<(), KernelError> {
        let vfs = fs::vfs::get_vfs_manager().ok_or(KernelError::NotInitialized)?;
        
        // Try to create the file (or truncate if exists)
        if let Err(KernelError::AlreadyExists) = vfs.create_file(path) {
            // File already exists, remove and recreate
            vfs.remove(path)?;
            vfs.create_file(path)?;
        }
        
        // Build the file content
        let mut content = String::new();
        content.push_str("# UniverseK OS Configuration\n");
        content.push_str("# Auto-generated - do not edit manually\n\n");
        
        // Add all values
        for (key, value) in &self.values {
            match value {
                ConfigValue::String(s) => content.push_str(&format!("{}={}\n", key, s)),
                ConfigValue::Integer(i) => content.push_str(&format!("{}={}\n", key, i)),
                ConfigValue::Boolean(b) => content.push_str(&format!("{}={}\n", key, b)),
            }
        }
        
        // Write to file
        let bytes = content.as_bytes();
        fs::direct_write_file(path, bytes)?;
        
        self.modified = false;
        Ok(())
    }
    
    /// Set default configuration values
    pub fn set_defaults(&mut self) {
        // System settings
        self.set("system.name", ConfigValue::string("UniverseK OS"));
        self.set("system.version", ConfigValue::string("0.1.0"));
        self.set("system.safe_mode", ConfigValue::boolean(true));
        
        // UI settings
        self.set("ui.theme", ConfigValue::string("default"));
        self.set("ui.color_scheme", ConfigValue::string("blue"));
        
        // Filesystem settings
        self.set("fs.root_device", ConfigValue::string("ramdisk"));
        self.set("fs.automount", ConfigValue::boolean(true));
        
        // Network settings (for future use)
        self.set("network.enabled", ConfigValue::boolean(false));
        self.set("network.dhcp", ConfigValue::boolean(true));
        
        // User settings
        self.set("user.auto_login", ConfigValue::boolean(false));
        self.set("user.default", ConfigValue::string("user"));
        
        self.modified = true;
    }
    
    /// Check if a specific boot option is enabled
    pub fn is_boot_option_enabled(&self, option: &str) -> bool {
        let key = format!("boot.{}", option);
        match self.get(&key) {
            Some(ConfigValue::Boolean(b)) => *b,
            _ => false,
        }
    }
    
    /// Set a boot option
    pub fn set_boot_option(&mut self, option: &str, enabled: bool) {
        let key = format!("boot.{}", option);
        self.set(&key, ConfigValue::boolean(enabled));
    }
}

// Global configuration manager
lazy_static! {
    static ref CONFIG: Mutex<ConfigManager> = Mutex::new(ConfigManager::new());
}

/// Initialize the configuration system
pub fn init() -> Result<(), KernelError> {
    serial_println!("Initializing configuration system");
    
    let mut config = CONFIG.lock();
    
    // Attempt to load configuration from file
    serial_println!("Attempting to load configuration from {}", config.config_file);
    if let Err(e) = config.load() {
        serial_println!("Warning: Failed to load config file: {:?}. Using defaults.", e);
        // Fallback to defaults if loading fails (e.g., file not found, parse error)
        config.set_defaults();
    } else {
        serial_println!("Configuration loaded successfully from file.");
    }
    
    serial_println!("Configuration system initialized.");
    Ok(())
}

/// Get a configuration value
pub fn get(key: &str) -> Option<ConfigValue> {
    CONFIG.lock().get(key).cloned()
}

/// Set a configuration value
pub fn set(key: &str, value: ConfigValue) {
    CONFIG.lock().set(key, value);
}

/// Save configuration changes
pub fn save() -> Result<(), KernelError> {
    CONFIG.lock().save()
}

/// Check if a specific boot option is enabled
pub fn is_boot_option_enabled(option: &str) -> bool {
    CONFIG.lock().is_boot_option_enabled(option)
}

/// Set a boot option
pub fn set_boot_option(option: &str, enabled: bool) {
    CONFIG.lock().set_boot_option(option, enabled);
} 