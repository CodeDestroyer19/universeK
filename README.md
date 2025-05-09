# UniverseK OS

A lightweight, bare-metal operating system with a modular kernel architecture, GUI subsystem, and comprehensive debugging capabilities.

## Project Overview

UniverseK OS is built from scratch with a focus on:

- Modular, efficient kernel architecture
- Windows-like GUI subsystem with applications
- Robust error handling and detection
- Comprehensive debugging infrastructure
- File system support and user management

## Key Features

- **Bare-Metal OS**: Runs directly on x86_64 hardware without requiring any host OS
- **Rust Implementation**: Built primarily in Rust for memory safety and modern language features
- **Graphical User Interface**: Windows-like desktop environment with:
  - Taskbar with START button
  - Desktop icons and clickable applications
  - Window management (dragging, resizing, minimizing)
  - Multiple pre-installed applications
- **File System**: In-memory and disk file systems with standard operations
- **Memory Management**: Paging, heap allocation, and memory protection
- **Device Drivers**: Support for keyboard, mouse, timers, and more
- **Multi-tasking**: Basic task scheduling and management

## Directory Structure

```
/
├── boot/              # Bootloader code
├── kernel/            # Kernel source code
│   ├── src/           # Main kernel codebase
│   │   ├── drivers/   # Hardware drivers
│   │   ├── fs/        # File system implementations
│   │   ├── gui/       # GUI subsystem
│   │   ├── memory/    # Memory management
│   │   └── ...        # Other kernel modules
├── tools/             # Development tools
├── docs/              # Documentation
└── target/            # Build outputs
```

## Building

### Prerequisites

- Rust toolchain (nightly)
- QEMU for emulation
- `cargo-xbuild` or `bootimage` for cross-compilation
- Optional: GDB for debugging

### Setup

1. Install the Rust toolchain and required tools:

```bash
# Install Rust with rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Install nightly toolchain
rustup toolchain install nightly
# Set as default
rustup default nightly
# Install required components
rustup component add rust-src llvm-tools-preview
# Install bootimage
cargo install bootimage
```

2. Install QEMU:

```bash
# Ubuntu/Debian
sudo apt-get install qemu-system-x86
# macOS
brew install qemu
```

3. Build UniverseK OS:

```bash
# In the project root directory
cargo build
```

4. Create a bootable image:

```bash
cargo bootimage
```

### Running

To run UniverseK OS in QEMU:

```bash
cargo run
```

To debug with GDB:

```bash
# Terminal 1
qemu-system-x86_64 -drive format=raw,file=target/x86_64-bear_os/debug/bootimage-kernel.bin -s -S
# Terminal 2
gdb -ex "target remote localhost:1234" -ex "symbol-file target/x86_64-bear_os/debug/kernel"
```

## GUI Subsystem

UniverseK OS features a Windows-like GUI system with:

### Desktop Environment

- Desktop with customizable background
- Icons for launching applications
- Taskbar with start button and system information

### Window Management

- Movable and resizable windows
- Title bars with minimize, maximize, and close buttons
- Focus management between multiple windows

### Applications

- **Terminal**: Command-line interface with shell functionality
- **About**: System information display
- **Files**: Basic file explorer for the filesystem
- **Settings**: System configuration interface

## Known Issues

- **Hardware Interrupt Handling**: The system is currently running in a limited "safe mode" with some hardware interrupts disabled to ensure stability.
- **GUI Limitations**: The GUI is still in development with some features not fully implemented or optimized.
- **File System Limitations**: The file system is primarily in-memory with limited persistence.

## Recent Fixes

- Fixed GUI subsystem compilation issues related to the window management
- Resolved borrowing conflicts in event handling for GUI components
- Fixed MouseButton import errors and window mutability issues
- Corrected mouse driver implementation with proper delimiter closure and overflow handling

## Contributing

We welcome contributions to UniverseK OS! Here's how to get started:

1. Fork the repository
2. Create a feature branch (`git checkout -b my-new-feature`)
3. Commit your changes (`git commit -am 'Add some feature'`)
4. Push to the branch (`git push origin my-new-feature`)
5. Create a new Pull Request

Please make sure to update tests and documentation as appropriate.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
