# Project Roadmap: Windows API Shim for Unikernel

## Project Goal

To execute a standard, compiled Windows x86_64 binary (`.exe`) inside a Unikernel environment without modifying the source binary.

---

## Technology Decisions

| Component | Choice | Rationale |
|-----------|--------|-----------|
| **PE Parser** | `goblin` | More mature, widely used, better documentation and community support |
| **Kernel Base** | Custom Rust Kernel | Full control, easier debugging, inspired by phil-opp/blog_os |
| **Build System** | `justfile` | Modern, cleaner syntax, better for Rust projects |
| **Architecture** | x86_64 | Standard target for Windows binaries |
| **Language** | Rust | Memory safety, no_std support, excellent for systems programming |

---

## Phase 0: Preparation & Artifacts

*Establish the test subjects and development environment.*

- [ ] **Create "Target Zero" Binary**
  - [ ] Write a minimal C application that calls `GetStdHandle`, `WriteFile`, and `ExitProcess`
  - [ ] Compile as a static release build (minimize CRT dependencies)
  - [ ] Verify functionality on a native Windows machine
  - [ ] Document build instructions

- [x] **Select Rust PE Parser**
  - [x] Evaluate `goblin` vs. `pelite` crates
  - [x] **Decision:** `goblin` - More mature ecosystem and multi-format support

- [ ] **Repo Setup**
  - [ ] Initialize Rust workspace
  - [ ] Setup `justfile` for cross-compilation and QEMU launching
  - [ ] Configure workspace members (loader, kernel, api-shim)
  - [ ] Add necessary dependencies to Cargo.toml

---

## Phase 1: The Userspace Prototype (Linux Host)

*Build the loader logic in a standard Linux environment first to utilize `gdb` and standard I/O for debugging.*

- [ ] **Binary Loading**
  - [ ] Implement file reading to load the `.exe` bytes into a buffer
  - [ ] Parse PE Headers (DOS Header, PE Header, Optional Header)
  - [ ] Validate "Magic" bytes and architecture (`PE32+` / x86_64)
  - [ ] Add comprehensive error handling for malformed binaries

- [ ] **Memory Mapping (Virtual Memory)**
  - [ ] Read `Section Headers` (.text, .data, .rdata)
  - [ ] Allocate memory at the specific `VirtualAddress` requested by the binary (using `mmap` for prototype)
  - [ ] Copy raw section data into the mapped memory
  - [ ] Handle section alignment requirements
  - [ ] **Milestone:** Memory layout matches the PE specification

- [ ] **Relocations (The `.reloc` Section)**
  - [ ] Implement Base Relocation parsing
  - [ ] Apply "fixups" if the preferred `ImageBase` cannot be allocated
  - [ ] Test with binaries that require relocation

- [ ] **Import Resolution (The Shim Core)**
  - [ ] Walk the Import Directory Table
  - [ ] Walk the Import Lookup Table (ILT) / Import Address Table (IAT)
  - [ ] Create a "Symbol Resolver" map that maps string names (e.g., `WriteFile`) to local Rust function pointers
  - [ ] Patch the IAT in the loaded binary's memory with the addresses of the Rust functions
  - [ ] Handle missing imports gracefully

---

## Phase 2: The Unikernel Foundation

*Porting the loader to a `no_std` bare-metal environment.*

- [ ] **Kernel Base Setup**
  - [x] **Decision:** Custom Rust Kernel (based on phil-opp tutorials)
  - [ ] Set up bootloader (bootimage or limine)
  - [ ] Initialize basic kernel with VGA buffer/serial output
  - [ ] **Task:** Get the kernel to boot to a "Hello World" serial console output

- [ ] **Memory Manager Integration**
  - [ ] Implement a Physical Memory Manager (PMM)
  - [ ] Implement a Virtual Memory Manager (VMM) capable of mapping pages to specific addresses
  - [ ] Implement page table manipulation
  - [ ] Implement `mprotect` equivalent (mark pages as RX, RW, or RO)
  - [ ] Add memory allocation (global allocator)

- [ ] **Filesystem / Initrd**
  - [ ] Create a mechanism to embed `target.exe` into the Unikernel image
  - [ ] Option A: initramfs with tar archive
  - [ ] Option B: raw byte inclusion via `include_bytes!`
  - [ ] Implement simple filesystem abstraction

---

## Phase 3: The Windows Execution Context

*Fooling the binary into thinking it's on Windows.*

- [ ] **GDT & Segmentation**
  - [ ] Set up the GDT (Global Descriptor Table)
  - [ ] Configure code and data segments
  - [ ] **Critical:** Set the `GS` register to point to a TIB (Thread Information Block)
  - [ ] Windows x64 uses `GS:[0x30]` to access the TEB

- [ ] **TEB & PEB Construction**
  - [ ] Define Rust structs that mimic the layout of the `TEB` (Thread Environment Block)
  - [ ] Define Rust structs for `PEB` (Process Environment Block)
  - [ ] Populate minimal PEB fields (e.g., `ImageBaseAddress`, `ProcessHeap`)
  - [ ] Set up TEB to point to PEB
  - [ ] Verify struct layouts match Windows ABI

- [ ] **Stack Setup**
  - [ ] Allocate a stack for the "Windows" thread
  - [ ] Ensure 16-byte alignment (required for x64 ABI)
  - [ ] Initialize stack with proper guard pages
  - [ ] Set RSP register correctly

---

## Phase 4: API Implementation (Kernel32.dll Shim)

*Implementing the functions required by "Target Zero".*

- [ ] **The "Magic" Jump**
  - [ ] Implement the assembly trampoline to switch stacks (if necessary)
  - [ ] Jump to the `AddressOfEntryPoint`
  - [ ] Ensure all registers are properly initialized per Windows ABI

- [ ] **Implement `kernel32.dll` stubs**
  - [ ] `GetStdHandle`: Return a fake handle index (e.g., `0x1` for stdout)
  - [ ] `WriteFile`: Intercept the buffer and redirect it to the Unikernel's serial port/VGA buffer
  - [ ] `ExitProcess`: Trigger a QEMU shutdown or Unikernel halt
  - [ ] `GetLastError`: Implement a simple thread-local storage variable for error codes
  - [ ] Ensure proper calling convention (x64 Windows ABI)

- [ ] **Testing & Validation**
  - [ ] Run Target Zero binary in userspace prototype
  - [ ] Verify output matches Windows behavior
  - [ ] Test on bare-metal unikernel

---

## Phase 5: Expansion & Hardening

*Moving beyond "Hello World".*

- [ ] **Heap Manager (`HeapAlloc`)**
  - [ ] Implement `GetProcessHeap`
  - [ ] Implement `HeapAlloc` / `HeapFree`
  - [ ] Bridge Windows heap calls to the Unikernel's global allocator
  - [ ] Add heap bounds checking

- [ ] **Command Line Arguments**
  - [ ] Parse command line string
  - [ ] Populate `PEB.ProcessParameters.CommandLine`
  - [ ] Implement `GetCommandLineA` / `GetCommandLineW`

- [ ] **Environment Variables**
  - [ ] Implement basic environment variable storage
  - [ ] Implement `GetEnvironmentVariableA` / `GetEnvironmentVariableW`
  - [ ] Implement `SetEnvironmentVariableA` / `SetEnvironmentVariableW`

- [ ] **File I/O Expansion**
  - [ ] Implement `CreateFileA` / `CreateFileW`
  - [ ] Implement `ReadFile`
  - [ ] Implement `CloseHandle`
  - [ ] Map to unikernel filesystem

- [ ] **Structured Exception Handling (SEH)** *(High Difficulty)*
  - [ ] Stub out `RtlAddFunctionTable`
  - [ ] Parse `.pdata` section (exception handling metadata)
  - [ ] (Long term) Implement minimal exception dispatching
  - [ ] Implement `RtlUnwindEx` stub

- [ ] **Thread Local Storage (TLS)**
  - [ ] Parse TLS directory
  - [ ] Allocate TLS data
  - [ ] Initialize TLS callbacks

---

## Phase 6: Advanced Features (Future)

*For running more complex Windows applications.*

- [ ] **Multi-threading Support**
  - [ ] Implement `CreateThread`
  - [ ] Thread scheduler
  - [ ] Synchronization primitives (mutexes, events)

- [ ] **Advanced Memory Management**
  - [ ] `VirtualAlloc` / `VirtualFree`
  - [ ] `VirtualProtect`
  - [ ] Memory-mapped files

- [ ] **Console I/O**
  - [ ] `ReadConsoleA` / `ReadConsoleW`
  - [ ] Full console emulation

- [ ] **More DLLs**
  - [ ] ntdll.dll basics
  - [ ] user32.dll (for GUI apps - very ambitious)
  - [ ] advapi32.dll (registry, security)

---

## Development Workflow

1. **Incremental Testing**: Test each phase thoroughly before moving to the next
2. **Userspace First**: Always prototype in userspace before porting to bare-metal
3. **Reference Implementations**: Consult ReactOS and Wine source code for ABI details
4. **Debugging**: Use GDB for userspace, serial logging for bare-metal

---

## Success Criteria

- **Phase 0**: Target Zero compiles and runs on Windows
- **Phase 1**: Target Zero runs in Linux userspace prototype and produces correct output
- **Phase 2**: Unikernel boots and can load binary into memory
- **Phase 3**: Unikernel can set up Windows-compatible execution context
- **Phase 4**: Target Zero runs on bare-metal unikernel and produces correct output
- **Phase 5**: More complex Windows binaries work (with heap, file I/O, etc.)

---

## Resources & Documentation

### Specifications
- [Microsoft PE Format Specification](https://learn.microsoft.com/en-us/windows/win32/debug/pe-format)
- [Windows x64 Calling Convention](https://learn.microsoft.com/en-us/cpp/build/x64-calling-convention)
- [Thread Environment Block (TEB) Structure](https://en.wikipedia.org/wiki/Win32_Thread_Information_Block)

### Reference Implementations
- [ReactOS Source Code](https://github.com/reactos/reactos) - Reference for NT structures and API implementations
- [Wine Source Code](https://gitlab.winehq.org/wine/wine) - Reference for loader logic and Windows API
- [LLVM PE/COFF Specification](https://github.com/llvm/llvm-project/blob/main/llvm/include/llvm/BinaryFormat/COFF.h)

### Rust Resources
- [phil-opp/blog_os](https://os.phil-opp.com/) - Writing an OS in Rust
- [goblin crate](https://docs.rs/goblin/) - Binary parsing library
- [Rust Embedded Book](https://rust-embedded.github.io/book/) - no_std development

### Tools
- [PE-bear](https://github.com/hasherezade/pe-bear) - PE file analyzer
- [Dependencies](https://github.com/lucasg/Dependencies) - Modern Dependency Walker
- [x64dbg](https://x64dbg.com/) - Windows debugger for examining PE behavior

---

## Notes

- This is an educational project exploring OS concepts, PE format, and unikernel architecture
- Full Windows compatibility is not the goal; running specific target binaries is
- Security is not a primary concern for the initial implementation
- Performance optimization comes after correctness is established
