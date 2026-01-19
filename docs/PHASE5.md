# Phase 5: Expansion and Hardening - Detailed Task Checklist

**Goal**: Expand API support beyond basic I/O to enable more complex Windows binaries, including heap allocation, file I/O, command line arguments, and basic exception handling.

**Location**: `crates/api-shim/` and `crates/kernel/`

**Success Criteria**: Run more complex Windows binaries that use heap allocation, file operations, and command-line processing.

---

## 1. Heap Management

### 1.1 Heap Allocator Integration
**File**: `crates/api-shim/src/heap.rs`

- [ ] **1.1.1** Create global heap allocator instance
- [ ] **1.1.2** Initialize heap with large memory region (e.g., 16 MB)
- [ ] **1.1.3** Implement heap growth mechanism (allocate more pages as needed)
- [ ] **1.1.4** Add heap statistics tracking
- [ ] **1.1.5** Log heap initialization

### 1.2 GetProcessHeap Implementation
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **1.2.1** Define function signature:
  ```rust
  #[no_mangle]
  pub extern "C" fn GetProcessHeap() -> HANDLE
  ```
- [ ] **1.2.2** Return handle to process heap (fake handle, e.g., 0x1000)
- [ ] **1.2.3** Store heap handle in PEB:
  ```rust
  unsafe {
      let peb = get_peb();
      (*peb).process_heap = PROCESS_HEAP_HANDLE;
  }
  ```
- [ ] **1.2.4** Test function
- [ ] **1.2.5** Add logging

### 1.3 HeapAlloc Implementation
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **1.3.1** Define function signature:
  ```rust
  #[no_mangle]
  pub extern "C" fn HeapAlloc(
      heap: HANDLE,
      flags: DWORD,
      size: usize,
  ) -> *mut u8
  ```
- [ ] **1.3.2** Validate heap handle
- [ ] **1.3.3** Check flags:
  - [ ] HEAP_ZERO_MEMORY (0x00000008) - Zero the allocated memory
  - [ ] HEAP_GENERATE_EXCEPTIONS (0x00000004) - Raise exception on failure
- [ ] **1.3.4** Allocate memory using kernel allocator:
  ```rust
  let layout = Layout::from_size_align(size, 8).unwrap();
  let ptr = unsafe { alloc::alloc::alloc(layout) };
  ```
- [ ] **1.3.5** If HEAP_ZERO_MEMORY, zero the memory
- [ ] **1.3.6** Track allocation in heap metadata
- [ ] **1.3.7** Return pointer or null on failure
- [ ] **1.3.8** Set last error appropriately
- [ ] **1.3.9** Add logging
- [ ] **1.3.10** Test with various sizes

### 1.4 HeapFree Implementation
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **1.4.1** Define function signature:
  ```rust
  #[no_mangle]
  pub extern "C" fn HeapFree(
      heap: HANDLE,
      flags: DWORD,
      ptr: *mut u8,
  ) -> BOOL
  ```
- [ ] **1.4.2** Validate heap handle
- [ ] **1.4.3** Validate pointer is not null
- [ ] **1.4.4** Look up allocation size in metadata
- [ ] **1.4.5** Deallocate memory:
  ```rust
  let layout = Layout::from_size_align(size, 8).unwrap();
  unsafe { alloc::alloc::dealloc(ptr, layout); }
  ```
- [ ] **1.4.6** Remove from heap metadata
- [ ] **1.4.7** Return TRUE on success
- [ ] **1.4.8** Set last error on failure
- [ ] **1.4.9** Add logging
- [ ] **1.4.10** Test freeing allocations

### 1.5 HeapReAlloc Implementation
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **1.5.1** Define function signature:
  ```rust
  #[no_mangle]
  pub extern "C" fn HeapReAlloc(
      heap: HANDLE,
      flags: DWORD,
      ptr: *mut u8,
      size: usize,
  ) -> *mut u8
  ```
- [ ] **1.5.2** Validate parameters
- [ ] **1.5.3** Allocate new block with new size
- [ ] **1.5.4** Copy old data to new block
- [ ] **1.5.5** Free old block
- [ ] **1.5.6** Update metadata
- [ ] **1.5.7** Return new pointer
- [ ] **1.5.8** Test growing and shrinking

### 1.6 Heap Test Binary
**File**: Create `target-heap-test.c`

- [ ] **1.6.1** Create test that allocates memory:
  ```c
  HANDLE heap = GetProcessHeap();
  void* ptr = HeapAlloc(heap, HEAP_ZERO_MEMORY, 1024);
  // Use memory
  HeapFree(heap, 0, ptr);
  ```
- [ ] **1.6.2** Build as Windows PE
- [ ] **1.6.3** Test in unikernel
- [ ] **1.6.4** Verify allocations work correctly

---

## 2. Command Line Arguments

### 2.1 Command Line Storage
**File**: `crates/kernel/src/windows_compat/cmdline.rs`

- [ ] **2.1.1** Define command line string (hardcoded for Phase 5)
- [ ] **2.1.2** Convert to UTF-16 (Windows uses wide strings):
  ```rust
  fn to_wide_string(s: &str) -> Vec<u16> {
      s.encode_utf16().chain(once(0)).collect()
  }
  ```
- [ ] **2.1.3** Store in kernel memory
- [ ] **2.1.4** Update PEB.ProcessParameters.CommandLine

### 2.2 UNICODE_STRING Structure
**File**: `crates/common/src/windows.rs`

- [ ] **2.2.1** Define UNICODE_STRING:
  ```rust
  #[repr(C)]
  pub struct UNICODE_STRING {
      pub length: u16,          // Length in bytes, not including null
      pub maximum_length: u16,  // Buffer size
      pub buffer: *mut u16,     // Wide string pointer
  }
  ```
- [ ] **2.2.2** Implement initialization function
- [ ] **2.2.3** Test structure layout

### 2.3 GetCommandLineW Implementation
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **2.3.1** Define function signature:
  ```rust
  #[no_mangle]
  pub extern "C" fn GetCommandLineW() -> *const u16
  ```
- [ ] **2.3.2** Return pointer to command line from PEB:
  ```rust
  unsafe {
      let peb = get_peb();
      let params = (*peb).process_parameters;
      (*params).command_line.buffer
  }
  ```
- [ ] **2.3.3** Test function
- [ ] **2.3.4** Add logging

### 2.4 GetCommandLineA Implementation
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **2.4.1** Define function signature:
  ```rust
  #[no_mangle]
  pub extern "C" fn GetCommandLineA() -> *const u8
  ```
- [ ] **2.4.2** Convert wide string to ANSI
- [ ] **2.4.3** Store ANSI version
- [ ] **2.4.4** Return pointer
- [ ] **2.4.5** Test function

### 2.5 Command Line Test Binary
**File**: Create `target-cmdline-test.c`

- [ ] **2.5.1** Create test that reads command line:
  ```c
  LPWSTR cmdline = GetCommandLineW();
  // Parse and print command line
  ```
- [ ] **2.5.2** Build and test
- [ ] **2.5.3** Verify command line is correct

---

## 3. Environment Variables

### 3.1 Environment Storage
**File**: `crates/kernel/src/windows_compat/env.rs`

- [ ] **3.1.1** Create environment variable storage (HashMap or array)
- [ ] **3.1.2** Initialize with default variables:
  - [ ] PATH
  - [ ] COMPUTERNAME
  - [ ] USERNAME
  - [ ] OS
  - [ ] PROCESSOR_ARCHITECTURE
- [ ] **3.1.3** Store in wide string format (UTF-16)
- [ ] **3.1.4** Link to PEB.ProcessParameters.Environment

### 3.2 GetEnvironmentVariableW Implementation
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **3.2.1** Define function signature:
  ```rust
  #[no_mangle]
  pub extern "C" fn GetEnvironmentVariableW(
      name: *const u16,
      buffer: *mut u16,
      size: DWORD,
  ) -> DWORD
  ```
- [ ] **3.2.2** Convert name from wide string
- [ ] **3.2.3** Look up variable in storage
- [ ] **3.2.4** If found, copy to buffer (respecting size)
- [ ] **3.2.5** Return length of value
- [ ] **3.2.6** If buffer too small, return required size
- [ ] **3.2.7** If not found, set last error and return 0
- [ ] **3.2.8** Test function

### 3.3 SetEnvironmentVariableW Implementation
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **3.3.1** Define function signature:
  ```rust
  #[no_mangle]
  pub extern "C" fn SetEnvironmentVariableW(
      name: *const u16,
      value: *const u16,
  ) -> BOOL
  ```
- [ ] **3.3.2** Convert name and value from wide strings
- [ ] **3.3.3** If value is null, delete variable
- [ ] **3.3.4** Otherwise, set/update variable
- [ ] **3.3.5** Return TRUE on success
- [ ] **3.3.6** Test function

### 3.4 ANSI Versions
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **3.4.1** Implement GetEnvironmentVariableA
- [ ] **3.4.2** Implement SetEnvironmentVariableA
- [ ] **3.4.3** Convert between ANSI and wide strings
- [ ] **3.4.4** Test both versions

---

## 4. File I/O

### 4.1 File System Abstraction
**File**: `crates/kernel/src/fs/mod.rs`

- [ ] **4.1.1** Design simple file system interface:
  ```rust
  trait FileSystem {
      fn open(&self, path: &str, mode: OpenMode) -> Result<FileHandle>;
      fn close(&self, handle: FileHandle) -> Result<()>;
      fn read(&self, handle: FileHandle, buffer: &mut [u8]) -> Result<usize>;
      fn write(&self, handle: FileHandle, buffer: &[u8]) -> Result<usize>;
  }
  ```
- [ ] **4.1.2** Implement in-memory file system (simple RAM disk)
- [ ] **4.1.3** Support basic operations (open, close, read, write)
- [ ] **4.1.4** Store file handles in table
- [ ] **4.1.5** Implement file permissions (optional)

### 4.2 CreateFileW Implementation
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **4.2.1** Define function signature (simplified):
  ```rust
  #[no_mangle]
  pub extern "C" fn CreateFileW(
      filename: *const u16,
      desired_access: DWORD,
      share_mode: DWORD,
      security_attributes: *mut u8,
      creation_disposition: DWORD,
      flags_and_attributes: DWORD,
      template_file: HANDLE,
  ) -> HANDLE
  ```
- [ ] **4.2.2** Convert filename from wide string
- [ ] **4.2.3** Parse creation disposition:
  - [ ] CREATE_NEW (1)
  - [ ] CREATE_ALWAYS (2)
  - [ ] OPEN_EXISTING (3)
  - [ ] OPEN_ALWAYS (4)
  - [ ] TRUNCATE_EXISTING (5)
- [ ] **4.2.4** Call file system to open file
- [ ] **4.2.5** Allocate handle
- [ ] **4.2.6** Return handle or INVALID_HANDLE_VALUE
- [ ] **4.2.7** Set last error appropriately
- [ ] **4.2.8** Test function

### 4.3 ReadFile Implementation (Extended)
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **4.3.1** Extend ReadFile to support file handles (not just stdin)
- [ ] **4.3.2** Determine if handle is a file or console
- [ ] **4.3.3** For files, call file system read
- [ ] **4.3.4** Update bytes_read parameter
- [ ] **4.3.5** Return TRUE/FALSE
- [ ] **4.3.6** Test reading from files

### 4.4 WriteFile Implementation (Extended)
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **4.4.1** Extend WriteFile to support file handles
- [ ] **4.4.2** Determine if handle is a file or console
- [ ] **4.4.3** For files, call file system write
- [ ] **4.4.4** Update bytes_written parameter
- [ ] **4.4.5** Return TRUE/FALSE
- [ ] **4.4.6** Test writing to files

### 4.5 CloseHandle Implementation
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **4.5.1** Define function signature:
  ```rust
  #[no_mangle]
  pub extern "C" fn CloseHandle(handle: HANDLE) -> BOOL
  ```
- [ ] **4.5.2** Validate handle
- [ ] **4.5.3** Determine handle type (file, console, etc.)
- [ ] **4.5.4** For files, call file system close
- [ ] **4.5.5** Free handle
- [ ] **4.5.6** Return TRUE on success
- [ ] **4.5.7** Test function

### 4.6 File I/O Test Binary
**File**: Create `target-file-test.c`

- [ ] **4.6.1** Create test that writes to a file:
  ```c
  HANDLE file = CreateFileW(L"test.txt", GENERIC_WRITE, ...);
  WriteFile(file, "Hello, file!", 12, &written, NULL);
  CloseHandle(file);
  ```
- [ ] **4.6.2** Build and test
- [ ] **4.6.3** Verify file operations work

---

## 5. Exception Handling (Basic)

### 5.1 Exception Structures
**File**: `crates/common/src/windows.rs`

- [ ] **5.1.1** Define EXCEPTION_RECORD:
  ```rust
  #[repr(C)]
  pub struct EXCEPTION_RECORD {
      pub exception_code: DWORD,
      pub exception_flags: DWORD,
      pub exception_record: *mut EXCEPTION_RECORD,
      pub exception_address: *mut u8,
      pub number_parameters: DWORD,
      pub exception_information: [usize; 15],
  }
  ```
- [ ] **5.1.2** Define EXCEPTION_POINTERS
- [ ] **5.1.3** Define CONTEXT structure (complex, simplified version)

### 5.2 RtlAddFunctionTable Stub
**File**: `crates/api-shim/src/ntdll.rs`

- [ ] **5.2.1** Define function signature:
  ```rust
  #[no_mangle]
  pub extern "C" fn RtlAddFunctionTable(
      function_table: *mut u8,
      entry_count: DWORD,
      base_address: u64,
  ) -> BOOL
  ```
- [ ] **5.2.2** For Phase 5, just return TRUE (stub)
- [ ] **5.2.3** Log that function was called
- [ ] **5.2.4** Note: Full implementation is complex (Phase 6+)

### 5.3 UnhandledExceptionFilter Stub
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **5.3.1** Define function signature
- [ ] **5.3.2** Log exception information
- [ ] **5.3.3** Return EXCEPTION_EXECUTE_HANDLER
- [ ] **5.3.4** Note: Proper SEH is Phase 6+

### 5.4 Exception Test
**File**: Create `target-exception-test.c`

- [ ] **5.4.1** Create test that uses __try/__except (if compiler supports)
- [ ] **5.4.2** Or test that links against code using RtlAddFunctionTable
- [ ] **5.4.3** Verify stub doesn't crash
- [ ] **5.4.4** Note: Full SEH testing is future work

---

## 6. Thread Local Storage (TLS) - Basic

### 6.1 TLS Directory Parsing
**File**: `crates/kernel/src/pe_loader/tls.rs`

- [ ] **6.1.1** Parse TLS directory from data directories
- [ ] **6.1.2** Extract TLS data:
  - [ ] Start address of TLS data
  - [ ] End address of TLS data
  - [ ] Address of index
  - [ ] Address of callbacks
  - [ ] Size of zero fill
  - [ ] Characteristics
- [ ] **6.1.3** Allocate TLS data
- [ ] **6.1.4** Copy initial data
- [ ] **6.1.5** Zero-fill additional space

### 6.2 TLS Callbacks
**File**: `crates/kernel/src/pe_loader/tls.rs`

- [ ] **6.2.1** Parse TLS callback array
- [ ] **6.2.2** For each callback, call it before main:
  ```rust
  type TlsCallback = extern "C" fn(*mut u8, DWORD, *mut u8);
  for callback in tls_callbacks {
      callback(image_base, DLL_PROCESS_ATTACH, null_mut());
  }
  ```
- [ ] **6.2.3** Test with binary that has TLS callbacks

### 6.3 TLS Access (FS/GS)
**File**: `crates/kernel/src/windows_compat/tls.rs`

- [ ] **6.3.1** Store TLS index in TEB
- [ ] **6.3.2** Support basic TLS access (future: full implementation)
- [ ] **6.3.3** Note: Full TLS support is complex (Phase 6+)

---

## 7. Additional API Functions

### 7.1 String Functions
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **7.1.1** Implement lstrcmpW (string compare)
- [ ] **7.1.2** Implement lstrcpyW (string copy)
- [ ] **7.1.3** Implement lstrlenW (string length)
- [ ] **7.1.4** Test functions

### 7.2 Memory Functions
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **7.2.1** Implement VirtualAlloc (simplified)
- [ ] **7.2.2** Implement VirtualFree (simplified)
- [ ] **7.2.3** Implement VirtualProtect (use VMM)
- [ ] **7.2.4** Test functions

### 7.3 System Information
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **7.3.1** Implement GetSystemInfo (return fake values)
- [ ] **7.3.2** Implement GetVersionExW (return Windows version)
- [ ] **7.3.3** Implement GetTickCount (return uptime)
- [ ] **7.3.4** Test functions

### 7.4 Synchronization (Stubs)
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **7.4.1** Stub InitializeCriticalSection
- [ ] **7.4.2** Stub EnterCriticalSection
- [ ] **7.4.3** Stub LeaveCriticalSection
- [ ] **7.4.4** Stub DeleteCriticalSection
- [ ] **7.4.5** Note: Single-threaded for now, so stubs are OK

---

## 8. Testing with Complex Binaries

### 8.1 Create Advanced Test Binaries
**Directory**: `target-advanced/`

- [ ] **8.1.1** Binary that uses heap (malloc/free)
- [ ] **8.1.2** Binary that reads command line
- [ ] **8.1.3** Binary that uses environment variables
- [ ] **8.1.4** Binary that does file I/O
- [ ] **8.1.5** Binary that combines multiple features
- [ ] **8.1.6** Build all binaries

### 8.2 Test Each Binary
- [ ] **8.2.1** Run heap test in unikernel
- [ ] **8.2.2** Run command line test in unikernel
- [ ] **8.2.3** Run environment test in unikernel
- [ ] **8.2.4** Run file I/O test in unikernel
- [ ] **8.2.5** Run combined test in unikernel
- [ ] **8.2.6** Verify all tests pass

### 8.3 Real-World Binary Testing (Stretch Goal)
- [ ] **8.3.1** Try simple Windows utilities (e.g., busybox-w32)
- [ ] **8.3.2** Document compatibility
- [ ] **8.3.3** Identify missing API functions
- [ ] **8.3.4** Prioritize implementation

---

## 9. Error Handling and Robustness

### 9.1 Error Code Coverage
**File**: `crates/api-shim/src/errors.rs`

- [ ] **9.1.1** Define all common Windows error codes
- [ ] **9.1.2** Ensure all API functions set appropriate errors
- [ ] **9.1.3** Test error paths
- [ ] **9.1.4** Log errors consistently

### 9.2 Parameter Validation
**File**: All API functions

- [ ] **9.2.1** Validate all pointers (not null, properly aligned)
- [ ] **9.2.2** Validate all handles (valid range, correct type)
- [ ] **9.2.3** Validate all sizes (reasonable limits)
- [ ] **9.2.4** Validate all flags (known values)
- [ ] **9.2.5** Return errors instead of panicking

### 9.3 Memory Safety
**File**: All memory operations

- [ ] **9.3.1** Check for buffer overruns
- [ ] **9.3.2** Check for use-after-free
- [ ] **9.3.3** Check for double-free
- [ ] **9.3.4** Check for memory leaks (add tracking)
- [ ] **9.3.5** Run memory checker (if available)

---

## 10. Performance Optimization

### 10.1 Profiling
- [ ] **10.1.1** Add performance counters
- [ ] **10.1.2** Measure API call overhead
- [ ] **10.1.3** Measure memory allocation overhead
- [ ] **10.1.4** Identify hot paths
- [ ] **10.1.5** Log performance metrics

### 10.2 Optimization Opportunities
- [ ] **10.2.1** Cache PEB/TEB lookups
- [ ] **10.2.2** Optimize heap allocator
- [ ] **10.2.3** Reduce logging overhead (make conditional)
- [ ] **10.2.4** Optimize page table operations
- [ ] **10.2.5** Batch operations where possible

### 10.3 Benchmarking
- [ ] **10.3.1** Create benchmark suite
- [ ] **10.3.2** Measure before optimization
- [ ] **10.3.3** Apply optimizations
- [ ] **10.3.4** Measure after optimization
- [ ] **10.3.5** Document improvements

---

## 11. Documentation and Maintenance

### 11.1 API Reference
- [ ] **11.1.1** Document all implemented API functions
- [ ] **11.1.2** Document differences from Windows
- [ ] **11.1.3** Document limitations and known issues
- [ ] **11.1.4** Provide usage examples
- [ ] **11.1.5** Create API compatibility matrix

### 11.2 Developer Guide
- [ ] **11.2.1** How to add new API functions
- [ ] **11.2.2** How to extend file system
- [ ] **11.2.3** How to add new test binaries
- [ ] **11.2.4** Best practices and patterns
- [ ] **11.2.5** Troubleshooting guide

### 11.3 User Documentation
- [ ] **11.3.1** Update README with Phase 5 features
- [ ] **11.3.2** Document supported binaries
- [ ] **11.3.3** Provide examples
- [ ] **11.3.4** Create FAQ
- [ ] **11.3.5** Document known limitations

---

## Success Criteria Checklist

Phase 5 is complete when:

- [ ] **S1** Heap allocation (HeapAlloc/HeapFree) works
- [ ] **S2** Command line arguments are accessible
- [ ] **S3** Environment variables work (get/set)
- [ ] **S4** File I/O works (create, read, write, close)
- [ ] **S5** All basic string/memory functions work
- [ ] **S6** Multiple complex test binaries run successfully
- [ ] **S7** Error handling is robust
- [ ] **S8** Memory safety is verified
- [ ] **S9** Performance is acceptable
- [ ] **S10** Documentation is comprehensive
- [ ] **S11** Code is maintainable and well-tested

---

## Estimated Task Breakdown

| Section | Tasks | Estimated Complexity |
|---------|-------|---------------------|
| 1. Heap Management | 30 | Medium-High |
| 2. Command Line | 15 | Medium |
| 3. Environment Vars | 15 | Medium |
| 4. File I/O | 30 | High |
| 5. Exception Handling | 15 | Medium (stubs) |
| 6. TLS | 15 | Medium (basic) |
| 7. Additional APIs | 20 | Medium |
| 8. Testing | 20 | Medium |
| 9. Error Handling | 20 | Medium |
| 10. Performance | 15 | Low-Medium |
| 11. Documentation | 20 | Medium |
| **Total** | **~215 tasks** | **Medium-High** |

---

## Beyond Phase 5 (Phase 6+)

Future enhancements could include:

### Phase 6: Advanced Features
- Full exception handling (SEH/VEH)
- Complete TLS implementation
- DLL loading support
- Multi-threading
- More file system features

### Phase 7: GUI Support (Ambitious)
- Basic windowing (user32.dll)
- Graphics (GDI32.dll)
- Input handling
- Message loop

### Phase 8: Networking
- Winsock2 API
- TCP/IP stack
- Network drivers

### Phase 9: Security
- Process isolation
- Memory protection
- Sandboxing

---

## Conclusion

Phase 5 completes the core functionality needed to run moderately complex Windows binaries. After this phase, the unikernel can:

- Run binaries with dynamic memory allocation
- Process command line arguments
- Access environment variables
- Perform file I/O operations
- Handle basic exceptions (stubs)

This represents a fully functional Windows API compatibility layer suitable for many console applications!

See [FUTURE_ROADMAP.md](FUTURE_ROADMAP.md) for long-term vision.
