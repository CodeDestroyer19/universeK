# UniverseK OS - Utility Features

This document describes the utility features implemented in UniverseK OS.

## Shell / Terminal

The shell module (`kernel/src/shell/mod.rs`) provides a command-line interface for interacting with the system:

### Features

- **Interactive Command Line**: Supports typing commands and viewing output
- **Command History**: Navigate through previous commands with arrow keys
- **Built-in Commands**:
  - `help` - Display available commands
  - `echo [msg]` - Display a message
  - `ls [dir]` - List directory contents
  - `cd [dir]` - Change directory
  - `pwd` - Print working directory
  - `cat [file]` - Display file contents
  - `clear/cls` - Clear the screen
  - `touch [file]` - Create a new file
  - `mkdir [dir]` - Create a new directory
  - `rm [path]` - Remove a file or directory
  - `reboot` - Restart the system
  - `version` - Display OS version

### Implementation

The shell is implemented using a polling-based input mechanism since the kernel runs in "safe mode" without hardware interrupts. It provides a command parser and execution framework that could be extended with additional commands in the future.

## Debug Console / Logger

The logger module (`kernel/src/logger/mod.rs`) provides a structured logging system:

### Features

- **Multiple Log Levels**: Debug, Info, Warning, Error, Critical
- **Multiple Output Targets**: Serial port, screen, in-memory buffer
- **Formatted Output**: Includes timestamp, log level, and module name
- **In-Memory Buffer**: Stores recent log messages for later viewing

### Implementation

The logger is implemented as a global singleton that can be accessed from anywhere in the kernel. It provides simple functions for logging at different levels and supports outputting to different targets simultaneously.

## Error Handling and Recovery

Error handling is implemented across the kernel with the `KernelError` type in `kernel/src/errors.rs`:

### Features

- **Structured Error Types**: Different error variants for different failure modes
- **Error Propagation**: Uses Rust's `?` operator for concise error handling
- **Error Reporting**: Logs errors to the console and/or serial port
- **Panic Screen**: Displays helpful information when the kernel panics

### Implementation

The error handling system is designed to allow components to fail gracefully when possible. The kernel attempts to continue operating even if some subsystems fail to initialize.

## Configuration Management

The config module (`kernel/src/config/mod.rs`) provides a system-wide configuration system:

### Features

- **Typed Configuration Values**: String, Integer, Boolean
- **Configuration File**: Loads and saves configuration from a file
- **Default Settings**: Provides reasonable defaults if configuration is missing
- **Boot Options**: Specific configuration for boot-time settings

### Implementation

The configuration system loads settings from a file at `/System/Library/config.ini` and provides a simple key-value store for system configuration. Settings are automatically saved when changed and persist across reboots.

## Usage Examples

### Using the Shell

The shell is automatically launched after booting:

```
user:/$ ls
System/ Library/ Applications/ Users/ root/ tmp/ welcome.txt
user:/$ cd Users
user:/Users$ mkdir test_user
user:/Users$ ls
test_user/
user:/Users$ cd test_user
user:/Users/test_user$ touch hello.txt
user:/Users/test_user$ cat hello.txt
(empty file)
```

### Using the Logger

Logging can be done from any part of the kernel:

```rust
// Log at different levels
logger::debug("mymodule", "This is a debug message");
logger::info("mymodule", "System started successfully");
logger::warning("mymodule", "Disk space is running low");
logger::error("mymodule", "Failed to initialize device");
logger::critical("mymodule", "Kernel panic: out of memory");
```

### Using the Configuration System

Configuration values can be accessed and modified:

```rust
// Get a configuration value
if let Some(value) = config::get("system.name") {
    println!("System name: {}", value.as_string());
}

// Set a configuration value
config::set("ui.theme", ConfigValue::string("dark"));

// Save configuration changes
config::save()?;

// Check if a boot option is enabled
if config::is_boot_option_enabled("safe_mode") {
    println!("Running in safe mode");
}
```

## Future Improvements

### Shell / Terminal

- Add tab completion for commands and file paths
- Implement proper scrolling for command output
- Add support for pipe operators and redirects
- Add scripting support with variables and control structures

### Logger

- Add support for log categories and filtering
- Implement log rotation for in-memory buffer
- Add a log viewer command to the shell
- Support logging to a file on disk

### Error Handling

- Implement better crash recovery mechanisms
- Add a crash dump system to preserve error information
- Implement more specific error types for different subsystems

### Configuration

- Add support for configuration categories
- Implement user-specific configuration settings
- Support for overriding configuration via boot parameters
- Implement configuration change notifications
