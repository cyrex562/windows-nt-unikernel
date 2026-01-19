# AI Agent Navigation Guide

**Purpose**: Help AI assistants (Claude, GitHub Copilot, Gemini, etc.) quickly understand and navigate the Windows NT Unikernel codebase.

**Last Updated**: 2026-01-19

---

## Quick Start for AI Agents

### Project Summary (One-Paragraph Pitch)

This project implements a minimal bare-metal x86_64 unikernel that executes unmodified Windows PE executables without Windows. It parses PE binaries, maps them to memory, resolves imports to Rust-implemented Windows API functions (kernel32.dll), and executes them directly on hardware/QEMU. The project is structured in phases: Phase 1 is a userspace Linux prototype, Phase 2+ ports to bare-metal with full memory management and Windows execution context (TEB/PEB/GS register setup).

### Current Status

**Phase 0**: ✅ Complete - Project bootstrapped, structure created, Target Zero binary defined
**Phase 1**: 🚧 In Progress - Implementing userspace PE loader on Linux
**Phase 2-5**: 📋 Planned - See phase documents for details

---

## Codebase Structure

###Overview
```
windows-nt-unikernel/
├── crates/                  # Rust workspace with 4 crates
│   ├── common/              # [no_std compatible] Shared types (Windows types, PE structures, TEB/PEB)
│   ├── pe-loader/           # [Phase 1] Userspace PE loader prototype (std, runs on Linux)
│   ├── api-shim/            # [no_std compatible] Windows API implementations (kernel32.dll)
│   └── kernel/              # [Phase 2+] Bare-metal no_std kernel (boots via bootloader crate)
├── target-zero/             # Minimal Windows test binary (C, cross-compiled with MinGW)
├── docs/                    # Documentation
│   ├── PHASE1.md            # Detailed Phase 1 task checklist (~194 tasks)
│   ├── PHASE2.md            # Phase 2 tasks (kernel, memory management)
│   ├── PHASE3.md            # Phase 3 tasks (Windows execution context)
│   ├── PHASE4.md            # Phase 4 tasks (API implementation, first execution)
│   ├── PHASE5.md            # Phase 5 tasks (expansion: heap, files, cmdline)
│   ├── DESIGN.md            # Comprehensive architecture document
│   └── AI_AGENT_GUIDE.md    # This file
├── ROADMAP.md               # High-level project roadmap
├── README.md                # User-facing documentation
├── justfile                 # Build automation (use `just --list`)
└── Cargo.toml               # Workspace root
```

---

## Critical Files by Use Case

### "Where is the PE parsing logic?"

**Phase 1** (userspace prototype):
- `crates/pe-loader/src/loader.rs` - Loads and parses PE files
- `crates/pe-loader/src/memory.rs` - Memory mapping (uses mmap)
- `crates/pe-loader/src/imports.rs` - Import resolution and IAT patching
- `crates/pe-loader/src/reloc.rs` - Base relocations (Phase 1, to be created)

**Phase 2+** (kernel):
- `crates/kernel/src/pe_loader/loader.rs` - Ported PE parsing (no_std)
- `crates/kernel/src/pe_loader/memory.rs` - Uses VMM instead of mmap
- `crates/kernel/src/pe_loader/embedded.rs` - Accesses embedded binary via include_bytes!

**Dependencies**:
- Uses `goblin` crate for PE header parsing
- See `crates/common/src/pe.rs` for PE-related types

### "Where are the Windows API functions implemented?"

**API implementations**:
- `crates/api-shim/src/kernel32.rs` - kernel32.dll functions (GetStdHandle, WriteFile, ExitProcess, GetLastError, SetLastError)
- `crates/api-shim/src/lib.rs` - Error handling, last error storage

**Phase 5 additions**:
- `crates/api-shim/src/heap.rs` - Heap management (HeapAlloc, HeapFree)
- `crates/api-shim/src/ntdll.rs` - ntdll.dll functions (future)

**Calling convention**: All functions use `#[no_mangle]` and `extern "C"` to match Windows x64 calling convention (RCX, RDX, R8, R9 for first 4 params).

### "Where are Windows structures (TEB, PEB) defined?"

**Common definitions**:
- `crates/common/src/windows.rs` - TEB, PEB, RTL_USER_PROCESS_PARAMETERS, etc.
- `crates/common/src/lib.rs` - HANDLE, DWORD, BOOL, error codes

**Kernel-specific**:
- `crates/kernel/src/windows_compat/teb.rs` - TEB allocation and initialization
- `crates/kernel/src/windows_compat/peb.rs` - PEB allocation and initialization
- `crates/kernel/src/windows_compat/gs_setup.rs` - GS register setup (for TEB access)

**Key insight**: GS:[0x30] points to TEB in Windows. We set IA32_GS_BASE MSR to TEB address.

### "Where is the memory management?"

**Kernel memory management**:
- `crates/kernel/src/memory/physical.rs` - Physical Memory Manager (PMM), frame allocation
- `crates/kernel/src/memory/virtual.rs` - Virtual Memory Manager (VMM), page tables
- `crates/kernel/src/memory/heap.rs` - Heap allocator (wraps linked_list_allocator)
- `crates/kernel/src/memory/layout.rs` - Memory layout constants

**Memory layout**:
- See DESIGN.md "Memory Layout" section for detailed map
- PE binaries load at ~0x0000_0000_0040_0000
- Kernel code identity-mapped from 0x0000_0000_0000_1000

### "Where is the kernel entry point?"

**Boot sequence**:
1. Bootloader (from bootloader crate) loads kernel and jumps to `_start`
2. `crates/kernel/src/main.rs` - `_start()` function
3. Initialize hardware (GDT, IDT, serial, VGA)
4. Initialize memory (PMM, VMM, heap)
5. Load PE binary
6. Set up Windows environment (TEB, PEB, GS, stack)
7. Jump to PE entry point

**Hardware init**:
- `crates/kernel/src/gdt.rs` - Global Descriptor Table
- `crates/kernel/src/interrupts/mod.rs` - Interrupt Descriptor Table
- `crates/kernel/src/serial.rs` - Serial port (for logging)
- `crates/kernel/src/vga_buffer.rs` - VGA text buffer

### "Where is the execution jump to PE entry point?"

**Execution**:
- `crates/kernel/src/windows_compat/execution.rs` - Entry point jump
- Assembly trampoline (`jump_to_entry`) sets RSP, clears registers, jumps to RIP
- `crates/kernel/src/windows_compat/stack.rs` - Stack allocation

**Calling convention adherence**:
- Stack must be 16-byte aligned
- Return address pushed to stack (points to ExitProcess wrapper)
- See DESIGN.md "Execution Model" section for details

### "Where is the test binary (target-zero.exe)?"

**Test binary**:
- `target-zero/target-zero.c` - Minimal Windows C program
- Calls: GetStdHandle → WriteFile → ExitProcess
- Outputs: "Hello from Target Zero!"

**Build**:
- `target-zero/Makefile` - Cross-compiles with MinGW (x86_64-w64-mingw32-gcc)
- `just build-target-zero` - Convenience command

**Embedding in kernel**:
- `crates/kernel/src/pe_loader/embedded.rs` uses `include_bytes!` to embed target-zero.exe

---

## Common AI Agent Tasks

### Task: "Add a new Windows API function"

**Steps**:
1. **Define function in `crates/api-shim/src/kernel32.rs`**:
   ```rust
   #[no_mangle]
   pub extern "C" fn NewFunction(param1: DWORD) -> BOOL {
       // Validate parameters
       // Implement functionality
       // Set last error if needed
       // Return result
   }
   ```

2. **Register in symbol resolver** (`crates/pe-loader/src/imports.rs` or `crates/kernel/src/pe_loader/imports.rs`):
   ```rust
   resolver.register("kernel32.dll", "NewFunction", NewFunction as usize);
   ```

3. **Test**:
   - Create C test binary that calls NewFunction
   - Compile with MinGW
   - Run in pe-loader (Phase 1) or kernel (Phase 2+)

4. **Document**:
   - Add rustdoc comments
   - Update API reference in docs

### Task: "Debug a page fault"

**Diagnosis steps**:
1. **Check page fault handler** (`crates/kernel/src/interrupts/handlers.rs`):
   - Logs faulting address, error code, RIP

2. **Common causes**:
   - **Unmapped page**: Section not mapped during PE loading
   - **Wrong permissions**: Execute on non-executable page
   - **Invalid pointer**: API function received bad pointer
   - **Stack overflow**: Stack too small or corrupted

3. **Debugging tools**:
   - Serial output: `serial_println!` in `crates/kernel/src/serial.rs`
   - Memory dump: `crates/kernel/src/windows_compat/debug.rs`
   - GDB: `just debug-qemu` then `gdb -ex "target remote :1234"`

4. **Check memory layout**:
   - Verify section is in section table
   - Verify RVA → VA translation correct
   - Check page table mappings with VMM debug output

### Task: "Implement a new phase task"

**Process**:
1. **Read phase document** (e.g., `docs/PHASE1.md`)
2. **Find task** (tasks are numbered, e.g., **1.2.3**)
3. **Locate file** (specified in task section header)
4. **Implement task**:
   - Follow task description
   - Add error handling
   - Add logging
   - Write tests
5. **Mark complete** in phase document (checkbox)
6. **Commit** with message referencing task number

**Example**:
```
git commit -m "feat(pe-loader): Implement DOS header parsing (task 2.2.1-2.2.4)

- Validate MZ signature
- Read e_lfanew offset
- Validate offset is within bounds
- Add logging for DOS header info
"
```

### Task: "Understand the PE loading flow"

**High-level flow** (see DESIGN.md for details):
1. Read binary (file or embedded)
2. Parse DOS header (MZ signature, e_lfanew)
3. Parse PE header (PE signature)
4. Parse COFF header (machine type, sections count)
5. Parse optional header (entry point, image base, sizes)
6. Parse data directories (imports, relocations, etc.)
7. Parse section headers (.text, .data, .rdata, etc.)
8. Allocate memory (at preferred base or elsewhere)
9. Copy headers to base
10. Map sections (copy section data)
11. Apply relocations (if base changed)
12. Resolve imports (patch IAT)
13. Set protections (RWX for each section)
14. Ready to execute!

**Key files to read in order**:
1. `crates/common/src/pe.rs` - PE structures
2. `crates/pe-loader/src/loader.rs` - Parsing
3. `crates/pe-loader/src/memory.rs` - Mapping
4. `crates/pe-loader/src/reloc.rs` - Relocations
5. `crates/pe-loader/src/imports.rs` - Import resolution

### Task: "Run tests"

**Commands**:
```bash
# Run all Rust tests
just test

# Build everything
just build-all

# Run Phase 1 PE loader (userspace)
just run-loader

# Run Phase 2+ kernel (QEMU)
just run-qemu

# Run kernel with GDB
just debug-qemu
# Then in another terminal: gdb -ex "target remote :1234"

# Build Target Zero binary
just build-target-zero

# Check code (fast, no build)
just check

# Format code
just fmt

# Run clippy linter
just clippy
```

---

## Code Patterns and Conventions

### Error Handling

**Phase 1** (userspace):
```rust
use anyhow::{Result, Context};

fn load_binary(path: &str) -> Result<Vec<u8>> {
    fs::read(path).context("Failed to read PE binary")
}
```

**Phase 2+** (kernel, no_std):
```rust
type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ParseError,
    MemoryError,
    ImportError,
}

fn load_binary() -> Result<Vec<u8>> {
    // ...
    Ok(data)
}
```

### Logging

**Phase 1**:
```rust
log::info!("Loading PE binary: {}", path);
```

**Phase 2+**:
```rust
serial_println!("Loading PE binary");
```

### Unsafe Code

**When needed**:
- Hardware access (MSRs, I/O ports)
- Assembly (entry point jump, GS register)
- Raw pointers (PE binary manipulation)
- FFI (API functions callable from Windows binary)

**Pattern**:
```rust
// SAFETY: <Explain why this is safe>
unsafe {
    // Unsafe operation
}
```

**Example**:
```rust
// SAFETY: TEB pointer is valid and properly initialized.
// GS base is set to TEB address, so GS:[0x30] returns TEB address.
unsafe {
    let teb: *const TEB;
    asm!("mov {}, gs:[0x30]", out(reg) teb);
    (*teb).last_error
}
```

### Naming Conventions

- **Types**: PascalCase (e.g., `LoadedPE`, `TEB`, `PEB`)
- **Functions**: snake_case (e.g., `load_binary`, `parse_headers`)
- **Constants**: SCREAMING_SNAKE_CASE (e.g., `STD_OUTPUT_HANDLE`)
- **Windows API functions**: PascalCase (e.g., `GetStdHandle`, `WriteFile`) - matches Windows

### Module Organization

```rust
// Module declaration
mod submodule;

// Re-exports
pub use submodule::{Type, function};

// Imports
use crate::other_module::Thing;
use core::ptr;
```

---

## Phase-Specific Guidance

### Working on Phase 1 (Userspace Prototype)

**Context**: Building PE loader that runs on Linux, uses standard library.

**Key files**:
- `crates/pe-loader/src/*.rs`

**Testing**:
- Can use `cargo test`
- Can use GDB for debugging
- Can run directly: `cargo run -p pe-loader -- target-zero/target-zero.exe`

**Gotchas**:
- Remember this code will be ported to no_std later
- Avoid Linux-specific syscalls if possible
- Use abstractions that can be replaced (e.g., mmap → VMM)

### Working on Phase 2 (Kernel Foundation)

**Context**: Porting to bare-metal, no standard library.

**Key files**:
- `crates/kernel/src/*.rs`

**Testing**:
- Must use QEMU: `just run-qemu`
- Use serial output for debugging
- Can use GDB: `just debug-qemu`

**Gotchas**:
- No `std::fs`, `std::io`, etc.
- Must use `serial_println!` instead of `println!`
- All memory must be explicitly managed
- Panics halt the system (no unwinding)

### Working on Phase 3-5 (Features)

**Context**: Adding functionality to kernel and API shim.

**Approach**:
- Implement API function
- Register in symbol resolver
- Create test binary
- Test in kernel

**Remember**:
- All API functions must be `extern "C"`
- Follow Windows x64 calling convention
- Set last error appropriately
- Add logging for debugging

---

## Common Pitfalls for AI Agents

### 1. Mixing Phase 1 and Phase 2 Code

**Symptom**: Using `std::fs` in kernel code, or `serial_println!` in userspace code.

**Solution**: Check which crate you're in:
- `crates/pe-loader/` → Phase 1, uses `std`
- `crates/kernel/` → Phase 2+, no `std`

### 2. Incorrect Calling Convention

**Symptom**: API function not being called correctly, crashes or wrong parameters.

**Solution**: Ensure function is:
```rust
#[no_mangle]
pub extern "C" fn FunctionName(params...) -> ReturnType {
```

The `extern "C"` is crucial for Windows x64 calling convention.

### 3. Forgetting to Register API Function

**Symptom**: Import resolution fails with "unknown function".

**Solution**: Add to symbol resolver:
```rust
resolver.register("kernel32.dll", "FunctionName", FunctionName as usize);
```

### 4. Null Pointer Dereference

**Symptom**: Page fault, kernel panic.

**Solution**: Validate all pointers before dereferencing:
```rust
if ptr.is_null() {
    set_last_error(ERROR_INVALID_PARAMETER);
    return 0; // FALSE
}
unsafe { *ptr = value; }
```

### 5. Stack Misalignment

**Symptom**: Crashes or strange behavior when calling PE entry point.

**Solution**: Ensure stack is 16-byte aligned:
```rust
let stack_top = (stack_base + STACK_SIZE) & !0xF; // Align to 16 bytes
```

### 6. TEB/PEB Structure Mismatch

**Symptom**: PE binary reads wrong data from GS:[offset].

**Solution**: Verify field offsets match Windows:
- TEB: GS:[0x30] → TEB address
- TEB: GS:[0x60] → PEB address
- Use `#[repr(C)]` on all structures
- Check with `core::mem::offset_of!` (Rust 1.77+)

### 7. Forgetting to Flush TLB

**Symptom**: Page table changes don't take effect.

**Solution**: After modifying page tables:
```rust
unsafe {
    asm!("invlpg [{}]", in(reg) virtual_address);
}
```

Or flush entire TLB:
```rust
unsafe {
    let cr3: u64;
    asm!("mov {}, cr3", out(reg) cr3);
    asm!("mov cr3, {}", in(reg) cr3);
}
```

---

## Asking Good Questions

When stuck, gather this information:

### For PE Loading Issues:
- Which phase? (1 or 2+)
- Error message or symptom
- Which PE binary? (target-zero.exe or other)
- Output of `objdump -x binary.exe | head -100`
- Relevant logs from serial output

### For Memory Issues:
- Faulting address
- Error code (from page fault handler)
- Instruction pointer (RIP)
- Expected vs. actual page mapping
- Which section was being accessed?

### For API Issues:
- Which function?
- Input parameters
- Expected output
- Actual output or error
- GetLastError value

### For Build Issues:
- Full error message
- Rust version (`rustc --version`)
- Cargo.toml dependencies
- Operating system

---

## Useful Debugging Commands

### QEMU
```bash
# Run with serial output
qemu-system-x86_64 -drive format=raw,file=kernel.bin -serial stdio -display none

# Run with GDB server
qemu-system-x86_64 -drive format=raw,file=kernel.bin -serial stdio -display none -s -S

# Run with QEMU monitor
qemu-system-x86_64 -drive format=raw,file=kernel.bin -serial stdio -monitor telnet:127.0.0.1:1234,server,nowait
```

### GDB
```bash
# Connect to QEMU
target remote :1234

# Set breakpoint at address
break *0x<address>

# Continue execution
continue

# Print register
info registers

# Print memory
x/10gx 0x<address>

# Disassemble
disassemble $rip

# Single step
stepi
```

### Cargo
```bash
# Build with verbose output
cargo build -v

# Show expanded macros
cargo expand

# Check for common issues
cargo clippy --all-targets

# Build documentation
cargo doc --open
```

---

## Quick Reference: Key Memory Addresses

| Address | Description |
|---------|-------------|
| `0x0000_0000_0000_0000` | Null page (unmapped) |
| `0x0000_0000_0000_1000` | Kernel code start (identity mapped) |
| `0x0000_0000_0040_0000` | Default PE load address |
| `0x0000_0040_0000_0000` | Stack region |
| `0xB8000` | VGA text buffer (mapped in higher address space) |
| `0x3F8` | COM1 serial port (I/O port, not memory) |

---

## Quick Reference: Windows x64 Calling Convention

| Parameter | Register | Notes |
|-----------|----------|-------|
| 1 | RCX | |
| 2 | RDX | |
| 3 | R8 | |
| 4 | R9 | |
| 5+ | Stack | Right to left |
| Return | RAX | |

**Stack frame**:
```
[RSP + 0x00]: Return address
[RSP + 0x08]: Shadow space (param 1)
[RSP + 0x10]: Shadow space (param 2)
[RSP + 0x18]: Shadow space (param 3)
[RSP + 0x20]: Shadow space (param 4)
[RSP + 0x28]: Param 5 (if any)
[RSP + 0x30]: Param 6 (if any)
...
```

**Before call**: RSP must be 16-byte aligned.

---

## Additional Resources

- **DESIGN.md**: Comprehensive architecture and design decisions
- **PHASE*.md**: Detailed task checklists for each phase
- **ROADMAP.md**: High-level project roadmap
- **README.md**: User-facing documentation

**External resources**:
- [PE Format Spec](https://learn.microsoft.com/en-us/windows/win32/debug/pe-format)
- [Windows x64 ABI](https://learn.microsoft.com/en-us/cpp/build/x64-calling-convention)
- [phil-opp's OS tutorial](https://os.phil-opp.com/)
- [OSDev Wiki](https://wiki.osdev.org/)

---

## For AI Code Assistants: How to Help

### When asked to implement a feature:
1. Identify which phase it belongs to
2. Check the phase document for related tasks
3. Locate the correct file(s)
4. Follow project conventions (error handling, logging, unsafe)
5. Add tests if possible
6. Add documentation comments

### When debugging:
1. Ask for symptom and context (phase, file, error message)
2. Check common pitfalls section
3. Suggest logging/debugging commands
4. Propose hypothesis and how to verify it

### When explaining code:
1. Reference DESIGN.md for architecture
2. Use diagrams (ASCII art is fine)
3. Explain both "what" and "why"
4. Point to related code and documents

---

## Conclusion

This guide should help you quickly orient yourself in the codebase. Remember:

1. **Start with DESIGN.md** for architecture overview
2. **Check phase documents** for detailed tasks
3. **Use `just --list`** for available commands
4. **Log liberally** with `serial_println!`
5. **Test incrementally** - don't try to do too much at once
6. **Ask specific questions** with context

Happy hacking! 🚀
