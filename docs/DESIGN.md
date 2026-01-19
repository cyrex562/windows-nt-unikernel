# Windows NT Unikernel - Design Document

**Version**: 1.0
**Last Updated**: 2026-01-19
**Status**: Phase 0 Complete, Phase 1+ In Planning

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [System Architecture](#system-architecture)
3. [Component Design](#component-design)
4. [PE Loading Process](#pe-loading-process)
5. [Memory Management](#memory-management)
6. [Windows API Compatibility](#windows-api-compatibility)
7. [Execution Model](#execution-model)
8. [Design Decisions](#design-decisions)
9. [Security Considerations](#security-considerations)
10. [Performance Considerations](#performance-considerations)
11. [Future Directions](#future-directions)

---

## Executive Summary

### Project Goal

Build a minimal unikernel that can execute unmodified Windows x86_64 PE binaries on bare-metal hardware without Windows.

### Approach

1. **Phase 1**: Build userspace prototype on Linux for rapid development and debugging
2. **Phase 2-3**: Port to bare-metal with memory management and Windows execution context
3. **Phase 4**: Execute first binary successfully
4. **Phase 5+**: Expand API support for more complex binaries

### Key Innovation

Instead of emulating Windows, we provide a **shim layer** that implements Windows API functions using kernel primitives, allowing Windows binaries to run directly on bare-metal.

---

## System Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────┐
│           Windows PE Binary (.exe)                       │
│       (Unmodified x86_64 Windows Executable)             │
└────────────────────┬────────────────────────────────────┘
                     │
                     │ [Imports via IAT]
                     ▼
┌─────────────────────────────────────────────────────────┐
│         Windows API Shim Layer                          │
│  ┌────────────────┐  ┌────────────────┐                │
│  │  kernel32.dll  │  │   ntdll.dll    │                │
│  │                │  │                 │                │
│  │ • GetStdHandle │  │ • RtlAddFunc...│                │
│  │ • WriteFile    │  │ • RtlUnwindEx  │                │
│  │ • ReadFile     │  │ • ...          │                │
│  │ • ExitProcess  │  │                 │                │
│  │ • HeapAlloc    │  │                 │                │
│  │ • ...          │  │                 │                │
│  └────────────────┘  └────────────────┘                │
└────────────────────┬────────────────────────────────────┘
                     │
                     │ [Kernel API]
                     ▼
┌─────────────────────────────────────────────────────────┐
│              Bare-metal Rust Kernel                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
│  │  PE Loader  │  │   Memory    │  │   I/O       │    │
│  │             │  │  Management  │  │  Subsystem  │    │
│  │ • Parser    │  │             │  │             │    │
│  │ • Mapper    │  │ • PMM       │  │ • Serial    │    │
│  │ • Relocator │  │ • VMM       │  │ • VGA       │    │
│  │ • Importer  │  │ • Heap      │  │ • (Future)  │    │
│  └─────────────┘  └─────────────┘  └─────────────┘    │
│                                                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
│  │  Hardware   │  │  Interrupt  │  │  Windows    │    │
│  │    Init     │  │   Handlers  │  │   Context   │    │
│  │             │  │             │  │             │    │
│  │ • GDT       │  │ • IDT       │  │ • TEB/PEB   │    │
│  │ • TSS       │  │ • Exceptions│  │ • GS Setup  │    │
│  │ • MSRs      │  │ • Page Flt  │  │ • Stack     │    │
│  └─────────────┘  └─────────────┘  └─────────────┘    │
└────────────────────┬────────────────────────────────────┘
                     │
                     │ [Direct Access]
                     ▼
┌─────────────────────────────────────────────────────────┐
│              x86_64 Hardware                            │
│  • CPU (Long Mode)                                       │
│  • Memory (Page Tables)                                  │
│  • I/O Devices (Serial, VGA, etc.)                       │
└─────────────────────────────────────────────────────────┘
```

### Data Flow: From Boot to Execution

```
[1] BIOS/UEFI
     ↓
[2] Bootloader (bootloader crate)
     ↓ [Loads kernel, sets up paging, jumps to _start]
[3] Kernel Entry (_start)
     ↓
[4] Initialize Hardware
     ├─> GDT (segment descriptors)
     ├─> IDT (exception/interrupt handlers)
     ├─> Serial (debugging output)
     └─> VGA (visual output)
     ↓
[5] Initialize Memory Management
     ├─> PMM (physical frame allocator)
     ├─> VMM (page table manager)
     └─> Heap (global allocator)
     ↓
[6] Load PE Binary
     ├─> Read from embedded data (include_bytes!)
     ├─> Parse PE headers
     ├─> Allocate memory at base address
     ├─> Map sections (.text, .data, .rdata, etc.)
     ├─> Apply relocations (if base changed)
     └─> Resolve imports (patch IAT)
     ↓
[7] Set Up Windows Environment
     ├─> Allocate and initialize TEB
     ├─> Allocate and initialize PEB
     ├─> Link TEB → PEB
     ├─> Set GS register to TEB address
     └─> Allocate and prepare stack
     ↓
[8] Jump to Entry Point
     ├─> Prepare registers (RSP, RIP, etc.)
     ├─> Use assembly trampoline
     └─> Execute PE binary code
     ↓
[9] API Calls
     ├─> Binary calls imported function
     ├─> IAT redirects to our implementation
     ├─> Shim function executes
     └─> Returns to binary
     ↓
[10] Exit
     └─> ExitProcess() halts kernel
```

---

## Component Design

### 1. Common Library (`crates/common/`)

**Purpose**: Shared types and structures used by all components.

**Key Types**:
- `HANDLE`, `DWORD`, `BOOL` - Windows primitive types
- `TEB`, `PEB` - Thread and process environment blocks
- `SectionCharacteristics` - PE section flags

**Design Principles**:
- `no_std` compatible (can be used in kernel)
- Minimal dependencies
- Match Windows ABI exactly (repr(C))

**File Structure**:
```
common/
├── src/
│   ├── lib.rs         # Module root, common types
│   ├── pe.rs          # PE format structures
│   └── windows.rs     # Windows API structures (TEB, PEB, etc.)
└── Cargo.toml
```

### 2. PE Loader (`crates/pe-loader/`)

**Purpose**: Parse and load Windows PE executables.

**Phase 1**: Userspace prototype (Linux)
- Uses `std` library
- Uses `mmap` for memory mapping
- Uses `goblin` crate for PE parsing
- Enables rapid development and debugging

**Phase 2+**: Kernel version (bare-metal)
- Ported to `no_std`
- Uses VMM for memory mapping
- Shares parsing logic with Phase 1

**Key Operations**:
1. **Parse PE Headers**: DOS header, COFF header, optional header, sections
2. **Map Sections**: Allocate memory and copy section data
3. **Apply Relocations**: Fix up addresses if loaded at different base
4. **Resolve Imports**: Patch IAT with addresses of our API functions
5. **Prepare Execution**: Set up entry point

**File Structure**:
```
pe-loader/
├── src/
│   ├── main.rs        # Entry point (Phase 1 CLI)
│   ├── loader.rs      # PE parsing and validation
│   ├── memory.rs      # Memory mapping (mmap → VMM)
│   ├── imports.rs     # Import resolution and IAT patching
│   └── reloc.rs       # Base relocation processing
└── Cargo.toml
```

**Critical Data Structures**:

```rust
// Loaded PE representation
pub struct LoadedPE {
    pub base_address: u64,
    pub entry_point: u64,
    pub image_size: u64,
    pub sections: Vec<Section>,
    pub import_table: ImportTable,
    pub relocation_table: Option<RelocationTable>,
}

// Section information
pub struct Section {
    pub name: String,
    pub virtual_address: u32,
    pub virtual_size: u32,
    pub raw_data_offset: u32,
    pub raw_data_size: u32,
    pub characteristics: SectionCharacteristics,
}
```

### 3. API Shim (`crates/api-shim/`)

**Purpose**: Implement Windows API functions.

**Supported DLLs**:
- `kernel32.dll`: Core Windows API
- `ntdll.dll`: Native API (future)

**Implementation Strategy**:
- Each function has `#[no_mangle]` and `extern "C"` for correct calling convention
- Functions use Rust implementations of Windows behavior
- Error handling via `GetLastError`/`SetLastError`

**Phase 4 API Functions**:
```rust
// kernel32.dll
pub extern "C" fn GetStdHandle(std_handle: DWORD) -> HANDLE;
pub extern "C" fn WriteFile(...) -> BOOL;
pub extern "C" fn ExitProcess(exit_code: u32) -> !;
pub extern "C" fn GetLastError() -> DWORD;
pub extern "C" fn SetLastError(error: DWORD);
```

**Phase 5+ API Functions**:
```rust
// Heap management
pub extern "C" fn GetProcessHeap() -> HANDLE;
pub extern "C" fn HeapAlloc(heap: HANDLE, flags: DWORD, size: usize) -> *mut u8;
pub extern "C" fn HeapFree(heap: HANDLE, flags: DWORD, ptr: *mut u8) -> BOOL;

// File I/O
pub extern "C" fn CreateFileW(...) -> HANDLE;
pub extern "C" fn ReadFile(...) -> BOOL;
pub extern "C" fn CloseHandle(handle: HANDLE) -> BOOL;

// Command line
pub extern "C" fn GetCommandLineW() -> *const u16;

// Environment
pub extern "C" fn GetEnvironmentVariableW(...) -> DWORD;
pub extern "C" fn SetEnvironmentVariableW(...) -> BOOL;
```

**File Structure**:
```
api-shim/
├── src/
│   ├── lib.rs         # Module root, common error handling
│   ├── kernel32.rs    # kernel32.dll functions
│   ├── ntdll.rs       # ntdll.dll functions (Phase 5+)
│   └── heap.rs        # Heap management (Phase 5+)
└── Cargo.toml
```

### 4. Kernel (`crates/kernel/`)

**Purpose**: Bare-metal kernel that hosts the PE loader and API shim.

**Key Responsibilities**:
- Boot and hardware initialization
- Memory management (PMM, VMM, heap)
- Interrupt/exception handling
- I/O (serial, VGA)
- Windows compatibility layer (TEB, PEB, GS setup)
- PE binary execution

**File Structure**:
```
kernel/
├── src/
│   ├── main.rs                  # Entry point, boot sequence
│   ├── gdt.rs                   # Global Descriptor Table
│   ├── interrupts/
│   │   ├── mod.rs               # IDT, exception handlers
│   │   └── handlers.rs          # Specific handlers (page fault, etc.)
│   ├── memory/
│   │   ├── mod.rs               # Memory management root
│   │   ├── physical.rs          # Physical Memory Manager (PMM)
│   │   ├── virtual.rs           # Virtual Memory Manager (VMM)
│   │   ├── heap.rs              # Heap allocator
│   │   └── layout.rs            # Memory layout definitions
│   ├── serial.rs                # Serial port driver
│   ├── vga_buffer.rs            # VGA text mode driver
│   ├── pe_loader/
│   │   ├── mod.rs               # PE loader (ported from Phase 1)
│   │   ├── loader.rs
│   │   ├── memory.rs            # Uses VMM instead of mmap
│   │   ├── imports.rs
│   │   ├── reloc.rs
│   │   └── embedded.rs          # Access embedded binary
│   ├── windows_compat/
│   │   ├── mod.rs               # Windows compatibility layer
│   │   ├── teb.rs               # TEB structure and initialization
│   │   ├── peb.rs               # PEB structure and initialization
│   │   ├── gs_setup.rs          # GS register setup
│   │   ├── stack.rs             # Stack allocation and setup
│   │   ├── execution.rs         # Entry point jump
│   │   └── debug.rs             # Debugging utilities
│   └── api_shim/
│       └── mod.rs               # Re-export api-shim with no_std glue
└── Cargo.toml
```

---

## PE Loading Process

### Overview

Loading a Windows PE binary involves several complex steps. Here's the detailed process:

### Step 1: Read Binary

**Input**: Path to .exe file (Phase 1) or embedded bytes (Phase 2+)
**Output**: Raw binary data in memory

```rust
// Phase 1 (userspace)
let data = std::fs::read("target.exe")?;

// Phase 2+ (kernel)
static TARGET_BINARY: &[u8] = include_bytes!("../../target-zero/target-zero.exe");
let data = TARGET_BINARY;
```

### Step 2: Parse PE Headers

**Components**:
1. **DOS Header** (offset 0x00):
   - Magic: `MZ` (0x5A4D)
   - `e_lfanew`: Offset to PE header
2. **PE Signature** (offset `e_lfanew`):
   - Magic: `PE\0\0` (0x00004550)
3. **COFF Header**:
   - Machine type: 0x8664 (AMD64)
   - Number of sections
   - Size of optional header
4. **Optional Header** (PE32+):
   - Magic: 0x20B (PE32+, 64-bit)
   - Entry point RVA
   - Image base (preferred load address)
   - Section/file alignment
   - Image size
   - Headers size
   - Subsystem (console, GUI, etc.)
   - Data directories (16 entries)
5. **Section Headers**:
   - For each section: name, VAs, sizes, characteristics

**Validation**:
- Check all magic numbers
- Verify architecture (x86_64)
- Validate offsets are within file bounds
- Verify sections don't overlap

### Step 3: Allocate Memory

**Goal**: Allocate a contiguous virtual memory region for the PE image.

**Process**:
1. Calculate total size (`image_size` from optional header)
2. Try to allocate at preferred base address
3. If that fails, allocate elsewhere (will need relocations)
4. Zero the allocated memory

**Phase 1** (mmap):
```rust
let ptr = unsafe {
    libc::mmap(
        preferred_base as *mut _,
        image_size,
        PROT_READ | PROT_WRITE,
        MAP_PRIVATE | MAP_ANONYMOUS | MAP_FIXED_NOREPLACE,
        -1,
        0,
    )
};
```

**Phase 2+** (VMM):
```rust
let base_addr = vmm.allocate_region(image_size, preferred_base)?;
for page in base_addr..base_addr + image_size {
    let frame = pmm.allocate_frame()?;
    vmm.map_page(page, frame, PageFlags::WRITABLE)?;
}
```

### Step 4: Copy Headers

Copy DOS header, PE headers, and section table to the base address.

```rust
unsafe {
    ptr::copy_nonoverlapping(
        data.as_ptr(),
        base_address as *mut u8,
        headers_size,
    );
}
```

### Step 5: Map Sections

For each section:
1. Calculate destination: `base_address + section.virtual_address`
2. Copy section data from file: `section.pointer_to_raw_data`
3. Copy `min(virtual_size, raw_size)` bytes
4. If `virtual_size > raw_size`, zero remaining bytes (BSS)

```rust
for section in sections {
    let dest = base_address + section.virtual_address;
    let src = data[section.pointer_to_raw_data..];
    let copy_size = min(section.virtual_size, section.size_of_raw_data);

    unsafe {
        ptr::copy_nonoverlapping(src.as_ptr(), dest as *mut u8, copy_size);
        if section.virtual_size > copy_size {
            ptr::write_bytes(
                (dest + copy_size) as *mut u8,
                0,
                section.virtual_size - copy_size,
            );
        }
    }
}
```

### Step 6: Apply Relocations

**When Needed**: If `actual_base != preferred_base`

**Relocation Process**:
1. Calculate delta: `delta = actual_base - preferred_base`
2. Parse base relocation directory (`.reloc` section)
3. For each relocation block:
   - Block contains a page RVA and list of offsets
   - For each offset in the block:
     - Calculate target address: `base + block.rva + offset`
     - Read current value at target
     - Add delta to current value
     - Write back

**Relocation Types** (x86_64):
- `IMAGE_REL_BASED_DIR64` (type 10): Add full 64-bit delta [MOST COMMON]
- `IMAGE_REL_BASED_ABSOLUTE` (type 0): No-op, used for padding

```rust
let delta = (actual_base as i64) - (preferred_base as i64);

for block in relocation_blocks {
    for entry in block.entries {
        let rva = block.virtual_address + entry.offset;
        let target = (base_address + rva) as *mut u64;

        unsafe {
            let value = *target;
            *target = (value as i64 + delta) as u64;
        }
    }
}
```

### Step 7: Resolve Imports

**Goal**: Replace IAT entries with addresses of our API functions.

**Import Process**:
1. Parse import directory (data directory index 1)
2. For each DLL:
   - Read DLL name
   - Get Import Lookup Table (ILT) and Import Address Table (IAT)
   - For each import:
     - Read import name (or ordinal)
     - Look up function in symbol resolver
     - Write function address to IAT

```rust
for dll in import_descriptors {
    let dll_name = read_string(base + dll.name_rva);
    let ilt = base + dll.original_first_thunk;
    let iat = base + dll.first_thunk;

    for (i, entry) in ilt_entries.enumerate() {
        let import_name = if entry.is_ordinal() {
            ImportName::Ordinal(entry.ordinal())
        } else {
            let name_struct = base + entry.rva;
            ImportName::Name(read_string(name_struct + 2))
        };

        let func_addr = symbol_resolver.resolve(&dll_name, &import_name)?;

        unsafe {
            *(iat + i * 8) = func_addr as u64;
        }
    }
}
```

### Step 8: Set Memory Protections

Set each section's memory protection based on characteristics:
- `.text`: `PROT_READ | PROT_EXEC` (RX)
- `.rdata`: `PROT_READ` (R)
- `.data`, `.bss`: `PROT_READ | PROT_WRITE` (RW)

```rust
for section in sections {
    let mut prot = 0;
    if section.characteristics.is_readable() {
        prot |= PROT_READ;
    }
    if section.characteristics.is_writable() {
        prot |= PROT_WRITE;
    }
    if section.characteristics.is_executable() {
        prot |= PROT_EXEC;
    }

    // Phase 1
    mprotect(section.address, section.size, prot)?;

    // Phase 2+
    let flags = PageFlags::from_characteristics(section.characteristics);
    vmm.set_protection(section.address, section.size, flags)?;
}
```

### Step 9: Ready for Execution

At this point, the PE binary is fully loaded and ready to execute.

---

## Memory Management

### Memory Layout (x86_64)

```
Virtual Address Space (48-bit addressing)

0x0000_0000_0000_0000  ┌─────────────────────────────────┐
                       │  Null Page (Unmapped)           │
0x0000_0000_0000_1000  ├─────────────────────────────────┤
                       │  Kernel Code & Data             │
                       │  (Identity Mapped)              │
0x0000_0000_0040_0000  ├─────────────────────────────────┤
                       │  PE Binary Load Region          │
                       │  (Default: ~4MB+)               │
                       │  - PE Headers                   │
                       │  - .text (code)                 │
                       │  - .rdata (read-only data)      │
                       │  - .data (initialized data)     │
                       │  - .bss (uninitialized data)    │
                       │  - ...                          │
0x0000_0000_1000_0000  ├─────────────────────────────────┤
                       │  Heap Region                    │
                       │  (Grows upward)                 │
0x0000_0040_0000_0000  ├─────────────────────────────────┤
                       │  Stack Region                   │
                       │  (Grows downward from top)      │
0x0000_7FFF_FFFF_F000  ├─────────────────────────────────┤
                       │                                 │
                       │  (Gap - Canonical Address)      │
                       │                                 │
0xFFFF_8000_0000_0000  ├─────────────────────────────────┤
                       │  Kernel Higher Half             │
                       │  (Future Use)                   │
0xFFFF_FFFF_8000_0000  ├─────────────────────────────────┤
                       │  MMIO Devices                   │
                       │  - VGA Buffer (0xB8000)         │
                       │  - Serial Ports                 │
0xFFFF_FFFF_FFFF_FFFF  └─────────────────────────────────┘
```

### Physical Memory Manager (PMM)

**Purpose**: Track and allocate physical memory frames (4KB pages).

**Design**: Bitmap allocator (simple and effective for small systems)

**Operations**:
- `allocate_frame()` → PhysAddr
- `free_frame(addr)` → Result<()>
- `allocate_frames(count)` → PhysAddr (contiguous)

**Implementation**:
```rust
pub struct BitmapAllocator {
    bitmap: &'static mut [u8],
    frame_count: usize,
    base_address: PhysAddr,
}

impl BitmapAllocator {
    pub fn allocate_frame(&mut self) -> Option<PhysAddr> {
        for (byte_idx, byte) in self.bitmap.iter_mut().enumerate() {
            if *byte != 0xFF {
                for bit in 0..8 {
                    if (*byte & (1 << bit)) == 0 {
                        *byte |= 1 << bit;
                        let frame_idx = byte_idx * 8 + bit;
                        return Some(self.base_address + (frame_idx * 4096));
                    }
                }
            }
        }
        None
    }
}
```

### Virtual Memory Manager (VMM)

**Purpose**: Manage page tables and virtual-to-physical mappings.

**x86_64 Paging Structure**: 4-level page table
- PML4 (Level 4) → 512 entries, each covers 512 GB
- PDPT (Level 3) → 512 entries, each covers 1 GB
- PD (Level 2) → 512 entries, each covers 2 MB
- PT (Level 1) → 512 entries, each covers 4 KB

**Operations**:
- `map_page(virt, phys, flags)` → Result<()>
- `unmap_page(virt)` → Result<()>
- `translate(virt)` → Option<PhysAddr>

**Page Flags**:
- Present (P): Page is in memory
- Writable (R/W): Page can be written
- User (U/S): Page accessible from user mode
- Execute Disable (XD/NX): Page cannot be executed

**Implementation Sketch**:
```rust
impl PageTableManager {
    pub fn map_page(&mut self, virt: VirtAddr, phys: PhysAddr, flags: PageFlags) -> Result<()> {
        let indices = virt.page_table_indices();

        // Walk page table hierarchy, allocating tables as needed
        let pml4 = self.get_pml4();
        let pdpt = self.get_or_allocate_table(&mut pml4[indices.pml4])?;
        let pd = self.get_or_allocate_table(&mut pdpt[indices.pdpt])?;
        let pt = self.get_or_allocate_table(&mut pd[indices.pd])?;

        // Set page table entry
        pt[indices.pt] = PageTableEntry::new(phys, flags);

        // Flush TLB
        unsafe { asm!("invlpg [{}]", in(reg) virt.as_u64()); }

        Ok(())
    }
}
```

### Heap Allocator

**Purpose**: Provide dynamic memory allocation (`Box`, `Vec`, etc.)

**Approach**: Use `linked_list_allocator` crate (simple, works well for small systems)

**Integration**:
```rust
use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap(heap_start: VirtAddr, heap_size: usize) {
    unsafe {
        ALLOCATOR.lock().init(heap_start.as_mut_ptr(), heap_size);
    }
}
```

**Error Handling**:
```rust
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}
```

---

## Windows API Compatibility

### TEB (Thread Environment Block)

**Purpose**: Per-thread data structure. In Windows, `GS:[0x30]` points to TEB.

**Key Fields**:
```rust
#[repr(C)]
pub struct TEB {
    pub exception_list: *mut ExceptionRegistration,  // +0x00
    pub stack_base: *mut u8,                         // +0x08
    pub stack_limit: *mut u8,                        // +0x10
    pub subsystem_tib: *mut u8,                      // +0x18
    pub fiber_data: *mut u8,                         // +0x20
    pub arbitrary_user_pointer: *mut u8,             // +0x28
    pub teb_address: *mut TEB,                       // +0x30 [Points to itself]
    // ... more fields ...
    pub last_error: u32,                             // Last error code
    // ... more fields ...
    pub peb: *mut PEB,                               // +0x60 [Process Environment Block]
    // ... more fields up to total size ~0x1000 bytes
}
```

**Initialization**:
```rust
pub fn init_teb(peb: *mut PEB, stack_base: *mut u8, stack_limit: *mut u8) -> *mut TEB {
    let teb = allocate_teb();
    unsafe {
        (*teb).teb_address = teb;  // Self-reference for GS:[0x30]
        (*teb).peb = peb;
        (*teb).stack_base = stack_base;
        (*teb).stack_limit = stack_limit;
        (*teb).last_error = 0;
        // ... initialize other fields ...
    }
    teb
}
```

### PEB (Process Environment Block)

**Purpose**: Process-wide data structure. Pointed to by TEB.

**Key Fields**:
```rust
#[repr(C)]
pub struct PEB {
    pub inherited_address_space: u8,
    pub read_image_file_exec_options: u8,
    pub being_debugged: u8,
    pub bit_field: u8,
    pub mutant: HANDLE,
    pub image_base_address: *mut u8,                 // [PE base address]
    pub ldr: *mut PEB_LDR_DATA,                      // [Loader data]
    pub process_parameters: *mut RTL_USER_PROCESS_PARAMETERS,
    pub subsystem_data: *mut u8,
    pub process_heap: HANDLE,                        // [Default heap]
    // ... more fields ...
}
```

**Initialization**:
```rust
pub fn init_peb(image_base: *mut u8, heap: HANDLE) -> *mut PEB {
    let peb = allocate_peb();
    unsafe {
        (*peb).image_base_address = image_base;
        (*peb).process_heap = heap;
        (*peb).being_debugged = 0;
        // ... initialize other fields ...
    }
    peb
}
```

### GS Register Setup

**Purpose**: Make TEB accessible via `GS:[offset]` just like in Windows.

**Implementation**: Use the `IA32_GS_BASE` MSR (Model-Specific Register)

```rust
pub fn set_gs_base(teb_addr: *const TEB) {
    const IA32_GS_BASE: u32 = 0xC0000101;

    unsafe {
        // Write to MSR
        let low = (teb_addr as u64) & 0xFFFF_FFFF;
        let high = (teb_addr as u64) >> 32;

        asm!(
            "wrmsr",
            in("ecx") IA32_GS_BASE,
            in("eax") low as u32,
            in("edx") high as u32,
        );
    }
}
```

**Verification**:
```rust
pub fn test_gs_access() {
    let teb_ptr: u64;
    unsafe {
        asm!("mov {}, gs:[0x30]", out(reg) teb_ptr);
    }
    serial_println!("TEB pointer from GS:[0x30]: 0x{:x}", teb_ptr);
}
```

---

## Execution Model

### Entry Point Jump

After loading the PE and setting up the environment, we jump to the entry point.

**Assembly Trampoline**:
```rust
#[naked]
pub unsafe extern "C" fn jump_to_entry(entry_point: u64, stack_top: u64) -> ! {
    asm!(
        // Set stack pointer
        "mov rsp, rsi",

        // Clear all general-purpose registers
        "xor rax, rax",
        "xor rbx, rbx",
        "xor rcx, rcx",
        "xor rdx, rdx",
        "xor rsi, rsi",
        "xor rdi, rdi",
        "xor rbp, rbp",
        "xor r8, r8",
        "xor r9, r9",
        "xor r10, r10",
        "xor r11, r11",
        "xor r12, r12",
        "xor r13, r13",
        "xor r14, r14",
        "xor r15, r15",

        // Jump to entry point
        "jmp rdi",
        options(noreturn)
    )
}
```

**Call Site**:
```rust
let entry_point = image_base + pe.entry_point_rva;
let stack_top = stack_base + STACK_SIZE;

serial_println!("Jumping to entry point: 0x{:x}", entry_point);
serial_println!("Stack top: 0x{:x}", stack_top);

unsafe {
    jump_to_entry(entry_point, stack_top);
}
```

### Windows x64 Calling Convention

**Parameter Passing**:
- First 4 parameters: RCX, RDX, R8, R9
- Additional parameters: Stack (right to left)
- Return value: RAX

**Stack Requirements**:
- Must be 16-byte aligned before `call`
- Caller must allocate 32 bytes of "shadow space" on stack
- Caller cleans up stack

**Example**: `WriteFile(handle, buffer, size, written, overlapped)`
- RCX = handle
- RDX = buffer
- R8 = size
- R9 = written
- [RSP+32] = overlapped

**Our Implementation**:
```rust
#[no_mangle]
pub extern "C" fn WriteFile(
    handle: HANDLE,          // RCX
    buffer: *const u8,       // RDX
    bytes_to_write: DWORD,   // R8
    bytes_written: *mut DWORD, // R9
    overlapped: *mut u8,     // [RSP+32]
) -> BOOL {
    // Implementation
}
```

The `extern "C"` ensures Rust uses the correct calling convention (which for x86_64 Windows is the Microsoft x64 calling convention).

### API Call Flow

```
[1] PE Binary executes:
    call qword ptr [rip + <IAT offset>]

[2] IAT contains address of our function:
    IAT[offset] = &WriteFile  (our implementation)

[3] CPU jumps to our function:
    WriteFile(rcx=handle, rdx=buffer, r8=size, r9=written, [rsp+32]=overlapped)

[4] Our function executes:
    - Validates parameters
    - Writes to serial port
    - Updates bytes_written
    - Sets last error
    - Returns TRUE

[5] CPU returns to PE binary:
    <next instruction after call>
```

---

## Design Decisions

### 1. Why Rust?

**Pros**:
- Memory safety without garbage collection
- Excellent `no_std` support for bare-metal
- Strong type system catches bugs at compile-time
- Zero-cost abstractions
- Great tooling (cargo, rustup)

**Cons**:
- Learning curve
- Slower compile times
- Some unsafe code required for hardware access

**Decision**: Rust is ideal for this project due to safety and bare-metal support.

### 2. Why goblin for PE parsing?

**Alternatives**:
- `pelite`: PE-specific, lighter, more PE features
- Manual parsing: Complete control, but lots of work

**Decision**: goblin is mature, well-documented, and supports multiple formats (useful for learning/comparison).

### 3. Why Custom Kernel vs Unikraft?

**Unikraft Pros**:
- Mature infrastructure
- Existing drivers and libraries
- Performance optimizations

**Unikraft Cons**:
- Steeper learning curve
- Less control
- Rust bindings are incomplete

**Decision**: Custom kernel provides full control, easier debugging, and educational value.

### 4. Why Phase 1 Userspace Prototype?

**Reasoning**:
- Faster development (no reboot cycle)
- Better debugging tools (GDB, valgrind)
- Can test PE loading logic thoroughly before bare-metal
- Can use standard library during prototyping

**Trade-offs**:
- Some code needs to be rewritten for no_std
- mmap doesn't perfectly match bare-metal VMM

**Decision**: Worth it for the rapid iteration and debugging benefits.

### 5. Why Identity Mapping for Kernel?

**Reasoning**:
- Simplifies address translation (phys == virt for kernel)
- Common approach in OS development
- Makes debugging easier
- Sufficient for a unikernel (no user/kernel separation initially)

**Trade-offs**:
- Less flexible than higher-half kernel
- Limits where PE binary can be loaded

**Decision**: Identity mapping is simpler for initial implementation. Can migrate to higher-half later if needed.

### 6. Why Fake Handles Instead of Real File Descriptors?

**Reasoning**:
- Simpler initial implementation
- No need for full VFS (Virtual File System)
- Sufficient for Phase 4 goal (console I/O)

**Trade-offs**:
- Not realistic Windows behavior
- Limits functionality

**Decision**: Start simple, expand in Phase 5.

---

## Security Considerations

### Current Security Posture

**This is NOT a secure system**. The goal is education and proof-of-concept, not production use.

**Current Vulnerabilities**:
1. **No isolation**: PE binary runs in kernel mode
2. **No validation**: We trust the PE binary
3. **No ASLR**: Fixed load addresses
4. **No DEP enforcement** (initially)
5. **No bounds checking**: Minimal parameter validation
6. **Buffer overflows**: Possible in API functions
7. **No authentication**: Anyone can load any binary

### Future Security Enhancements (Phase 6+)

1. **Privilege Separation**:
   - Run PE binary in Ring 3 (user mode)
   - Use syscalls for API functions
   - Kernel remains in Ring 0

2. **Memory Protection**:
   - Enforce W^X (Write XOR Execute)
   - Implement DEP (Data Execution Prevention)
   - Add ASLR (Address Space Layout Randomization)

3. **Input Validation**:
   - Validate all API parameters
   - Check buffer lengths
   - Verify pointers are in valid ranges

4. **Sandboxing**:
   - Limit file system access
   - Limit network access
   - Restrict system calls

5. **Trusted Boot**:
   - Verify PE signature before loading
   - Check against whitelist

---

## Performance Considerations

### Current Performance

**Phase 1** (Userspace):
- PE loading: <10ms for small binary
- Execution: Near-native speed

**Phase 2+** (Bare-metal):
- Boot time: ~100ms (depends on hardware/QEMU)
- PE loading: ~20ms
- Execution: Near-native speed (minimal overhead)

### Performance Overheads

1. **API Call Overhead**:
   - Single indirect jump via IAT
   - Minimal overhead (~5 cycles)

2. **Memory Allocation**:
   - Depends on allocator (linked_list is simple but not fast)
   - Consider buddy allocator for better performance

3. **Page Faults**:
   - Lazy allocation can add page fault overhead
   - Pre-fault all pages at load time to eliminate this

### Optimization Opportunities (Future)

1. **JIT Compilation**:
   - Recompile PE code for better optimization
   - Eliminate API call indirection

2. **Memory Allocator**:
   - Use faster allocator (buddy, slab)
   - Pre-allocate common sizes

3. **I/O**:
   - Buffer serial output
   - Use DMA for bulk transfers

4. **Caching**:
   - Cache TEB/PEB lookups
   - Cache resolved imports

---

## Environmental Dependencies

### Overview

Windows binaries don't operate in isolation—they expect a rich execution environment. Understanding these dependencies is crucial for determining project scope and implementation priorities.

**See [DEPENDENCIES.md](DEPENDENCIES.md) for comprehensive coverage of all Windows environment expectations.**

### Dependency Hierarchy

```
Console Apps (Simple)
    ↓
Console Apps (Advanced) → Multi-threading, File I/O
    ↓
GUI Apps (Basic) → Window Manager, Graphics, Input
    ↓
GUI Apps (Advanced) → Controls, Dialogs, Fonts
    ↓
Networked Apps → TCP/IP, Sockets
    ↓
Complex Apps → COM, Security, Services
```

### Critical Dependencies by Application Type

#### Console Applications (Phase 4-5)

**Minimal requirements**:
- kernel32.dll core functions (GetStdHandle, WriteFile, ExitProcess)
- Standard handles (stdin, stdout, stderr)
- TEB/PEB structures with basic fields
- Heap allocator
- Environment variables
- Basic file system

**Extended requirements** (Phase 5-6):
- Command line processing
- File I/O (CreateFile, ReadFile, WriteFile, CloseHandle)
- Directory operations
- Time/date functions
- Minimal registry (for configuration queries)
- Multi-threading and synchronization (Phase 6)

#### GUI Applications (Phase 7+)

**Essential additions**:
- user32.dll - Window creation, message loop, input handling
- gdi32.dll - Graphics primitives (lines, rectangles, text, bitmaps)
- Framebuffer access (replace VGA text mode)
- Keyboard and mouse drivers
- Font rendering system
- Window manager (Z-order, clipping, hit testing)

**Advanced GUI** (Phase 8+):
- comctl32.dll - Common controls (buttons, lists, tabs)
- comdlg32.dll - Common dialogs (open/save file, color picker)
- gdiplus.dll - Enhanced graphics (anti-aliasing, alpha blending, image formats)
- Menu system, clipboard, drag-and-drop

### System Services and Background Components

#### Registry

**Why needed**: Configuration storage, file associations, system information

**Implementation strategy**:
1. Phase 5: In-memory fake registry with pre-populated values
2. Phase 6+: Parse real registry hives or maintain persistent registry

**Key registry paths apps query**:
```
HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion → System version
HKLM\SOFTWARE\Classes → File associations
HKCU\Software\<AppName> → Application settings
HKLM\SYSTEM\CurrentControlSet\Control → System config
```

#### File System Structure

Apps expect standard Windows directories:
```
C:\Windows\System32\ → System DLLs
C:\Windows\Fonts\ → Font files
C:\Users\<User>\AppData\Local\ → App data
C:\Windows\Temp\ → Temporary files
```

**Strategy**: Virtual file system that fakes these paths, returning fake metadata for system files.

#### Dynamic Linking

**LoadLibrary/GetProcAddress**: Apps may load DLLs at runtime

**Implementation**:
- Phase 5: Pre-register all our DLL implementations
- Phase 6+: Support dynamic PE loading for additional DLLs

#### Named Objects

**Mutexes, Events, Semaphores**: Used for synchronization and single-instance apps

**Example**: `CreateMutexW(NULL, FALSE, L"Global\\MyAppMutex")`

**Strategy**: Track named objects in kernel data structures (Phase 6)

### Discovery-Driven Development

**Don't implement everything upfront.** Instead:

1. **Run real binaries** and track missing imports
2. **Implement logging stub** that reports which functions are called:
   ```rust
   extern "C" fn missing_api_stub() -> usize {
       serial_println!("STUB: Missing API called from {}", get_return_address());
       0  // Safe default
   }
   ```
3. **Prioritize by frequency**: Implement most-called functions first
4. **Test incrementally**: Start with simple apps, gradually increase complexity

### Compatibility Testing Strategy

**Progressive binary testing** (from simple to complex):

1. **Phase 4-5**: Custom test binaries, simple utilities
2. **Phase 5-6**: busybox-w32 (Unix utilities), 7-zip, curl
3. **Phase 6**: Tiny C Compiler, SQLite
4. **Phase 7**: Simple GUI apps, custom Win32 tests
5. **Phase 8+**: Notepad++, PuTTY, simple games

Each binary will reveal new API requirements, guiding implementation priorities.

### What We Don't Need (Usually)

**Can often be stubbed or omitted**:
- Windows Explorer (desktop shell)
- Windows Event Log (can log to serial instead)
- Windows Management Instrumentation (WMI)
- Background services (BITS, Windows Update, etc.)
- Security/authentication services (unless app specifically needs them)
- Service Control Manager (for Windows services)

### Scope Management

**In scope for this project**:
- ✅ Console applications (full support by Phase 6)
- ✅ Basic GUI applications (Phase 7-8)
- ⚠️ Networked applications (Phase 9+, ambitious)
- ❌ Full Windows compatibility (not a goal)

**Out of scope**:
- ❌ Running arbitrary Windows software
- ❌ Complete API coverage (thousands of functions)
- ❌ Advanced features (COM, security, services) unless needed for target apps
- ❌ Windows desktop environment

### Implementation Priorities

**Must have** (Phase 4-5):
- kernel32.dll core API
- Basic I/O, heap, files
- Minimal registry stub

**Should have** (Phase 6):
- Multi-threading
- Extended file operations
- Dynamic DLL loading
- Full registry simulation

**Nice to have** (Phase 7+):
- GUI support (user32, gdi32)
- Enhanced graphics
- Networking

**Future exploration** (Phase 8+):
- Advanced GUI features
- COM runtime
- More DLLs as needed

---

## Future Directions

### Phase 6: Advanced Windows Features

- **Multi-threading**: `CreateThread`, thread scheduler
- **Synchronization**: Mutexes, semaphores, events
- **Advanced heap**: Multiple heaps, heap debugging
- **Full SEH/VEH**: Structured Exception Handling, Vectored Exception Handling
- **TLS**: Complete Thread Local Storage support
- **DLL Loading**: Load additional DLLs at runtime

### Phase 7: GUI Support (Very Ambitious)

- **user32.dll**: Window creation, message loop
- **GDI32.dll**: Graphics primitives
- **Framebuffer**: Replace VGA text with graphics mode
- **Input**: Keyboard, mouse drivers

### Phase 8: Networking

- **Winsock2 API**: Socket programming
- **TCP/IP Stack**: Implement or port (e.g., smoltcp)
- **Network drivers**: E1000, virtio-net

### Phase 9: File System

- **Real file system**: FAT32, ext2, or custom
- **Virtual file system**: Unified interface
- **File system drivers**: Disk access

### Phase 10: Compatibility Layer Improvements

- **Wine integration**: Study Wine's approach
- **API coverage**: Expand to more DLLs
- **Windows version spoofing**: Report specific Windows version
- **Registry simulation**: Fake registry for apps that need it

---

## Conclusion

This design document outlines the architecture and implementation strategy for the Windows NT Unikernel project. The phased approach allows for incremental development, with each phase building on the previous.

The key insight is that we don't need to emulate all of Windows—just provide compatible API functions. By implementing a shim layer that translates Windows API calls to kernel operations, we can execute Windows binaries with minimal overhead.

**Current Status**: Phase 0 complete, ready for Phase 1 implementation.

**Next Steps**: Implement Phase 1 (userspace PE loader prototype) as detailed in [PHASE1.md](PHASE1.md).

---

## References

- [Microsoft PE Format Specification](https://learn.microsoft.com/en-us/windows/win32/debug/pe-format)
- [Windows x64 Calling Convention](https://learn.microsoft.com/en-us/cpp/build/x64-calling-convention)
- [Intel x86_64 Software Developer Manual](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html)
- [ReactOS Source Code](https://github.com/reactos/reactos)
- [Wine Source Code](https://gitlab.winehq.org/wine/wine)
- [phil-opp's Writing an OS in Rust](https://os.phil-opp.com/)
- [OSDev Wiki](https://wiki.osdev.org/)
