# UniverseK OS - Known Issues

This document tracks known issues and limitations in the current implementation of UniverseK OS.

## Core System Issues

### Filesystem Issues

- **Directory Creation Hangs**: The filesystem setup process can hang when creating nested directories, particularly when creating `/Library/Preferences`. The current workaround is to bypass the filesystem setup during boot.
- **Path Normalization Problems**: The path normalization in TempFS had issues with handling trailing slashes and multiple sequential slashes.
- **Error Handling in Filesystem**: Directory creation errors were causing the entire boot process to fail instead of continuing gracefully.

### Hardware Support

- **Safe Mode Required**: The kernel currently operates in "safe mode" with hardware interrupts disabled due to issues with interrupt handling causing system hangs.
- **PS/2 Keyboard and Mouse**: Drivers exist but are not fully functional due to safe mode operation.
- **PIT Timer**: The Programmable Interval Timer is configured but not actively used for timing due to interrupt issues.

### User Management

- **Limited User Authentication**: No password or authentication mechanism is implemented yet.
- **User Setup**: The first-time user setup process is minimal and doesn't offer full customization.
- **User Permissions**: No file permissions or access control for users.

## Interface Issues

### UI/UX Limitations

- **Text-Only Interface**: No graphical user interface beyond VGA text mode with basic colors.
- **Limited Input Handling**: Keyboard input handling is primitive with no support for complex key combinations.
- **No Window Management**: No windowing system or desktop environment.

### Shell/Terminal

- **No Command Line**: No interactive shell or terminal for user input.
- **Missing Utilities**: No basic shell commands or utilities.

## Development and Debugging

### Memory Management

- **No Memory Protection**: Limited protection against invalid memory access.
- **Heap Management**: Basic heap allocation but no garbage collection or memory leak detection.

### Debugging

- **Limited Diagnostics**: Debug information is primarily sent to serial port output.
- **No Core Dumps**: No support for crash dumps or state preservation on errors.

## Compatibility Issues

### Standards Compliance

- **Limited POSIX Compliance**: Not compatible with POSIX standards.
- **No System Calls**: No standardized system call interface for applications.

### Hardware Support

- **Limited Device Support**: Only basic hardware devices are supported.
- **No USB Support**: No drivers for USB devices.
- **No Graphics Acceleration**: Limited to basic VGA capabilities.

## Workarounds

1. **Filesystem Setup Bypass**: The filesystem setup code is currently bypassed in `lib.rs` to prevent hangs during boot.
2. **Safe Mode Operation**: The kernel operates with interrupts disabled to avoid crashes.
3. **Polling-Based Main Loop**: Using a busy-waiting loop rather than interrupt-driven events.

## Roadmap for Resolution

1. **Short Term**

   - Implement proper error handling in filesystem operations
   - Debug and fix the directory creation issues
   - Implement basic shell/terminal functionality

2. **Medium Term**

   - Fix interrupt handling issues to disable safe mode
   - Implement proper user authentication
   - Add file permissions system

3. **Long Term**
   - Add window management capabilities
   - Implement a graphical user interface
   - Add driver support for more hardware devices
