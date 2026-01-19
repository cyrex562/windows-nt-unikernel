# Phase 3: Windows Execution Context - Detailed Task Checklist

**Goal**: Complete the Windows execution environment to make the loaded PE binary believe it's running on Windows, then jump to its entry point.

**Location**: `crates/kernel/src/windows_compat/`

**Success Criteria**: Successfully jump to target-zero.exe's entry point with a fully initialized Windows-compatible execution context.

---

## 1. Enhanced TEB/PEB Implementation

### 1.1 Complete TEB Structure
**File**: `crates/common/src/windows.rs` and `crates/kernel/src/windows_compat/teb.rs`

- [ ] **1.1.1** Expand TEB structure with additional fields:
  ```rust
  #[repr(C)]
  pub struct TEB {
      pub exception_list: *mut ExceptionRegistration,  // Offset 0x00
      pub stack_base: *mut u8,                         // Offset 0x08
      pub stack_limit: *mut u8,                        // Offset 0x10
      pub subsystem_tib: *mut u8,                      // Offset 0x18
      pub fiber_data: *mut u8,                         // Offset 0x20
      pub arbitrary_user_pointer: *mut u8,             // Offset 0x28
      pub teb_address: *mut TEB,                       // Offset 0x30 [CRITICAL]
      // ... additional fields
      pub last_error: u32,                             // Thread last error
      // ... more fields up to PEB pointer at 0x60
      pub peb: *mut PEB,                               // Offset 0x60 [CRITICAL]
  }
  ```
- [ ] **1.1.2** Initialize all TEB fields with appropriate values
- [ ] **1.1.3** Set `teb_address` to point to itself (GS:[0x30])
- [ ] **1.1.4** Set `peb` pointer (GS:[0x60])
- [ ] **1.1.5** Set stack_base and stack_limit
- [ ] **1.1.6** Verify field offsets match Windows ABI
- [ ] **1.1.7** Add helper functions for TEB access
- [ ] **1.1.8** Test TEB structure size and alignment

### 1.2 Complete PEB Structure
**File**: `crates/common/src/windows.rs` and `crates/kernel/src/windows_compat/peb.rs`

- [ ] **1.2.1** Expand PEB structure:
  ```rust
  #[repr(C)]
  pub struct PEB {
      pub inherited_address_space: u8,
      pub read_image_file_exec_options: u8,
      pub being_debugged: u8,                          // Set to 0
      pub bit_field: u8,
      pub mutant: HANDLE,
      pub image_base_address: *mut u8,                 // [CRITICAL]
      pub ldr: *mut PEB_LDR_DATA,
      pub process_parameters: *mut RTL_USER_PROCESS_PARAMETERS,
      pub subsystem_data: *mut u8,
      pub process_heap: HANDLE,                        // [CRITICAL]
      pub fast_peb_lock: *mut u8,
      // ... more fields
      pub session_id: u32,
  }
  ```
- [ ] **1.2.2** Initialize all PEB fields
- [ ] **1.2.3** Set image_base_address to PE load address
- [ ] **1.2.4** Set process_heap (will implement in Phase 5)
- [ ] **1.2.5** Allocate and initialize PEB_LDR_DATA
- [ ] **1.2.6** Allocate and initialize RTL_USER_PROCESS_PARAMETERS
- [ ] **1.2.7** Verify field offsets match Windows ABI
- [ ] **1.2.8** Test PEB structure size and alignment

### 1.3 PEB_LDR_DATA Structure
**File**: `crates/common/src/windows.rs`

- [ ] **1.3.1** Define PEB_LDR_DATA structure:
  ```rust
  #[repr(C)]
  pub struct PEB_LDR_DATA {
      pub length: u32,
      pub initialized: u32,
      pub ss_handle: *mut u8,
      pub in_load_order_module_list: LIST_ENTRY,
      pub in_memory_order_module_list: LIST_ENTRY,
      pub in_initialization_order_module_list: LIST_ENTRY,
  }
  ```
- [ ] **1.3.2** Initialize loader data
- [ ] **1.3.3** Create module list entries (empty for now)
- [ ] **1.3.4** Link PEB to PEB_LDR_DATA

### 1.4 RTL_USER_PROCESS_PARAMETERS Structure
**File**: `crates/common/src/windows.rs`

- [ ] **1.4.1** Expand RTL_USER_PROCESS_PARAMETERS:
  ```rust
  #[repr(C)]
  pub struct RTL_USER_PROCESS_PARAMETERS {
      pub maximum_length: u32,
      pub length: u32,
      pub flags: u32,
      pub debug_flags: u32,
      pub console_handle: HANDLE,
      pub console_flags: u32,
      pub standard_input: HANDLE,
      pub standard_output: HANDLE,
      pub standard_error: HANDLE,
      pub current_directory: CURDIR,
      pub dll_path: UNICODE_STRING,
      pub image_path_name: UNICODE_STRING,
      pub command_line: UNICODE_STRING,
      pub environment: *mut u16,
      // ... more fields
  }
  ```
- [ ] **1.4.2** Initialize all fields
- [ ] **1.4.3** Set standard handles (stdin, stdout, stderr)
- [ ] **1.4.4** Set command line (basic for now, expand in Phase 5)
- [ ] **1.4.5** Set image path name
- [ ] **1.4.6** Link to PEB

---

## 2. GDT Enhancements for Windows Compatibility

### 2.1 User Mode Segments
**File**: `crates/kernel/src/gdt.rs`

- [ ] **2.1.1** Add user code segment descriptor (DPL = 3)
- [ ] **2.1.2** Add user data segment descriptor (DPL = 3)
- [ ] **2.1.3** Configure for 64-bit mode
- [ ] **2.1.4** Reload GDT with new descriptors
- [ ] **2.1.5** Note: For Phase 3/4, we'll stay in kernel mode (DPL 0)
- [ ] **2.1.6** User mode segments prepared for future expansion

### 2.2 TSS (Task State Segment) Setup
**File**: `crates/kernel/src/gdt.rs`

- [ ] **2.2.1** Define TSS structure:
  ```rust
  #[repr(C, packed)]
  pub struct TaskStateSegment {
      reserved_1: u32,
      pub privilege_stack_table: [VirtAddr; 3],  // RSP0, RSP1, RSP2
      reserved_2: u64,
      pub interrupt_stack_table: [VirtAddr; 7],  // IST1-IST7
      reserved_3: u64,
      reserved_4: u16,
      pub iomap_base: u16,
  }
  ```
- [ ] **2.2.2** Allocate and initialize TSS
- [ ] **2.2.3** Set up privilege stack table (RSP0 for kernel)
- [ ] **2.2.4** Set up interrupt stack table (for double fault)
- [ ] **2.2.5** Load TSS into GDT
- [ ] **2.2.6** Execute `ltr` instruction
- [ ] **2.2.7** Test TSS is loaded correctly

### 2.3 FS and GS Segment Setup
**File**: `crates/kernel/src/windows_compat/segments.rs`

- [ ] **2.3.1** Implement FS base setup (currently unused)
- [ ] **2.3.2** Implement GS base setup for TEB:
  ```rust
  pub fn set_gs_base(teb_addr: *const TEB) {
      unsafe {
          Msr::new(IA32_GS_BASE).write(teb_addr as u64);
      }
  }
  ```
- [ ] **2.3.3** Verify GS:[0x30] reads back TEB address
- [ ] **2.3.4** Verify GS:[0x60] reads back PEB address
- [ ] **2.3.5** Test with inline assembly:
  ```rust
  let teb_ptr: u64;
  unsafe {
      asm!("mov {}, gs:[0x30]", out(reg) teb_ptr);
  }
  assert_eq!(teb_ptr, teb_addr as u64);
  ```
- [ ] **2.3.6** Log GS base setup

---

## 3. Stack Setup and Management

### 3.1 Main Thread Stack
**File**: `crates/kernel/src/windows_compat/stack.rs`

- [ ] **3.1.1** Define stack size (e.g., 1 MB = 0x100000)
- [ ] **3.1.2** Allocate stack pages using VMM
- [ ] **3.1.3** Map stack with PROT_READ | PROT_WRITE
- [ ] **3.1.4** Calculate stack base (high address)
- [ ] **3.1.5** Calculate stack limit (low address)
- [ ] **3.1.6** Set up guard page at bottom (map without write permissions)
- [ ] **3.1.7** Update TEB with stack_base and stack_limit
- [ ] **3.1.8** Ensure stack pointer is 16-byte aligned
- [ ] **3.1.9** Log stack allocation

### 3.2 Stack Frame Preparation
**File**: `crates/kernel/src/windows_compat/stack.rs`

- [ ] **3.2.1** Understand Windows x64 calling convention:
  - [ ] Parameters: RCX, RDX, R8, R9, then stack
  - [ ] Caller must allocate 32 bytes shadow space
  - [ ] Stack must be 16-byte aligned before call
- [ ] **3.2.2** Prepare initial stack frame:
  ```
  [Stack Top]
  +0x00: Return address (set to exit handler)
  +0x08: Shadow space (32 bytes)
  +0x28: [Additional parameters if needed]
  ```
- [ ] **3.2.3** Set return address to ExitProcess wrapper
- [ ] **3.2.4** Allocate shadow space (32 bytes)
- [ ] **3.2.5** Ensure alignment
- [ ] **3.2.6** Calculate final RSP value

### 3.3 Exception Stack (IST)
**File**: `crates/kernel/src/interrupts/mod.rs`

- [ ] **3.3.1** Allocate separate stack for double fault handler
- [ ] **3.3.2** Configure IST entry in TSS
- [ ] **3.3.3** Update double fault IDT entry to use IST
- [ ] **3.3.4** Test exception handling still works

---

## 4. Register Initialization

### 4.1 General Purpose Registers
**File**: `crates/kernel/src/windows_compat/execution.rs`

- [ ] **4.1.1** Prepare register state structure:
  ```rust
  pub struct RegisterState {
      pub rax: u64,
      pub rbx: u64,
      pub rcx: u64,
      pub rdx: u64,
      pub rsi: u64,
      pub rdi: u64,
      pub rbp: u64,
      pub rsp: u64,
      pub r8: u64,
      pub r9: u64,
      pub r10: u64,
      pub r11: u64,
      pub r12: u64,
      pub r13: u64,
      pub r14: u64,
      pub r15: u64,
      pub rip: u64,
      pub rflags: u64,
  }
  ```
- [ ] **4.1.2** Initialize all registers to zero
- [ ] **4.1.3** Set RSP to prepared stack top
- [ ] **4.1.4** Set RIP to PE entry point
- [ ] **4.1.5** Set RFLAGS (enable interrupts: IF bit)
- [ ] **4.1.6** RCX, RDX, R8, R9: Set to 0 (no parameters for now)

### 4.2 Segment Registers
**File**: `crates/kernel/src/windows_compat/execution.rs`

- [ ] **4.2.1** Set CS to kernel code segment
- [ ] **4.2.2** Set DS, ES, SS to kernel data segment
- [ ] **4.2.3** Set FS to 0 (unused)
- [ ] **4.2.4** Verify GS points to TEB (already set)

### 4.3 Control Registers
**File**: `crates/kernel/src/windows_compat/execution.rs`

- [ ] **4.3.1** Verify CR0 settings (paging enabled, etc.)
- [ ] **4.3.2** Verify CR3 points to correct page table
- [ ] **4.3.3** Verify CR4 settings (PAE, PSE, etc.)
- [ ] **4.3.4** No changes needed, just validation

### 4.4 Model-Specific Registers (MSRs)
**File**: `crates/kernel/src/windows_compat/execution.rs`

- [ ] **4.4.1** Verify IA32_GS_BASE is set to TEB
- [ ] **4.4.2** Verify IA32_KERNEL_GS_BASE (for kernel)
- [ ] **4.4.3** Verify IA32_EFER (Long Mode enabled)
- [ ] **4.4.4** Verify SYSCALL MSRs (for future use)

---

## 5. Entry Point Jump Mechanism

### 5.1 Context Switch Implementation
**File**: `crates/kernel/src/windows_compat/execution.rs`

- [ ] **5.1.1** Implement assembly trampoline:
  ```rust
  #[naked]
  pub unsafe extern "C" fn jump_to_entry(entry_point: u64, stack_top: u64) -> ! {
      asm!(
          // Set up stack
          "mov rsp, {stack}",
          // Clear all general purpose registers
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
          "jmp {entry}",
          entry = in(reg) entry_point,
          stack = in(reg) stack_top,
          options(noreturn)
      )
  }
  ```
- [ ] **5.1.2** Ensure function is marked as `#[naked]`
- [ ] **5.1.3** Ensure function never returns
- [ ] **5.1.4** Test assembly compiles correctly

### 5.2 Pre-Jump Validation
**File**: `crates/kernel/src/windows_compat/execution.rs`

- [ ] **5.2.1** Validate entry point address:
  - [ ] Is within .text section
  - [ ] Is executable (page permissions)
  - [ ] Is properly aligned
- [ ] **5.2.2** Validate stack pointer:
  - [ ] Is within allocated stack
  - [ ] Is 16-byte aligned
  - [ ] Has sufficient space
- [ ] **5.2.3** Validate TEB/PEB are set up
- [ ] **5.2.4** Validate imports are resolved
- [ ] **5.2.5** Validate GS register is set
- [ ] **5.2.6** Log all validations

### 5.3 Post-Jump Handling
**File**: `crates/kernel/src/windows_compat/execution.rs`

- [ ] **5.3.1** Set up return address on stack to exit handler:
  ```rust
  pub extern "C" fn exit_handler() -> ! {
      serial_println!("PE binary returned to kernel (should not happen)");
      unsafe { ExitProcess(0); }
  }
  ```
- [ ] **5.3.2** Push return address to stack before jump
- [ ] **5.3.3** If binary returns, handle gracefully
- [ ] **5.3.4** Log exit

---

## 6. Debugging and Instrumentation

### 6.1 Execution Tracing
**File**: `crates/kernel/src/windows_compat/debug.rs`

- [ ] **6.1.1** Add logging before jump:
  - [ ] Entry point address
  - [ ] Stack pointer
  - [ ] TEB address
  - [ ] PEB address
  - [ ] GS base
- [ ] **6.1.2** Add breakpoint before entry (optional, for debugging)
- [ ] **6.1.3** Implement single-step debugging support (optional)

### 6.2 Memory Dump Utilities
**File**: `crates/kernel/src/windows_compat/debug.rs`

- [ ] **6.2.1** Implement TEB dump function
- [ ] **6.2.2** Implement PEB dump function
- [ ] **6.2.3** Implement stack dump function
- [ ] **6.2.4** Implement memory region dump
- [ ] **6.2.5** Call dumps before jump for debugging

### 6.3 Register Dump
**File**: `crates/kernel/src/windows_compat/debug.rs`

- [ ] **6.3.1** Dump all general purpose registers
- [ ] **6.3.2** Dump segment registers
- [ ] **6.3.3** Dump control registers
- [ ] **6.3.4** Dump critical MSRs
- [ ] **6.3.5** Log register state before jump

---

## 7. Integration Testing

### 7.1 Incremental Testing
- [ ] **7.1.1** Test TEB allocation and initialization
- [ ] **7.1.2** Test PEB allocation and initialization
- [ ] **7.1.3** Test TEB → PEB linking
- [ ] **7.1.4** Test GS register setup
- [ ] **7.1.5** Test stack allocation
- [ ] **7.1.6** Test register initialization
- [ ] **7.1.7** Test entry point calculation

### 7.2 Full Pipeline Test
- [ ] **7.2.1** Boot kernel
- [ ] **7.2.2** Load PE binary
- [ ] **7.2.3** Set up Windows environment
- [ ] **7.2.4** Prepare for jump
- [ ] **7.2.5** Jump to entry point
- [ ] **7.2.6** Observe behavior (will complete in Phase 4)

### 7.3 Crash Handling
- [ ] **7.3.1** Test page fault handler catches invalid memory access
- [ ] **7.3.2** Test general protection fault handler
- [ ] **7.3.3** Ensure informative error messages
- [ ] **7.3.4** Log crash location and state

---

## 8. Documentation

### 8.1 Architecture Documentation
- [ ] **8.1.1** Document TEB/PEB layout
- [ ] **8.1.2** Document memory layout
- [ ] **8.1.3** Document calling convention
- [ ] **8.1.4** Document entry point jump process
- [ ] **8.1.5** Create diagrams for memory layout

### 8.2 Code Documentation
- [ ] **8.2.1** Add rustdoc comments to all structures
- [ ] **8.2.2** Add rustdoc comments to all functions
- [ ] **8.2.3** Add examples where appropriate
- [ ] **8.2.4** Document Windows ABI compatibility

---

## Success Criteria Checklist

Phase 3 is complete when:

- [ ] **S1** TEB structure is complete and initialized
- [ ] **S2** PEB structure is complete and initialized
- [ ] **S3** TEB → PEB linking works
- [ ] **S4** GS:[0x30] returns TEB address
- [ ] **S5** GS:[0x60] returns PEB address
- [ ] **S6** Stack is allocated and initialized
- [ ] **S7** Stack pointer is 16-byte aligned
- [ ] **S8** Entry point address is valid
- [ ] **S9** All registers are initialized correctly
- [ ] **S10** Assembly trampoline compiles and is ready
- [ ] **S11** Pre-jump validation passes
- [ ] **S12** Jump to entry point executes
- [ ] **S13** Can observe entry point being reached (via GDB or logs)
- [ ] **S14** Exception handlers catch any crashes
- [ ] **S15** Documentation is complete

---

## Estimated Task Breakdown

| Section | Tasks | Estimated Complexity |
|---------|-------|---------------------|
| 1. TEB/PEB | 30 | High |
| 2. GDT Enhancements | 15 | Medium |
| 3. Stack Setup | 15 | Medium |
| 4. Registers | 20 | Medium |
| 5. Entry Jump | 15 | High |
| 6. Debugging | 15 | Medium |
| 7. Testing | 15 | Medium |
| 8. Documentation | 10 | Low |
| **Total** | **~135 tasks** | **Medium-High** |

---

## Next Steps

After Phase 3 is complete:
1. Move to Phase 4: Verify API functions work when called
2. Execute target-zero.exe and observe output
3. Debug any issues with API implementations

See [PHASE4.md](PHASE4.md) for next phase details.
