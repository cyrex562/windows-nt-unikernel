# Phase 6: Advanced Console Features - Detailed Task Checklist

**Goal**: Expand beyond basic console I/O to support multi-threading, advanced file operations, registry simulation, dynamic DLL loading, and process creation. Target more complex console applications.

**Location**: `crates/api-shim/` and `crates/kernel/`

**Success Criteria**: Run complex console applications like Tiny C Compiler (TCC), SQLite shell, or 7-Zip that require multi-threading, file system operations, and registry access.

---

## 1. Multi-Threading Support

### 1.1 Thread Management Infrastructure
**File**: `crates/kernel/src/threading/mod.rs`

- [ ] **1.1.1** Design thread control block (TCB) structure:
  ```rust
  pub struct Thread {
      pub id: u32,
      pub teb: *mut TEB,
      pub stack_base: *mut u8,
      pub stack_size: usize,
      pub state: ThreadState,  // Running, Ready, Blocked
      pub context: ThreadContext,  // Saved registers
      pub exit_code: Option<u32>,
  }
  ```
- [ ] **1.1.2** Create global thread table
- [ ] **1.1.3** Implement thread ID allocation
- [ ] **1.1.4** Create thread state machine (Running, Ready, Blocked, Terminated)
- [ ] **1.1.5** Log thread subsystem initialization

### 1.2 CreateThread Implementation
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **1.2.1** Define function signature:
  ```rust
  #[no_mangle]
  pub extern "C" fn CreateThread(
      security_attributes: *mut u8,
      stack_size: usize,
      start_address: extern "C" fn(*mut u8) -> u32,
      parameter: *mut u8,
      creation_flags: DWORD,
      thread_id: *mut DWORD,
  ) -> HANDLE
  ```
- [ ] **1.2.2** Allocate new thread ID
- [ ] **1.2.3** Allocate stack for new thread (default 1 MB if stack_size == 0)
- [ ] **1.2.4** Create new TEB for thread
- [ ] **1.2.5** Set up thread context (RIP = start_address, RCX = parameter)
- [ ] **1.2.6** Handle creation flags (CREATE_SUSPENDED)
- [ ] **1.2.7** Add thread to ready queue
- [ ] **1.2.8** Return thread handle
- [ ] **1.2.9** Set thread_id output parameter
- [ ] **1.2.10** Test thread creation

### 1.3 Thread Scheduler
**File**: `crates/kernel/src/threading/scheduler.rs`

- [ ] **1.3.1** Implement round-robin scheduler
- [ ] **1.3.2** Create ready queue (VecDeque of thread IDs)
- [ ] **1.3.3** Implement `schedule()` function (pick next thread)
- [ ] **1.3.4** Implement context switch:
  - [ ] Save current thread context (all registers)
  - [ ] Load next thread context
  - [ ] Switch stacks (RSP)
  - [ ] Switch TEB (GS register)
- [ ] **1.3.5** Set up timer interrupt for preemptive scheduling
- [ ] **1.3.6** Implement `yield()` function (voluntary context switch)
- [ ] **1.3.7** Test scheduling with multiple threads

### 1.4 Thread Termination
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **1.4.1** Implement ExitThread:
  ```rust
  #[no_mangle]
  pub extern "C" fn ExitThread(exit_code: DWORD) -> !
  ```
- [ ] **1.4.2** Set thread exit code
- [ ] **1.4.3** Mark thread as terminated
- [ ] **1.4.4** Remove from ready queue
- [ ] **1.4.5** Free thread resources (stack, TEB)
- [ ] **1.4.6** Trigger scheduler
- [ ] **1.4.7** Implement GetExitCodeThread
- [ ] **1.4.8** Test thread termination

### 1.5 Thread Utilities
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **1.5.1** Implement GetCurrentThreadId
- [ ] **1.5.2** Implement GetCurrentThread (return pseudo-handle)
- [ ] **1.5.3** Implement Sleep:
  ```rust
  pub extern "C" fn Sleep(milliseconds: DWORD)
  ```
- [ ] **1.5.4** Implement SleepEx (alertable sleep)
- [ ] **1.5.5** Implement SwitchToThread (yield)
- [ ] **1.5.6** Test utility functions

---

## 2. Synchronization Primitives

### 2.1 Critical Sections
**File**: `crates/api-shim/src/sync/critical_section.rs`

- [ ] **2.1.1** Define CRITICAL_SECTION structure:
  ```rust
  #[repr(C)]
  pub struct CRITICAL_SECTION {
      lock_count: i32,
      recursion_count: i32,
      owning_thread: u32,
      spin_count: u32,
  }
  ```
- [ ] **2.1.2** Implement InitializeCriticalSection
- [ ] **2.1.3** Implement EnterCriticalSection:
  - [ ] If unlocked, acquire
  - [ ] If owned by current thread, increment recursion
  - [ ] If owned by other thread, block and wait
- [ ] **2.1.4** Implement LeaveCriticalSection:
  - [ ] Decrement recursion count
  - [ ] If zero, release lock
  - [ ] Wake one waiting thread
- [ ] **2.1.5** Implement DeleteCriticalSection
- [ ] **2.1.6** Implement TryEnterCriticalSection
- [ ] **2.1.7** Test critical sections with multiple threads

### 2.2 Mutexes
**File**: `crates/api-shim/src/sync/mutex.rs`

- [ ] **2.2.1** Create mutex tracking structure in kernel
- [ ] **2.2.2** Implement CreateMutexW:
  ```rust
  pub extern "C" fn CreateMutexW(
      security_attributes: *mut u8,
      initial_owner: BOOL,
      name: *const u16,
  ) -> HANDLE
  ```
- [ ] **2.2.3** Support named mutexes (track by name)
- [ ] **2.2.4** Implement WaitForSingleObject for mutexes:
  - [ ] If available, acquire and return immediately
  - [ ] If held by other thread, block
  - [ ] Support timeout
- [ ] **2.2.5** Implement ReleaseMutex
- [ ] **2.2.6** Implement OpenMutexW (open existing named mutex)
- [ ] **2.2.7** Test mutexes with multiple threads

### 2.3 Events
**File**: `crates/api-shim/src/sync/event.rs`

- [ ] **2.3.1** Create event tracking structure
- [ ] **2.3.2** Implement CreateEventW:
  ```rust
  pub extern "C" fn CreateEventW(
      security_attributes: *mut u8,
      manual_reset: BOOL,
      initial_state: BOOL,
      name: *const u16,
  ) -> HANDLE
  ```
- [ ] **2.3.3** Support manual-reset and auto-reset events
- [ ] **2.3.4** Support named events
- [ ] **2.3.5** Implement SetEvent (signal)
- [ ] **2.3.6** Implement ResetEvent (unsignal)
- [ ] **2.3.7** Implement PulseEvent
- [ ] **2.3.8** Implement WaitForSingleObject for events
- [ ] **2.3.9** Test events with multiple threads

### 2.4 Semaphores
**File**: `crates/api-shim/src/sync/semaphore.rs`

- [ ] **2.4.1** Create semaphore tracking structure
- [ ] **2.4.2** Implement CreateSemaphoreW:
  ```rust
  pub extern "C" fn CreateSemaphoreW(
      security_attributes: *mut u8,
      initial_count: i32,
      maximum_count: i32,
      name: *const u16,
  ) -> HANDLE
  ```
- [ ] **2.4.3** Implement WaitForSingleObject for semaphores (decrement count)
- [ ] **2.4.4** Implement ReleaseSemaphore (increment count, wake waiters)
- [ ] **2.4.5** Test semaphores with producer/consumer pattern

### 2.5 Wait Functions
**File**: `crates/api-shim/src/sync/wait.rs`

- [ ] **2.5.1** Implement WaitForSingleObject:
  ```rust
  pub extern "C" fn WaitForSingleObject(
      handle: HANDLE,
      milliseconds: DWORD,
  ) -> DWORD
  ```
- [ ] **2.5.2** Support different handle types (mutex, event, semaphore, thread)
- [ ] **2.5.3** Support INFINITE timeout
- [ ] **2.5.4** Support timed waits
- [ ] **2.5.5** Return WAIT_OBJECT_0, WAIT_TIMEOUT, or WAIT_FAILED
- [ ] **2.5.6** Implement WaitForMultipleObjects (wait for array of handles)
- [ ] **2.5.7** Support wait-all and wait-any modes
- [ ] **2.5.8** Test wait functions

---

## 3. Advanced File Operations

### 3.1 File Seeking and Size
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **3.1.1** Implement SetFilePointer:
  ```rust
  pub extern "C" fn SetFilePointer(
      file: HANDLE,
      distance_to_move: i32,
      distance_to_move_high: *mut i32,
      move_method: DWORD,
  ) -> DWORD
  ```
- [ ] **3.1.2** Support FILE_BEGIN, FILE_CURRENT, FILE_END
- [ ] **3.1.3** Handle 64-bit file positions
- [ ] **3.1.4** Implement SetFilePointerEx (64-bit version)
- [ ] **3.1.5** Implement GetFileSize
- [ ] **3.1.6** Implement GetFileSizeEx
- [ ] **3.1.7** Test seeking in files

### 3.2 File Attributes and Times
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **3.2.1** Implement GetFileAttributesW
- [ ] **3.2.2** Implement SetFileAttributesW
- [ ] **3.2.3** Support attributes: READ_ONLY, HIDDEN, SYSTEM, ARCHIVE
- [ ] **3.2.4** Implement GetFileTime (creation, access, write times)
- [ ] **3.2.5** Implement SetFileTime
- [ ] **3.2.6** Implement GetFileInformationByHandle
- [ ] **3.2.7** Test attribute and time functions

### 3.3 Directory Enumeration
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **3.3.1** Implement FindFirstFileW:
  ```rust
  pub extern "C" fn FindFirstFileW(
      file_name: *const u16,
      find_file_data: *mut WIN32_FIND_DATAW,
  ) -> HANDLE
  ```
- [ ] **3.3.2** Support wildcards (* and ?)
- [ ] **3.3.3** Implement FindNextFileW
- [ ] **3.3.4** Implement FindClose
- [ ] **3.3.5** Populate WIN32_FIND_DATAW structure
- [ ] **3.3.6** Test directory enumeration

### 3.4 File Operations
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **3.4.1** Implement DeleteFileW
- [ ] **3.4.2** Implement CopyFileW
- [ ] **3.4.3** Implement MoveFileW / MoveFileExW
- [ ] **3.4.4** Implement CreateDirectoryW
- [ ] **3.4.5** Implement RemoveDirectoryW
- [ ] **3.4.6** Implement GetCurrentDirectoryW
- [ ] **3.4.7** Implement SetCurrentDirectoryW
- [ ] **3.4.8** Test file operations

### 3.5 File Mapping (Memory-Mapped Files)
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **3.5.1** Implement CreateFileMappingW:
  ```rust
  pub extern "C" fn CreateFileMappingW(
      file: HANDLE,
      attributes: *mut u8,
      protect: DWORD,
      maximum_size_high: DWORD,
      maximum_size_low: DWORD,
      name: *const u16,
  ) -> HANDLE
  ```
- [ ] **3.5.2** Support PAGE_READONLY, PAGE_READWRITE protections
- [ ] **3.5.3** Implement MapViewOfFile
- [ ] **3.5.4** Map file contents to virtual memory
- [ ] **3.5.5** Implement UnmapViewOfFile
- [ ] **3.5.6** Support named file mappings (shared memory)
- [ ] **3.5.7** Test file mapping

---

## 4. Registry Simulation

### 4.1 Registry Infrastructure
**File**: `crates/kernel/src/registry/mod.rs`

- [ ] **4.1.1** Design registry key/value structure:
  ```rust
  pub struct RegistryKey {
      name: String,
      values: HashMap<String, RegistryValue>,
      subkeys: HashMap<String, RegistryKey>,
  }

  pub enum RegistryValue {
      String(String),      // REG_SZ
      DWord(u32),          // REG_DWORD
      Binary(Vec<u8>),     // REG_BINARY
      MultiString(Vec<String>),  // REG_MULTI_SZ
  }
  ```
- [ ] **4.1.2** Create root hives (HKLM, HKCU, HKCR)
- [ ] **4.1.3** Pre-populate common keys
- [ ] **4.1.4** Implement key path parsing

### 4.2 Registry API Implementation
**File**: `crates/api-shim/src/advapi32.rs`

- [ ] **4.2.1** Implement RegOpenKeyExW:
  ```rust
  pub extern "C" fn RegOpenKeyExW(
      key: HKEY,
      sub_key: *const u16,
      options: DWORD,
      desired: REGSAM,
      result: *mut HKEY,
  ) -> i32
  ```
- [ ] **4.2.2** Implement RegCloseKey
- [ ] **4.2.3** Implement RegQueryValueExW:
  ```rust
  pub extern "C" fn RegQueryValueExW(
      key: HKEY,
      value_name: *const u16,
      reserved: *mut DWORD,
      type_: *mut DWORD,
      data: *mut u8,
      cb_data: *mut DWORD,
  ) -> i32
  ```
- [ ] **4.2.4** Implement RegSetValueExW
- [ ] **4.2.5** Implement RegCreateKeyExW
- [ ] **4.2.6** Implement RegDeleteKeyW
- [ ] **4.2.7** Implement RegDeleteValueW
- [ ] **4.2.8** Implement RegEnumKeyExW (enumerate subkeys)
- [ ] **4.2.9** Implement RegEnumValueW (enumerate values)

### 4.3 Common Registry Data
**File**: `crates/kernel/src/registry/defaults.rs`

- [ ] **4.3.1** Populate system version info:
  ```
  HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion
  ├── ProductName = "Windows 10"
  ├── CurrentVersion = "10.0"
  ├── CurrentBuildNumber = "19045"
  └── SystemRoot = "C:\Windows"
  ```
- [ ] **4.3.2** Populate file associations:
  ```
  HKCR\.txt = "txtfile"
  HKCR\txtfile\shell\open\command = "notepad.exe %1"
  ```
- [ ] **4.3.3** Populate environment variables (also in registry)
- [ ] **4.3.4** Populate hardware info
- [ ] **4.3.5** Test registry queries with real apps

---

## 5. Dynamic DLL Loading

### 5.1 DLL Registry
**File**: `crates/kernel/src/dll/registry.rs`

- [ ] **5.1.1** Create DLL info structure:
  ```rust
  pub struct DllInfo {
      name: String,
      base_address: usize,
      exports: HashMap<String, usize>,
      ordinals: HashMap<u16, usize>,
  }
  ```
- [ ] **5.1.2** Pre-register kernel32.dll
- [ ] **5.1.3** Pre-register ntdll.dll
- [ ] **5.1.4** Pre-register advapi32.dll
- [ ] **5.1.5** Pre-register user32.dll (Phase 7)
- [ ] **5.1.6** Pre-register gdi32.dll (Phase 7)

### 5.2 LoadLibrary Implementation
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **5.2.1** Implement LoadLibraryW:
  ```rust
  pub extern "C" fn LoadLibraryW(lib_name: *const u16) -> HMODULE
  ```
- [ ] **5.2.2** Convert library name to string
- [ ] **5.2.3** Look up in DLL registry
- [ ] **5.2.4** If not found, check if we should load it
- [ ] **5.2.5** Return module handle (DLL base address or fake handle)
- [ ] **5.2.6** Implement LoadLibraryExW
- [ ] **5.2.7** Support LOAD_LIBRARY_AS_DATAFILE flag
- [ ] **5.2.8** Test LoadLibrary

### 5.3 GetProcAddress Implementation
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **5.3.1** Implement GetProcAddress:
  ```rust
  pub extern "C" fn GetProcAddress(
      module: HMODULE,
      proc_name: *const u8,
  ) -> FARPROC
  ```
- [ ] **5.3.2** Handle proc_name as string or ordinal (low word set)
- [ ] **5.3.3** Look up module in DLL registry
- [ ] **5.3.4** Look up function in export table
- [ ] **5.3.5** Return function address
- [ ] **5.3.6** Return NULL if not found
- [ ] **5.3.7** Test GetProcAddress

### 5.4 FreeLibrary Implementation
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **5.4.1** Implement FreeLibrary:
  ```rust
  pub extern "C" fn FreeLibrary(module: HMODULE) -> BOOL
  ```
- [ ] **5.4.2** For pre-registered DLLs, just return TRUE
- [ ] **5.4.3** For dynamically loaded DLLs (future), free resources
- [ ] **5.4.4** Test FreeLibrary

---

## 6. Process Management (Basic)

### 6.1 Process Creation (Stub)
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **6.1.1** Stub CreateProcessW:
  ```rust
  pub extern "C" fn CreateProcessW(
      application_name: *const u16,
      command_line: *mut u16,
      process_attributes: *mut u8,
      thread_attributes: *mut u8,
      inherit_handles: BOOL,
      creation_flags: DWORD,
      environment: *mut u8,
      current_directory: *const u16,
      startup_info: *mut STARTUPINFOW,
      process_information: *mut PROCESS_INFORMATION,
  ) -> BOOL
  ```
- [ ] **6.1.2** Log parameters
- [ ] **6.1.3** Return FALSE (not yet implemented)
- [ ] **6.1.4** Set last error to ERROR_NOT_SUPPORTED
- [ ] **6.1.5** Note: Full implementation is complex (Phase 7+)

### 6.2 Current Process Information
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **6.2.1** Implement GetCurrentProcessId (return fake PID)
- [ ] **6.2.2** Implement GetCurrentProcess (return pseudo-handle)
- [ ] **6.2.3** Implement GetProcessId
- [ ] **6.2.4** Test functions

---

## 7. Advanced Time Functions

### 7.1 High-Resolution Timers
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **7.1.1** Implement QueryPerformanceCounter:
  ```rust
  pub extern "C" fn QueryPerformanceCounter(
      performance_count: *mut i64,
  ) -> BOOL
  ```
- [ ] **7.1.2** Use RDTSC instruction or PIT/HPET
- [ ] **7.1.3** Implement QueryPerformanceFrequency
- [ ] **7.1.4** Implement GetTickCount64
- [ ] **7.1.5** Test high-resolution timers

### 7.2 System Time
**File**: `crates/api-shim/src/kernel32.rs`

- [ ] **7.2.1** Implement GetSystemTimeAsFileTime
- [ ] **7.2.2** Implement FileTimeToSystemTime
- [ ] **7.2.3** Implement SystemTimeToFileTime
- [ ] **7.2.4** Implement GetTimeZoneInformation (return UTC)
- [ ] **7.2.5** Test time conversions

---

## 8. Testing with Complex Applications

### 8.1 Test Application Selection

- [ ] **8.1.1** Download/build busybox-w32
- [ ] **8.1.2** Download/build Tiny C Compiler (TCC)
- [ ] **8.1.3** Download/build SQLite shell
- [ ] **8.1.4** Download/build 7-Zip console version

### 8.2 Integration Testing

- [ ] **8.2.1** Test busybox utilities (ls, cat, grep, etc.)
- [ ] **8.2.2** Test TCC compilation
- [ ] **8.2.3** Test SQLite database operations
- [ ] **8.2.4** Test 7-Zip compression/decompression
- [ ] **8.2.5** Document compatibility issues
- [ ] **8.2.6** Implement missing APIs discovered

### 8.3 Multi-Threading Tests

- [ ] **8.3.1** Create test with multiple threads
- [ ] **8.3.2** Test thread synchronization
- [ ] **8.3.3** Test concurrent file access
- [ ] **8.3.4** Stress test with many threads
- [ ] **8.3.5** Verify no race conditions

---

## 9. Documentation and Cleanup

### 9.1 API Documentation

- [ ] **9.1.1** Document all new API functions
- [ ] **9.1.2** Document threading model
- [ ] **9.1.3** Document synchronization primitives
- [ ] **9.1.4** Document registry structure
- [ ] **9.1.5** Document file operations

### 9.2 Architecture Documentation

- [ ] **9.2.1** Document scheduler design
- [ ] **9.2.2** Document context switch mechanism
- [ ] **9.2.3** Document synchronization algorithms
- [ ] **9.2.4** Create diagrams for complex subsystems

---

## Success Criteria Checklist

Phase 6 is complete when:

- [ ] **S1** Multi-threading works (CreateThread, synchronization)
- [ ] **S2** Thread scheduler preempts threads
- [ ] **S3** Critical sections, mutexes, events, semaphores work
- [ ] **S4** Advanced file operations work (seek, attributes, mapping)
- [ ] **S5** Directory enumeration works
- [ ] **S6** Registry API is functional
- [ ] **S7** Common registry keys are populated
- [ ] **S8** LoadLibrary/GetProcAddress work
- [ ] **S9** At least one complex app (TCC, SQLite, or busybox) runs
- [ ] **S10** Multi-threaded test apps work correctly
- [ ] **S11** No race conditions or deadlocks in tests
- [ ] **S12** Documentation is complete

---

## Estimated Task Breakdown

| Section | Tasks | Estimated Complexity |
|---------|-------|---------------------|
| 1. Multi-Threading | 35 | High |
| 2. Synchronization | 45 | High |
| 3. Advanced File I/O | 40 | Medium-High |
| 4. Registry | 30 | Medium |
| 5. Dynamic DLL Loading | 20 | Medium |
| 6. Process Management | 5 | Low (stubs) |
| 7. Advanced Time | 10 | Low-Medium |
| 8. Testing | 15 | Medium |
| 9. Documentation | 15 | Low-Medium |
| **Total** | **~215 tasks** | **High** |

---

## Next Steps

After Phase 6:
- Move to Phase 7: GUI support (user32.dll, gdi32.dll)
- Or continue expanding console features based on app requirements

See [PHASE7.md](PHASE7.md) for GUI implementation details.
