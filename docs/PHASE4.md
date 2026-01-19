# Phase 4: API Implementation and Execution - Detailed Task Checklist

**Goal**: Successfully execute target-zero.exe and see "Hello from Target Zero!" output, demonstrating that all API functions work correctly.

**Location**: `crates/api-shim/` and `crates/kernel/src/api_shim/`

**Success Criteria**: target-zero.exe runs to completion, produces correct output, and exits cleanly with code 0.

---

## 1. API Function Implementation

### 1.1 GetStdHandle Implementation
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **1.1.1** Verify function signature matches Windows ABI:
  ```rust
  #[no_mangle]
  pub extern "C" fn GetStdHandle(std_handle: DWORD) -> HANDLE
  ```
- [ ] **1.1.2** Implement handle mapping:
  - [ ] STD_INPUT_HANDLE (0xFFFFFFF6) → 0x10
  - [ ] STD_OUTPUT_HANDLE (0xFFFFFFF5) → 0x11
  - [ ] STD_ERROR_HANDLE (0xFFFFFFF4) → 0x12
- [ ] **1.1.3** Handle invalid input (return INVALID_HANDLE_VALUE)
- [ ] **1.1.4** Set last error code on failure
- [ ] **1.1.5** Add logging:
  ```rust
  serial_println!("GetStdHandle({:x}) -> 0x{:x}", std_handle, result);
  ```
- [ ] **1.1.6** Test function independently
- [ ] **1.1.7** Verify calling convention (parameter in RCX)

### 1.2 WriteFile Implementation
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **1.2.1** Verify function signature:
  ```rust
  #[no_mangle]
  pub extern "C" fn WriteFile(
      handle: HANDLE,
      buffer: *const u8,
      bytes_to_write: DWORD,
      bytes_written: *mut DWORD,
      overlapped: *mut u8,
  ) -> BOOL
  ```
- [ ] **1.2.2** Validate parameters:
  - [ ] Handle is valid (0x11 or 0x12)
  - [ ] Buffer is not null
  - [ ] bytes_to_write is reasonable (< 1MB)
- [ ] **1.2.3** Create slice from buffer:
  ```rust
  let data = unsafe {
      core::slice::from_raw_parts(buffer, bytes_to_write as usize)
  };
  ```
- [ ] **1.2.4** Write to serial port:
  ```rust
  for &byte in data {
      serial_write_byte(byte);
  }
  ```
- [ ] **1.2.5** Update bytes_written pointer:
  ```rust
  if !bytes_written.is_null() {
      unsafe { *bytes_written = bytes_to_write; }
  }
  ```
- [ ] **1.2.6** Set last error to ERROR_SUCCESS
- [ ] **1.2.7** Return TRUE (1)
- [ ] **1.2.8** Handle errors:
  - [ ] Invalid handle → ERROR_INVALID_HANDLE
  - [ ] Null buffer → ERROR_INVALID_PARAMETER
  - [ ] Write failure → ERROR_WRITE_FAULT
- [ ] **1.2.9** Add logging
- [ ] **1.2.10** Test function independently
- [ ] **1.2.11** Verify calling convention (parameters in RCX, RDX, R8, R9, stack)

### 1.3 ExitProcess Implementation
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **1.3.1** Verify function signature:
  ```rust
  #[no_mangle]
  pub extern "C" fn ExitProcess(exit_code: u32) -> !
  ```
- [ ] **1.3.2** Log exit code:
  ```rust
  serial_println!("ExitProcess called with code: {}", exit_code);
  ```
- [ ] **1.3.3** In kernel mode, halt gracefully:
  ```rust
  loop { x86_64::instructions::hlt(); }
  ```
- [ ] **1.3.4** Optionally trigger QEMU exit:
  ```rust
  // QEMU exit device (port 0xf4)
  unsafe { x86_64::instructions::port::Port::new(0xf4).write(exit_code as u8); }
  ```
- [ ] **1.3.5** Never return (marked with `-> !`)
- [ ] **1.3.6** Test function independently

### 1.4 GetLastError Implementation
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **1.4.1** Verify function signature:
  ```rust
  #[no_mangle]
  pub extern "C" fn GetLastError() -> DWORD
  ```
- [ ] **1.4.2** Retrieve error from TEB:
  ```rust
  unsafe {
      let teb: *const TEB;
      asm!("mov {}, gs:[0x30]", out(reg) teb);
      (*teb).last_error
  }
  ```
- [ ] **1.4.3** For Phase 4, use global variable:
  ```rust
  static mut LAST_ERROR: DWORD = 0;
  unsafe { LAST_ERROR }
  ```
- [ ] **1.4.4** Add logging (optional, can be verbose)
- [ ] **1.4.5** Test function independently

### 1.5 SetLastError Implementation
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **1.5.1** Verify function signature:
  ```rust
  #[no_mangle]
  pub extern "C" fn SetLastError(error: DWORD)
  ```
- [ ] **1.5.2** Store error in TEB:
  ```rust
  unsafe {
      let teb: *mut TEB;
      asm!("mov {}, gs:[0x30]", out(reg) teb);
      (*teb).last_error = error;
  }
  ```
- [ ] **1.5.3** For Phase 4, use global variable:
  ```rust
  static mut LAST_ERROR: DWORD = 0;
  unsafe { LAST_ERROR = error; }
  ```
- [ ] **1.5.4** Test function independently

---

## 2. Symbol Resolution and IAT Patching

### 2.1 Symbol Resolver Registration
**File**: `crates/kernel/src/pe_loader/imports.rs`

- [ ] **2.1.1** Register all kernel32.dll functions:
  ```rust
  resolver.register("kernel32.dll", "GetStdHandle", GetStdHandle as usize);
  resolver.register("kernel32.dll", "WriteFile", WriteFile as usize);
  resolver.register("kernel32.dll", "ExitProcess", ExitProcess as usize);
  resolver.register("kernel32.dll", "GetLastError", GetLastError as usize);
  resolver.register("kernel32.dll", "SetLastError", SetLastError as usize);
  ```
- [ ] **2.1.2** Ensure function pointers are correct
- [ ] **2.1.3** Verify addresses are valid (not null)
- [ ] **2.1.4** Log registered functions and addresses

### 2.2 IAT Patching Verification
**File**: `crates/kernel/src/pe_loader/imports.rs`

- [ ] **2.2.1** After patching, verify IAT entries:
  ```rust
  for entry in iat {
      let addr = *entry;
      assert!(addr != 0, "IAT entry is null");
      assert!(is_valid_function_address(addr), "Invalid function address");
  }
  ```
- [ ] **2.2.2** Log IAT entries before and after patching
- [ ] **2.2.3** Ensure no imports are unresolved
- [ ] **2.2.4** Add debug dump of IAT

### 2.3 Import Name Verification
**File**: `crates/kernel/src/pe_loader/imports.rs`

- [ ] **2.3.1** Verify target-zero.exe imports exactly:
  - [ ] kernel32.dll: GetStdHandle
  - [ ] kernel32.dll: WriteFile
  - [ ] kernel32.dll: ExitProcess
- [ ] **2.3.2** Warn if unexpected imports found
- [ ] **2.3.3** Error if required imports missing
- [ ] **2.3.4** Log all import resolutions

---

## 3. Execution Flow

### 3.1 Pre-Execution Checklist
**File**: `crates/kernel/src/main.rs`

- [ ] **3.1.1** Verify kernel initialization complete:
  - [ ] GDT loaded
  - [ ] IDT loaded
  - [ ] Memory managers initialized
  - [ ] Serial/VGA output working
- [ ] **3.1.2** Verify PE loading complete:
  - [ ] Headers parsed
  - [ ] Sections mapped
  - [ ] Relocations applied (if needed)
  - [ ] Imports resolved
- [ ] **3.1.3** Verify Windows environment ready:
  - [ ] TEB allocated and initialized
  - [ ] PEB allocated and initialized
  - [ ] GS register set
  - [ ] Stack allocated
- [ ] **3.1.4** Verify API functions registered:
  - [ ] All required functions available
  - [ ] Function pointers valid
- [ ] **3.1.5** Log pre-execution state

### 3.2 Entry Point Invocation
**File**: `crates/kernel/src/windows_compat/execution.rs`

- [ ] **3.2.1** Calculate entry point VA:
  ```rust
  let entry_point = image_base + pe.entry_point_rva;
  ```
- [ ] **3.2.2** Validate entry point is executable
- [ ] **3.2.3** Set up stack with return address
- [ ] **3.2.4** Log execution start:
  ```rust
  serial_println!("Jumping to entry point: 0x{:x}", entry_point);
  serial_println!("Stack top: 0x{:x}", stack_top);
  ```
- [ ] **3.2.5** Jump using assembly trampoline:
  ```rust
  unsafe {
      jump_to_entry(entry_point, stack_top);
  }
  ```
- [ ] **3.2.6** Never returns (process exits via ExitProcess)

### 3.3 Execution Monitoring
**File**: `crates/kernel/src/windows_compat/execution.rs`

- [ ] **3.3.1** Set up page fault handler to log:
  - [ ] Faulting address
  - [ ] Instruction pointer
  - [ ] Error code
- [ ] **3.3.2** Set up general protection fault handler to log
- [ ] **3.3.3** Catch any crashes and print debug info
- [ ] **3.3.4** Log all API function calls (via wrapper)

---

## 4. Output Handling

### 4.1 Serial Output Buffer
**File**: `crates/kernel/src/serial.rs`

- [ ] **4.1.1** Verify serial port is initialized
- [ ] **4.1.2** Test writing bytes to serial
- [ ] **4.1.3** Test writing strings to serial
- [ ] **4.1.4** Ensure output is unbuffered (immediate)
- [ ] **4.1.5** Add newline handling (CRLF vs LF)

### 4.2 Expected Output Validation
**File**: Integration test

- [ ] **4.2.1** Define expected output:
  ```
  Hello from Target Zero!
  ```
- [ ] **4.2.2** Capture serial output
- [ ] **4.2.3** Compare with expected output
- [ ] **4.2.4** Verify no extra or missing characters
- [ ] **4.2.5** Verify exit code is 0

### 4.3 QEMU Serial Capture
**File**: Update `justfile`

- [ ] **4.3.1** Add QEMU flag to capture serial output:
  ```just
  run-qemu:
      qemu-system-x86_64 \
          -drive format=raw,file=target/kernel.bin \
          -serial file:output.txt \
          -display none \
          -device isa-debug-exit,iobase=0xf4,iosize=0x04
  ```
- [ ] **4.3.2** Read output.txt after execution
- [ ] **4.3.3** Validate output programmatically
- [ ] **4.3.4** Add to integration tests

---

## 5. Error Handling and Debugging

### 5.1 Common Issues and Solutions

#### 5.1.1 Page Fault on Entry
- [ ] **Issue**: Entry point causes page fault
- [ ] **Debug**:
  - [ ] Verify entry point address is correct
  - [ ] Verify .text section is mapped
  - [ ] Verify .text section has execute permission
  - [ ] Check PE headers for correct entry point RVA
- [ ] **Solution**: Fix section mapping or permissions

#### 5.1.2 Page Fault in API Function
- [ ] **Issue**: API function causes page fault
- [ ] **Debug**:
  - [ ] Verify function pointer in IAT is correct
  - [ ] Verify API function code is mapped
  - [ ] Check parameters passed to function
  - [ ] Verify buffer pointers are valid
- [ ] **Solution**: Fix IAT patching or parameter validation

#### 5.1.3 General Protection Fault
- [ ] **Issue**: GPF when calling API or at entry
- [ ] **Debug**:
  - [ ] Check segment registers (CS, DS, SS)
  - [ ] Verify stack pointer is valid
  - [ ] Check for null pointers
  - [ ] Verify function calling convention
- [ ] **Solution**: Fix segment setup or calling convention

#### 5.1.4 No Output Appears
- [ ] **Issue**: Execution proceeds but no output
- [ ] **Debug**:
  - [ ] Verify WriteFile is called (add logging)
  - [ ] Check serial port initialization
  - [ ] Verify buffer and length parameters
  - [ ] Check if QEMU serial is configured correctly
- [ ] **Solution**: Fix WriteFile or serial port

#### 5.1.5 Wrong Output
- [ ] **Issue**: Output is garbled or incorrect
- [ ] **Debug**:
  - [ ] Check buffer encoding (ASCII vs UTF-16)
  - [ ] Verify length parameter
  - [ ] Check for buffer overruns
  - [ ] Verify relocations applied correctly
- [ ] **Solution**: Fix relocation or parameter handling

### 5.2 Debugging Tools
**File**: `crates/kernel/src/debug/mod.rs`

- [ ] **5.2.1** Implement function call tracer:
  ```rust
  #[macro_export]
  macro_rules! trace_call {
      ($func:ident, $($arg:expr),*) => {
          serial_println!("TRACE: {}({:?})", stringify!($func), ($($arg,)*));
          let result = $func($($arg),*);
          serial_println!("TRACE: {} -> {:?}", stringify!($func), result);
          result
      };
  }
  ```
- [ ] **5.2.2** Wrap all API functions with tracer
- [ ] **5.2.3** Add memory access tracer (page fault handler)
- [ ] **5.2.4** Add register dump on crash
- [ ] **5.2.5** Add stack trace on crash

### 5.3 GDB Integration
**File**: Update `justfile`

- [ ] **5.3.1** Add GDB debugging support:
  ```just
  debug-qemu:
      qemu-system-x86_64 \
          -drive format=raw,file=target/kernel.bin \
          -serial stdio \
          -display none \
          -s -S
  ```
- [ ] **5.3.2** Create GDB script:
  ```gdb
  target remote :1234
  break *0x<entry_point_address>
  continue
  ```
- [ ] **5.3.3** Test GDB connection
- [ ] **5.3.4** Document debugging workflow

---

## 6. Testing and Validation

### 6.1 Unit Tests for API Functions
**File**: `crates/api-shim/tests/kernel32_tests.rs`

- [ ] **6.1.1** Test GetStdHandle:
  ```rust
  #[test]
  fn test_get_std_handle() {
      assert_eq!(GetStdHandle(STD_OUTPUT_HANDLE), 0x11);
      assert_eq!(GetStdHandle(STD_INPUT_HANDLE), 0x10);
      assert_eq!(GetStdHandle(0xFFFFFFFF), INVALID_HANDLE_VALUE);
  }
  ```
- [ ] **6.1.2** Test WriteFile with mock serial
- [ ] **6.1.3** Test GetLastError/SetLastError
- [ ] **6.1.4** Test error conditions

### 6.2 Integration Test
**File**: `crates/kernel/tests/integration_test.rs`

- [ ] **6.2.1** Test full pipeline:
  1. Boot kernel
  2. Load PE binary
  3. Execute binary
  4. Capture output
  5. Verify output matches expected
  6. Verify exit code is 0
- [ ] **6.2.2** Automate test in CI (optional)

### 6.3 Manual Testing
- [ ] **6.3.1** Build kernel
- [ ] **6.3.2** Run in QEMU
- [ ] **6.3.3** Observe serial output
- [ ] **6.3.4** Verify output: "Hello from Target Zero!"
- [ ] **6.3.5** Verify clean exit
- [ ] **6.3.6** Test multiple times for consistency

---

## 7. Performance and Optimization

### 7.1 Performance Metrics
- [ ] **7.1.1** Measure boot time
- [ ] **7.1.2** Measure PE load time
- [ ] **7.1.3** Measure execution time
- [ ] **7.1.4** Log performance metrics
- [ ] **7.1.5** Identify bottlenecks (for future optimization)

### 7.2 Optimization Opportunities (Future)
- [ ] **7.2.1** Optimize page table operations
- [ ] **7.2.2** Optimize IAT patching
- [ ] **7.2.3** Reduce logging overhead
- [ ] **7.2.4** Optimize memory allocation
- [ ] **7.2.5** Note: Keep it simple for Phase 4, optimize later

---

## 8. Documentation

### 8.1 API Documentation
- [ ] **8.1.1** Document each API function:
  - [ ] Purpose
  - [ ] Parameters
  - [ ] Return value
  - [ ] Error codes
  - [ ] Differences from Windows
- [ ] **8.1.2** Add usage examples
- [ ] **8.1.3** Document limitations

### 8.2 Execution Flow Documentation
- [ ] **8.2.1** Document entry point jump
- [ ] **8.2.2** Document calling convention
- [ ] **8.2.3** Document parameter passing
- [ ] **8.2.4** Create flow diagram
- [ ] **8.2.5** Document debugging process

### 8.3 User Guide
- [ ] **8.3.1** How to build and run
- [ ] **8.3.2** How to debug
- [ ] **8.3.3** How to add new API functions
- [ ] **8.3.4** Troubleshooting guide
- [ ] **8.3.5** FAQ

---

## Success Criteria Checklist

Phase 4 is complete when:

- [ ] **S1** All API functions (GetStdHandle, WriteFile, ExitProcess, GetLastError) are implemented
- [ ] **S2** All API functions are registered in symbol resolver
- [ ] **S3** IAT is patched with correct function addresses
- [ ] **S4** Entry point jump executes successfully
- [ ] **S5** GetStdHandle is called and returns correct handle
- [ ] **S6** WriteFile is called and produces output
- [ ] **S7** Output "Hello from Target Zero!" appears on serial console
- [ ] **S8** Output is exactly correct (no extra/missing characters)
- [ ] **S9** ExitProcess is called with exit code 0
- [ ] **S10** Process exits cleanly (no crashes)
- [ ] **S11** All tests pass
- [ ] **S12** Execution is reproducible (runs correctly multiple times)
- [ ] **S13** Documentation is complete
- [ ] **S14** Code is clean and well-commented

---

## Estimated Task Breakdown

| Section | Tasks | Estimated Complexity |
|---------|-------|---------------------|
| 1. API Implementation | 35 | Medium |
| 2. Symbol Resolution | 10 | Low |
| 3. Execution Flow | 15 | Medium |
| 4. Output Handling | 10 | Low |
| 5. Error Handling | 25 | Medium |
| 6. Testing | 15 | Medium |
| 7. Performance | 10 | Low |
| 8. Documentation | 15 | Low |
| **Total** | **~135 tasks** | **Medium** |

---

## Celebration Moment! 🎉

When Phase 4 is complete, you will have successfully:
- Built a bare-metal kernel from scratch
- Implemented a PE loader
- Created a Windows API compatibility layer
- Executed an unmodified Windows binary on bare-metal
- Achieved the core goal of the project!

This is a significant milestone. The remaining phases are enhancements.

---

## Next Steps

After Phase 4 is complete:
1. Move to Phase 5: Expand API support (heap, files, command line)
2. Add more complex test binaries
3. Optimize and harden the implementation

See [PHASE5.md](PHASE5.md) for next phase details.
