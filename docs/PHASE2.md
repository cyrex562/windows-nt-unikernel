# Phase 2: Unikernel Foundation - Detailed Task Checklist

**Goal**: Port the PE loader to a bare-metal no_std environment and establish the foundational kernel infrastructure.

**Location**: `crates/kernel/`

**Success Criteria**: Boot a minimal kernel in QEMU, display output via serial/VGA, and successfully load a PE binary from an embedded initrd.

---

## 1. Kernel Bootstrap and Boot Process

### 1.1 Bootloader Integration
- [ ] **1.1.1** Choose bootloader approach:
  - [ ] Option A: `bootloader` crate (Recommended)
  - [ ] Option B: `limine` bootloader
  - [ ] Option C: Custom bootloader (Advanced)
- [ ] **1.1.2** Configure bootloader in `kernel/Cargo.toml`
- [ ] **1.1.3** Create boot configuration file
- [ ] **1.1.4** Set up kernel entry point

### 1.2 Kernel Entry Point
**File**: `crates/kernel/src/main.rs`

- [ ] **1.2.1** Mark kernel as `#![no_std]` and `#![no_main]`
- [ ] **1.2.2** Implement `_start()` function
- [ ] **1.2.3** Accept bootloader information struct
- [ ] **1.2.4** Initialize basic output (serial or VGA)
- [ ] **1.2.5** Print "Kernel boot" message
- [ ] **1.2.6** Parse bootloader-provided memory map
- [ ] **1.2.7** Initialize kernel heap
- [ ] **1.2.8** Jump to `kernel_main()`

### 1.3 Panic Handler
- [ ] **1.3.1** Implement `#[panic_handler]` function
- [ ] **1.3.2** Print panic message to serial
- [ ] **1.3.3** Print panic location (file, line)
- [ ] **1.3.4** Halt CPU with `hlt` loop
- [ ] **1.3.5** Optionally trigger QEMU shutdown

### 1.4 Build System Integration
- [ ] **1.4.1** Create `.cargo/config.toml` for kernel
- [ ] **1.4.2** Configure target triple: `x86_64-unknown-none`
- [ ] **1.4.3** Set up build-std for core and alloc
- [ ] **1.4.4** Configure linker script (if needed)
- [ ] **1.4.5** Update justfile with kernel build commands
- [ ] **1.4.6** Test kernel builds successfully

---

## 2. Hardware Initialization

### 2.1 GDT (Global Descriptor Table)
**File**: `crates/kernel/src/gdt.rs`

- [ ] **2.1.1** Define GDT structure:
  ```rust
  struct Gdt {
      null: Descriptor,
      code: Descriptor,
      data: Descriptor,
      tss: Descriptor,
  }
  ```
- [ ] **2.1.2** Implement descriptor creation functions
- [ ] **2.1.3** Create null descriptor (index 0)
- [ ] **2.1.4** Create code segment descriptor (index 1)
  - [ ] Set as 64-bit code segment
  - [ ] Set DPL (privilege level) = 0 (kernel mode)
  - [ ] Set present bit
- [ ] **2.1.5** Create data segment descriptor (index 2)
  - [ ] Set as 64-bit data segment
  - [ ] Set DPL = 0
- [ ] **2.1.6** Create TSS (Task State Segment) descriptor
  - [ ] Allocate TSS structure
  - [ ] Set TSS address in descriptor
- [ ] **2.1.7** Implement `lgdt` instruction wrapper
- [ ] **2.1.8** Load GDT with `lgdt`
- [ ] **2.1.9** Reload segment registers (CS, DS, SS, ES, FS, GS)
- [ ] **2.1.10** Load TSS with `ltr` instruction
- [ ] **2.1.11** Log GDT initialization

### 2.2 IDT (Interrupt Descriptor Table)
**File**: `crates/kernel/src/interrupts/mod.rs`

- [ ] **2.2.1** Define IDT structure (256 entries)
- [ ] **2.2.2** Create interrupt gate descriptor type
- [ ] **2.2.3** Implement exception handlers:
  - [ ] Divide by zero (#DE)
  - [ ] Debug (#DB)
  - [ ] Non-maskable interrupt (#NMI)
  - [ ] Breakpoint (#BP)
  - [ ] Overflow (#OF)
  - [ ] Bound range exceeded (#BR)
  - [ ] Invalid opcode (#UD)
  - [ ] Device not available (#NM)
  - [ ] Double fault (#DF) **[CRITICAL]**
  - [ ] Invalid TSS (#TS)
  - [ ] Segment not present (#NP)
  - [ ] Stack-segment fault (#SS)
  - [ ] General protection fault (#GP) **[CRITICAL]**
  - [ ] Page fault (#PF) **[CRITICAL]**
  - [ ] x87 FPU error (#MF)
  - [ ] Alignment check (#AC)
  - [ ] Machine check (#MC)
  - [ ] SIMD exception (#XM)
- [ ] **2.2.4** Implement default handler for unhandled interrupts
- [ ] **2.2.5** Set up interrupt stack frame structure
- [ ] **2.2.6** Implement `lidt` instruction wrapper
- [ ] **2.2.7** Load IDT with `lidt`
- [ ] **2.2.8** Enable interrupts with `sti` (after setup)
- [ ] **2.2.9** Test exception handling (trigger breakpoint)
- [ ] **2.2.10** Log IDT initialization

### 2.3 Serial Port Driver (Enhanced)
**File**: `crates/kernel/src/serial.rs`

- [ ] **2.3.1** Initialize UART 16550 on COM1 (0x3F8)
- [ ] **2.3.2** Configure baud rate (38400 or 115200)
- [ ] **2.3.3** Configure data bits, parity, stop bits
- [ ] **2.3.4** Implement write_byte function
- [ ] **2.3.5** Implement write_str function
- [ ] **2.3.6** Implement fmt::Write trait
- [ ] **2.3.7** Create global SERIAL1 static
- [ ] **2.3.8** Add serial_print! and serial_println! macros
- [ ] **2.3.9** Add mutex protection for thread-safety
- [ ] **2.3.10** Test serial output in QEMU

### 2.4 VGA Text Buffer (Enhanced)
**File**: `crates/kernel/src/vga_buffer.rs`

- [ ] **2.4.1** Verify existing VGA buffer implementation
- [ ] **2.4.2** Add color customization
- [ ] **2.4.3** Add cursor position tracking
- [ ] **2.4.4** Implement scrolling
- [ ] **2.4.5** Implement clear screen
- [ ] **2.4.6** Add write! and writeln! support
- [ ] **2.4.7** Create global WRITER static
- [ ] **2.4.8** Add println! and print! macros
- [ ] **2.4.9** Test VGA output in QEMU

---

## 3. Memory Management

### 3.1 Physical Memory Manager (PMM)
**File**: `crates/kernel/src/memory/physical.rs`

- [ ] **3.1.1** Design PMM data structure:
  - [ ] Option A: Bitmap allocator (simple, fixed size)
  - [ ] Option B: Buddy allocator (efficient, variable size)
  - [ ] Option C: Stack allocator (very simple, LIFO)
- [ ] **3.1.2** Parse bootloader memory map
- [ ] **3.1.3** Identify usable memory regions
- [ ] **3.1.4** Initialize PMM with available memory
- [ ] **3.1.5** Implement `allocate_frame() -> Option<PhysicalAddress>`
- [ ] **3.1.6** Implement `free_frame(addr: PhysicalAddress)`
- [ ] **3.1.7** Implement `allocate_frames(count: usize) -> Option<PhysicalAddress>`
- [ ] **3.1.8** Track used and free frames
- [ ] **3.1.9** Handle out-of-memory conditions
- [ ] **3.1.10** Add debug statistics (total, used, free)
- [ ] **3.1.11** Test frame allocation and deallocation
- [ ] **3.1.12** Log PMM initialization

### 3.2 Virtual Memory Manager (VMM)
**File**: `crates/kernel/src/memory/virtual.rs`

- [ ] **3.2.1** Understand x86_64 paging structure:
  - [ ] PML4 (Level 4 page table)
  - [ ] PDPT (Page Directory Pointer Table)
  - [ ] PD (Page Directory)
  - [ ] PT (Page Table)
- [ ] **3.2.2** Define page table structures
- [ ] **3.2.3** Define page table entry (PTE) flags:
  - [ ] Present (P)
  - [ ] Read/Write (R/W)
  - [ ] User/Supervisor (U/S)
  - [ ] Execute Disable (XD/NX)
  - [ ] Accessed (A)
  - [ ] Dirty (D)
- [ ] **3.2.4** Implement page table walking
- [ ] **3.2.5** Implement `map_page(virt: VirtualAddress, phys: PhysicalAddress, flags: PageFlags)`
- [ ] **3.2.6** Implement `unmap_page(virt: VirtualAddress)`
- [ ] **3.2.7** Implement `translate_address(virt: VirtualAddress) -> Option<PhysicalAddress>`
- [ ] **3.2.8** Handle page table allocation (use PMM)
- [ ] **3.2.9** Flush TLB after page table changes
- [ ] **3.2.10** Create kernel page table
- [ ] **3.2.11** Identity map kernel code and data
- [ ] **3.2.12** Map MMIO regions (VGA, serial, etc.)
- [ ] **3.2.13** Test page mapping and translation
- [ ] **3.2.14** Log VMM initialization

### 3.3 Heap Allocator
**File**: `crates/kernel/src/memory/heap.rs`

- [ ] **3.3.1** Choose heap allocator:
  - [ ] Option A: linked_list_allocator crate (simple)
  - [ ] Option B: buddy_system_allocator crate (efficient)
  - [ ] Option C: Custom allocator
- [ ] **3.3.2** Define heap region (start address and size)
- [ ] **3.3.3** Map heap pages in VMM
- [ ] **3.3.4** Initialize chosen allocator
- [ ] **3.3.5** Implement `#[global_allocator]`
- [ ] **3.3.6** Implement `#[alloc_error_handler]`
- [ ] **3.3.7** Test heap allocation with Box, Vec
- [ ] **3.3.8** Add heap growth support (optional)
- [ ] **3.3.9** Log heap initialization

### 3.4 Memory Layout Planning
**File**: `crates/kernel/src/memory/layout.rs`

- [ ] **3.4.1** Define kernel memory layout:
  ```
  0x0000_0000_0000_0000 - Null page (unmapped)
  0x0000_0000_0000_1000 - Kernel code/data (identity mapped)
  0x0000_7FFF_FFFF_F000 - End of lower half
  0xFFFF_8000_0000_0000 - Start of higher half (kernel)
  0xFFFF_FFFF_FFFF_FFFF - End of address space
  ```
- [ ] **3.4.2** Define PE binary load region (e.g., 0x0000_0000_0040_0000)
- [ ] **3.4.3** Define heap region
- [ ] **3.4.4** Define stack region(s)
- [ ] **3.4.5** Document memory layout
- [ ] **3.4.6** Create constants for regions
- [ ] **3.4.7** Validate regions don't overlap

---

## 4. PE Loader Integration (no_std Port)

### 4.1 Create no_std PE Loader
**File**: `crates/kernel/src/pe_loader/mod.rs`

- [ ] **4.1.1** Copy Phase 1 loader code to kernel
- [ ] **4.1.2** Remove std dependencies:
  - [ ] Replace `std::fs` → embedded binary access
  - [ ] Replace `std::vec::Vec` → `alloc::vec::Vec`
  - [ ] Replace `std::collections` → `alloc::collections`
  - [ ] Replace `std::string::String` → `alloc::string::String`
- [ ] **4.1.3** Replace anyhow with custom Result types
- [ ] **4.1.4** Replace logging with serial/VGA output
- [ ] **4.1.5** Port memory mapping to use VMM instead of mmap
- [ ] **4.1.6** Port relocations (should work unchanged)
- [ ] **4.1.7** Port imports (should work unchanged)
- [ ] **4.1.8** Test compilation in no_std environment

### 4.2 Memory Mapping for PE (Kernel Version)
**File**: `crates/kernel/src/pe_loader/memory.rs`

- [ ] **4.2.1** Implement `allocate_image_base(size: usize, preferred_base: u64) -> Result<*mut u8>`
  - [ ] Use VMM to allocate virtual memory region
  - [ ] Allocate physical frames from PMM
  - [ ] Map virtual pages to physical frames
  - [ ] Return virtual address
- [ ] **4.2.2** Implement section mapping using VMM
- [ ] **4.2.3** Implement protection setting using page flags:
  - [ ] Executable → Clear NX bit
  - [ ] Writable → Set R/W bit
  - [ ] Readable → Set Present bit
- [ ] **4.2.4** Validate all mappings
- [ ] **4.2.5** Log memory operations

### 4.3 PE Binary Embedding
**File**: `crates/kernel/src/pe_loader/embedded.rs`

- [ ] **4.3.1** Use `include_bytes!` to embed target-zero.exe:
  ```rust
  static TARGET_BINARY: &[u8] = include_bytes!("../../../target-zero/target-zero.exe");
  ```
- [ ] **4.3.2** Create accessor function for embedded binary
- [ ] **4.3.3** Validate binary is embedded correctly
- [ ] **4.3.4** Add multiple binary support (optional)
- [ ] **4.3.5** Log embedded binary size

---

## 5. Windows Execution Environment

### 5.1 TEB/PEB Structures (Kernel Version)
**File**: `crates/kernel/src/windows_compat/teb_peb.rs`

- [ ] **5.1.1** Allocate TEB in kernel heap
- [ ] **5.1.2** Allocate PEB in kernel heap
- [ ] **5.1.3** Initialize TEB fields (same as Phase 1)
- [ ] **5.1.4** Initialize PEB fields (same as Phase 1)
- [ ] **5.1.5** Link TEB → PEB
- [ ] **5.1.6** Store TEB pointer globally (thread-local in future)

### 5.2 GS Register Setup for TEB
**File**: `crates/kernel/src/windows_compat/gs_setup.rs`

- [ ] **5.2.1** Understand IA32_GS_BASE MSR (0xC0000101)
- [ ] **5.2.2** Implement MSR write function:
  ```rust
  unsafe fn write_msr(msr: u32, value: u64) {
      asm!(
          "wrmsr",
          in("ecx") msr,
          in("eax") (value & 0xFFFF_FFFF) as u32,
          in("edx") (value >> 32) as u32,
      );
  }
  ```
- [ ] **5.2.3** Write TEB address to GS_BASE
- [ ] **5.2.4** Verify GS:[0x30] points to TEB
- [ ] **5.2.5** Test GS access (read back TEB pointer)
- [ ] **5.2.6** Log GS setup

### 5.3 Stack Setup (Kernel Version)
**File**: `crates/kernel/src/windows_compat/stack.rs`

- [ ] **5.3.1** Allocate stack pages using VMM
- [ ] **5.3.2** Map stack with R/W permissions
- [ ] **5.3.3** Set up guard page at bottom (optional)
- [ ] **5.3.4** Calculate stack top (ensure 16-byte alignment)
- [ ] **5.3.5** Store stack pointer for execution
- [ ] **5.3.6** Log stack setup

---

## 6. API Shim Integration

### 6.1 Kernel-Compatible API Shim
**File**: `crates/kernel/src/api_shim/mod.rs`

- [ ] **6.1.1** Import api-shim crate with no_std feature
- [ ] **6.1.2** Update WriteFile to use serial output:
  ```rust
  pub extern "C" fn WriteFile(...) -> BOOL {
      // Write to serial port instead of stdout
      serial_println!("{}", String::from_utf8_lossy(buffer_slice));
      ...
  }
  ```
- [ ] **6.1.3** Update ExitProcess to halt kernel:
  ```rust
  pub extern "C" fn ExitProcess(code: u32) -> ! {
      serial_println!("Process exiting with code: {}", code);
      loop { x86_64::instructions::hlt(); }
  }
  ```
- [ ] **6.1.4** Update GetStdHandle (no changes needed)
- [ ] **6.1.5** Update GetLastError/SetLastError for no_std
- [ ] **6.1.6** Register all API functions in symbol resolver
- [ ] **6.1.7** Test API functions independently

---

## 7. QEMU Integration and Testing

### 7.1 QEMU Configuration
**File**: Update `justfile`

- [ ] **7.1.1** Add QEMU run command:
  ```just
  run-qemu:
      qemu-system-x86_64 \
          -drive format=raw,file=target/kernel.bin \
          -serial stdio \
          -display none \
          -no-reboot \
          -no-shutdown
  ```
- [ ] **7.1.2** Configure serial output redirection
- [ ] **7.1.3** Add QEMU debug flags (optional)
- [ ] **7.1.4** Add QEMU exit device for automated testing
- [ ] **7.1.5** Test QEMU launches successfully

### 7.2 Kernel Boot Test
- [ ] **7.2.1** Build kernel
- [ ] **7.2.2** Boot kernel in QEMU
- [ ] **7.2.3** Verify boot message appears
- [ ] **7.2.4** Verify serial output works
- [ ] **7.2.5** Verify VGA output works (if using graphics)
- [ ] **7.2.6** Log successful boot

### 7.3 Memory Manager Test
- [ ] **7.3.1** Test PMM allocation
- [ ] **7.3.2** Test VMM page mapping
- [ ] **7.3.3** Test heap allocation (Box, Vec)
- [ ] **7.3.4** Verify no panics
- [ ] **7.3.5** Log test results

### 7.4 PE Loader Test (Simple)
- [ ] **7.4.1** Load embedded target-zero.exe
- [ ] **7.4.2** Parse PE headers
- [ ] **7.4.3** Map sections
- [ ] **7.4.4** Apply relocations
- [ ] **7.4.5** Resolve imports
- [ ] **7.4.6** Log success (don't execute yet)

---

## 8. Full Integration

### 8.1 Complete Boot-to-Load Pipeline
**File**: `crates/kernel/src/main.rs`

- [ ] **8.1.1** Boot kernel
- [ ] **8.1.2** Initialize GDT
- [ ] **8.1.3** Initialize IDT
- [ ] **8.1.4** Initialize serial/VGA
- [ ] **8.1.5** Initialize PMM
- [ ] **8.1.6** Initialize VMM
- [ ] **8.1.7** Initialize heap
- [ ] **8.1.8** Set up TEB/PEB
- [ ] **8.1.9** Set up GS register
- [ ] **8.1.10** Load embedded PE binary
- [ ] **8.1.11** Parse and map PE binary
- [ ] **8.1.12** Apply relocations
- [ ] **8.1.13** Resolve imports
- [ ] **8.1.14** Log completion
- [ ] **8.1.15** Halt (execution in Phase 4)

### 8.2 Error Handling
- [ ] **8.2.1** Add error handling for each initialization step
- [ ] **8.2.2** Print detailed error messages
- [ ] **8.2.3** Halt on critical errors
- [ ] **8.2.4** Add debug output at each step

### 8.3 Validation
- [ ] **8.3.1** Verify all initialization steps succeed
- [ ] **8.3.2** Verify PE binary loads correctly
- [ ] **8.3.3** Verify memory layout is correct
- [ ] **8.3.4** Verify imports are resolved
- [ ] **8.3.5** Log validation results

---

## 9. Documentation and Testing

### 9.1 Documentation
- [ ] **9.1.1** Document kernel architecture
- [ ] **9.1.2** Document memory layout
- [ ] **9.1.3** Document boot process
- [ ] **9.1.4** Document PE loading in kernel
- [ ] **9.1.5** Add inline code comments

### 9.2 Testing
- [ ] **9.2.1** Create automated boot tests
- [ ] **9.2.2** Test kernel boots consistently
- [ ] **9.2.3** Test memory managers work correctly
- [ ] **9.2.4** Test PE loader works in kernel
- [ ] **9.2.5** Add regression tests

### 9.3 Debugging Support
- [ ] **9.3.1** Add GDB support in QEMU
- [ ] **9.3.2** Create debug symbols
- [ ] **9.3.3** Add breakpoint support
- [ ] **9.3.4** Test debugging workflow

---

## Success Criteria Checklist

Phase 2 is complete when:

- [ ] **S1** Kernel boots successfully in QEMU
- [ ] **S2** Serial output works
- [ ] **S3** VGA output works
- [ ] **S4** GDT is initialized and loaded
- [ ] **S5** IDT is initialized and loaded
- [ ] **S6** Exception handling works (test with breakpoint)
- [ ] **S7** Physical memory manager allocates frames
- [ ] **S8** Virtual memory manager maps pages
- [ ] **S9** Heap allocator works (Box, Vec)
- [ ] **S10** PE binary loads from embedded data
- [ ] **S11** PE headers parse correctly
- [ ] **S12** PE sections map to memory
- [ ] **S13** PE relocations apply correctly
- [ ] **S14** PE imports resolve to kernel API functions
- [ ] **S15** TEB/PEB structures are initialized
- [ ] **S16** GS register points to TEB
- [ ] **S17** No panics or crashes during boot
- [ ] **S18** All initialization steps log successfully
- [ ] **S19** Ready for Phase 3 (execution)

---

## Estimated Task Breakdown

| Section | Tasks | Estimated Complexity |
|---------|-------|---------------------|
| 1. Bootstrap | 20 | Medium |
| 2. Hardware Init | 50 | High |
| 3. Memory Management | 60 | High |
| 4. PE Loader Port | 30 | Medium |
| 5. Windows Environment | 20 | Medium |
| 6. API Shim | 10 | Low |
| 7. QEMU Testing | 15 | Medium |
| 8. Integration | 20 | High |
| 9. Documentation | 15 | Low |
| **Total** | **~240 tasks** | **High** |

---

## Next Steps

After Phase 2 is complete:
1. Move to Phase 3: Set up complete Windows execution context
2. Implement entry point jump
3. Execute loaded PE binary

See [PHASE3.md](PHASE3.md) for next phase details.
