# BearOS

A lightweight, robust operating system with minimal GUI components and comprehensive debugging capabilities.

## Project Overview

BearOS is built from scratch with a focus on:

- Modular, efficient kernel architecture
- Minimal but functional GUI subsystem
- Robust error handling and detection
- Comprehensive debugging infrastructure
- Mixed programming approach using Assembly, C, and primarily C++

## Directory Structure

```
/
├── boot/               # Bootloader code
├── kernel/            # Kernel source code
│   ├── arch/         # Architecture-specific code
│   ├── core/         # Core kernel functionality
│   ├── drivers/      # Device drivers
│   ├── fs/           # File system implementations
│   ├── include/      # Header files
│   └── mm/           # Memory management
├── lib/              # Common libraries
├── gui/              # GUI subsystem
├── tools/            # Development tools
├── docs/             # Documentation
└── userland/         # User space applications
```

## Building

### Prerequisites

- Cross-compiler toolchain (x86_64-elf-gcc)
- CMake 3.12 or higher
- QEMU for emulation
- GNU Make

### Setup

1. Install the required tools:

```bash
# Ubuntu/Debian
sudo apt-get install build-essential cmake qemu-system-x86
```

2. Build the cross-compiler (script provided in tools/build-toolchain.sh)

3. Build BearOS:

```bash
mkdir build
cd build
cmake ..
make
```

### Running

To run BearOS in QEMU:

```bash
make run
```

To debug with GDB:

```bash
make debug
```

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
