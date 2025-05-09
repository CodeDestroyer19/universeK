# Getting Started with UniverseK OS Development

This guide will help you set up your development environment, understand the UniverseK OS codebase, build and run the operating system, and start making contributions.

## Development Environment Setup

### Prerequisites

You will need the following tools installed on your system:

- **Rust toolchain (nightly)**: UniverseK OS uses Rust's unstable features that are only available in the nightly channel.
- **QEMU**: For testing and running the OS in an emulator.
- **Bootimage**: For creating bootable disk images.
- **Optional**: GDB for debugging.

### Step 1: Install Rust and Required Components

```bash
# Install rustup (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install nightly toolchain
rustup toolchain install nightly

# Set nightly as default
rustup default nightly

# Install required components
rustup component add rust-src llvm-tools-preview

# Install bootimage tool
cargo install bootimage
```

### Step 2: Install QEMU

#### For Linux (Debian/Ubuntu):

```bash
sudo apt-get update
sudo apt-get install qemu-system-x86
```

#### For macOS:

```bash
brew install qemu
```

#### For Windows:

Download and install QEMU from [https://www.qemu.org/download/#windows](https://www.qemu.org/download/#windows)

### Step 3: Clone the Repository

```bash
git clone https://github.com/yourusername/universeK.git
cd universeK
```

## Understanding the Codebase

### Project Structure

UniverseK OS follows a modular architecture with these main components:

- **boot**: Bootloader and early initialization code
- **kernel**: The core operating system
  - **src/drivers**: Hardware device drivers
  - **src/fs**: File system implementations
  - **src/gui**: Graphical user interface
  - **src/memory**: Memory management
  - **src/shell**: Command-line interface
  - **src/interrupts**: Interrupt handling
  - **src/task**: Task scheduling and management

### Key Files and Their Purpose

- **kernel/src/lib.rs**: Kernel entry point and initialization
- **kernel/src/gui/mod.rs**: GUI subsystem initialization
- **kernel/src/gui/desktop.rs**: Desktop environment implementation
- **kernel/src/gui/window.rs**: Window management
- **kernel/src/gui/app.rs**: Application framework
- **kernel/src/drivers/ps2_keyboard.rs**: Keyboard input driver
- **kernel/src/drivers/ps2_mouse.rs**: Mouse input driver
- **kernel/src/fs/mod.rs**: File system initialization

## Building and Running

### Building the OS

From the project root directory:

```bash
# Build in debug mode
cargo build

# Build in release mode
cargo build --release

# Create a bootable image
cargo bootimage
```

### Running in QEMU

```bash
# Run with default settings
cargo run

# Run with custom options
qemu-system-x86_64 -drive format=raw,file=target/x86_64-bear_os/debug/bootimage-kernel.bin
```

### Debugging with GDB

```bash
# Terminal 1: Start QEMU with GDB server
qemu-system-x86_64 -drive format=raw,file=target/x86_64-bear_os/debug/bootimage-kernel.bin -s -S

# Terminal 2: Connect GDB
gdb -ex "target remote localhost:1234" -ex "symbol-file target/x86_64-bear_os/debug/kernel"
```

Common GDB commands:

- `c` - continue execution
- `b function_name` - set breakpoint
- `n` - next line
- `s` - step into function
- `info registers` - show register values
- `x/10x $rsp` - examine memory at stack pointer

## Making Changes

### Adding or Modifying a Feature

1. **Identify the module**: Determine which part of the codebase you need to modify.
2. **Create a feature branch**: `git checkout -b my-new-feature`
3. **Make your changes**: Implement your feature or fix.
4. **Test your changes**: Build and run the OS to verify your changes work.
5. **Submit a pull request**: Push your branch and create a PR for review.

### Common Development Tasks

#### Adding a New GUI Application

1. Create a new function in `kernel/src/gui/app.rs` that returns a `WindowHandle`:

```rust
fn create_my_app() -> Result<WindowHandle, KernelError> {
    // Create window
    let window_handle = create_window("My App", 15, 5, 50, 15);

    // Set up window content
    {
        let mut window = window_handle.lock();
        window.add_text("My App Content\n");
        // Add more content or functionality
    }

    Ok(window_handle)
}
```

2. Register the app in the `register_default_apps` function:

```rust
desktop::add_icon(AppIcon::new("My App", Box::new(create_my_app)))?;
```

#### Extending File System Functionality

Implement new methods in the relevant file system implementation:

```rust
// In kernel/src/fs/tempfs.rs or similar
impl TempFs {
    pub fn my_new_function(&mut self, param: Type) -> Result<ReturnType, KernelError> {
        // Implementation
    }
}
```

## Debugging Tips

### Serial Output

The OS outputs debug information to the serial port. In QEMU, this appears in the terminal where you launched QEMU.

### Understanding Error Messages

- **Kernel panics**: Look for stack traces in the serial output.
- **Compile errors**: Rust's error messages are usually informative about what needs to be fixed.
- **Borrow checker issues**: Common in OS development. Consider using unsafe code where necessary, but be careful.

### Visual Debugging

- Set background colors to visualize different UI areas
- Use simple test patterns to verify display functionality
- Add visual indicators for different system states

## Common Issues and Solutions

### "error[E0308]: mismatched types"

This often occurs when Rust's type inference can't determine the correct type. Add explicit type annotations where needed.

### Borrow checker errors

OS development often requires sharing mutable state across components. Consider:

- Using Arc<Mutex<T>> for shared ownership with mutability
- Implementing proper accessor methods instead of direct field access
- In some cases, using unsafe code with proper documentation

### Hardware-related issues

- If the OS crashes during hardware initialization, try disabling specific hardware components to isolate the issue.
- Use QEMU's debugging options to get more information about hardware interaction.

## Best Practices

- **Documentation**: Comment your code and update relevant documentation.
- **Error Handling**: Properly propagate errors using Result types.
- **Memory Safety**: Be careful with unsafe code and document why it's necessary.
- **Testing**: Test your changes thoroughly before submitting a PR.
- **Code Style**: Follow the existing code style and formatting.

## Additional Resources

- [Rust Documentation](https://doc.rust-lang.org/book/)
- [Writing an OS in Rust](https://os.phil-opp.com/)
- [OSDev Wiki](https://wiki.osdev.org/)
- [x86_64 Crate Documentation](https://docs.rs/x86_64/)

## Getting Help

If you get stuck, reach out to the community:

- Open an issue on GitHub
- Join our Discord server (if available)
- Check existing documentation and code comments
