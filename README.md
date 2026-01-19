# Windows NT Unikernel

An experimental unikernel that executes Windows PE executables without modification, bridging the Windows API to bare-metal x86_64 hardware.

## Overview

This project implements a minimal unikernel that can load and execute standard Windows `.exe` binaries (PE format) without requiring Windows. It provides a shim layer that implements Windows API functions (kernel32.dll, etc.) on top of a bare-metal Rust kernel.

### Key Features

- **PE Binary Loading**: Parse and load Windows executables
- **Windows API Shim**: Implement kernel32.dll functions in Rust
- **Bare-metal Execution**: Run on x86_64 hardware via QEMU
- **No Binary Modification**: Execute unmodified Windows binaries
- **Educational**: Learn about OS internals, PE format, and unikernel architecture

## Architecture

```
┌─────────────────────────────────────┐
│     Windows PE Binary (.exe)        │
│  (Unmodified Windows Executable)    │
└──────────────┬──────────────────────┘
               │
               │ Import Address Table (IAT)
               ▼
┌─────────────────────────────────────┐
│      Windows API Shim Layer         │
│  (kernel32.dll, ntdll.dll in Rust)  │
│                                      │
│  • GetStdHandle  • HeapAlloc        │
│  • WriteFile     • ExitProcess      │
│  • ReadFile      • GetLastError     │
└──────────────┬──────────────────────┘
               │
               │ System Calls
               ▼
┌─────────────────────────────────────┐
│    Bare-metal Rust Unikernel        │
│                                      │
│  • PE Loader    • Memory Manager    │
│  • TEB/PEB      • Virtual Memory    │
│  • Scheduler    • Serial/VGA I/O    │
└──────────────┬──────────────────────┘
               │
               │ Hardware Abstraction
               ▼
┌─────────────────────────────────────┐
│        x86_64 Hardware              │
│    (Physical or QEMU VM)            │
└─────────────────────────────────────┘
```

## Project Structure

```
windows-nt-unikernel/
├── crates/
│   ├── common/          # Shared types (PE structures, Windows types)
│   ├── pe-loader/       # PE binary loader (Phase 1: userspace prototype)
│   ├── api-shim/        # Windows API implementations
│   └── kernel/          # Bare-metal unikernel (Phase 2+)
├── target-zero/         # Minimal test binary (C program)
├── ROADMAP.md           # Detailed project roadmap
├── justfile             # Build automation (just command)
└── README.md            # This file
```

## Quick Start

### Prerequisites

- **Rust**: Install from [rustup.rs](https://rustup.rs/)
- **just**: Install with `cargo install just`
- **MinGW-w64**: For building Windows test binaries
- **QEMU**: For running the kernel (Phase 2+)

### Install Dependencies

```bash
# Install all required tools
just install-deps

# Or install individually
just install-mingw    # MinGW-w64 cross-compiler
just install-qemu     # QEMU emulator
```

### Build Everything

```bash
# Build all components
just build-all

# Or build individually
just build              # Build Rust workspace
just build-target-zero  # Build test binary
just build-loader       # Build PE loader
just build-kernel       # Build kernel
```

### Run Phase 1 (Userspace Prototype)

```bash
# Run the PE loader with Target Zero binary
just run-loader
```

### Run Phase 2 (Bare-metal Kernel)

```bash
# Boot the kernel in QEMU
just run-qemu
```

## Development Phases

This project is developed in phases, with each phase building on the previous:

### ✅ Phase 0: Preparation (COMPLETE)

- Set up Rust workspace
- Create Target Zero test binary
- Configure build system

### 🚧 Phase 1: Userspace Prototype (IN PROGRESS)

- Implement PE loader in userspace (Linux)
- Parse PE headers and sections
- Map sections to memory
- Apply relocations
- Resolve imports and patch IAT

### 📋 Phase 2: Unikernel Foundation

- Port loader to bare-metal
- Implement memory management (PMM/VMM)
- Set up bootloader
- Load PE binary from initrd

### 📋 Phase 3: Windows Execution Context

- Set up GDT and segment registers
- Implement TEB/PEB structures
- Configure GS register for TEB access
- Prepare stack and registers

### 📋 Phase 4: API Implementation

- Implement kernel32.dll functions
- `GetStdHandle`, `WriteFile`, `ExitProcess`, `GetLastError`
- Jump to PE entry point
- Execute Target Zero successfully

### 📋 Phase 5: Expansion

- Heap management (`HeapAlloc`, `HeapFree`)
- Command line arguments
- Environment variables
- File I/O
- Structured Exception Handling (SEH)

See [ROADMAP.md](ROADMAP.md) for detailed progress tracking.

## Technology Stack

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| **Language** | Rust | Memory safety, no_std support, excellent tooling |
| **PE Parser** | goblin crate | Mature, multi-format support, good documentation |
| **Kernel** | Custom (phil-opp inspired) | Full control, educational value |
| **Bootloader** | bootloader crate | Simple, Rust-native |
| **Build System** | justfile | Modern, clean syntax |
| **Testing** | QEMU + cargo test | Industry standard |

## Building Target Zero

Target Zero is a minimal Windows binary used for testing:

```bash
cd target-zero
make
```

This creates `target-zero.exe` that:
1. Calls `GetStdHandle(STD_OUTPUT_HANDLE)`
2. Calls `WriteFile()` to print "Hello from Target Zero!"
3. Calls `ExitProcess(0)`

See [target-zero/README.md](target-zero/README.md) for details.

## Testing

```bash
# Run Rust tests
just test

# Check code (fast, no build)
just check

# Run clippy linter
just clippy

# Format code
just fmt
```

## Documentation

```bash
# Generate and open documentation
just doc

# View project statistics
just stats

# View project tree
just tree
```

## Available Commands

Run `just --list` to see all available commands:

```bash
just --list
```

Common commands:
- `just build` - Build all Rust crates
- `just build-all` - Build everything (Rust + Target Zero)
- `just run-loader` - Run Phase 1 PE loader
- `just run-qemu` - Run Phase 2 kernel in QEMU
- `just test` - Run tests
- `just clean` - Clean build artifacts

## Resources

### Specifications
- [Microsoft PE Format](https://learn.microsoft.com/en-us/windows/win32/debug/pe-format)
- [Windows x64 ABI](https://learn.microsoft.com/en-us/cpp/build/x64-calling-convention)

### Reference Implementations
- [ReactOS](https://github.com/reactos/reactos) - Open source Windows implementation
- [Wine](https://gitlab.winehq.org/wine/wine) - Windows API on Unix

### Learning Resources
- [Writing an OS in Rust](https://os.phil-opp.com/) - phil-opp's blog_os
- [OSDev Wiki](https://wiki.osdev.org/) - OS development resources

## License

MIT License - See [LICENSE](LICENSE) file for details.

## Contributing

This is an educational project. Contributions are welcome! Please:

1. Follow the roadmap phases
2. Write tests for new functionality
3. Document Windows API behavior
4. Keep code simple and well-commented

## Goals and Non-Goals

### Goals
- Execute specific Windows binaries on bare-metal
- Understand PE format and Windows internals
- Learn unikernel and OS development
- Provide educational value

### Non-Goals
- Full Windows compatibility
- Running arbitrary Windows software
- Production use
- Security hardening
- Performance optimization (initially)

## Status

🚧 **Early Development** - Phase 0 complete, Phase 1 in progress.

The project can currently:
- ✅ Build a minimal Windows test binary
- ✅ Set up project structure
- 🚧 Parse PE headers (in progress)
- ⏳ Execute PE binaries (planned)

## Acknowledgments

- **phil-opp** for the excellent "Writing an OS in Rust" tutorial series
- **ReactOS** and **Wine** projects for Windows API reference
- **Rust embedded community** for no_std ecosystem

---

**Status**: Phase 0 Complete ✅ | Phase 1 In Progress 🚧