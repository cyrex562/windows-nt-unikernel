# Windows Environment Dependencies

**Purpose**: Document the environmental expectations of Windows binaries and strategies for satisfying them in a unikernel.

**Last Updated**: 2026-01-19

---

## Table of Contents

1. [Overview](#overview)
2. [Console Application Requirements](#console-application-requirements)
3. [GUI Application Requirements](#gui-application-requirements)
4. [System Services and Background Components](#system-services-and-background-components)
5. [File System Expectations](#file-system-expectations)
6. [Registry Dependencies](#registry-dependencies)
7. [Inter-Process Communication](#inter-process-communication)
8. [Dynamic Linking and DLL Dependencies](#dynamic-linking-and-dll-dependencies)
9. [Discovery and Compatibility Strategy](#discovery-and-compatibility-strategy)
10. [Implementation Priorities](#implementation-priorities)

---

## Overview

Windows binaries don't run in isolation—they expect a rich execution environment with numerous services, libraries, and system components. This document catalogs these dependencies and provides strategies for implementing or emulating them in the unikernel.

### Dependency Categories

**Critical** ⚠️ - Required for basic execution
**Important** 📋 - Needed for common functionality
**Optional** 💡 - Nice to have, app-specific
**Future** 🔮 - Long-term goal, not immediate

---

## Console Application Requirements

### Minimal Requirements (Phase 1-4)

**DLLs**:
- ⚠️ **kernel32.dll** - Core Windows API
  - File I/O: `ReadFile`, `WriteFile`, `CreateFile`, `CloseHandle`
  - Process: `ExitProcess`, `GetCommandLine`
  - Memory: `VirtualAlloc`, `VirtualFree`
  - Handles: `GetStdHandle`, `SetStdHandle`
  - Errors: `GetLastError`, `SetLastError`

**Environment**:
- ⚠️ **Standard handles**: stdin (0x10), stdout (0x11), stderr (0x12)
- ⚠️ **Process Environment Block (PEB)**: Contains image base, heap handle
- ⚠️ **Thread Environment Block (TEB)**: Contains last error, PEB pointer
- ⚠️ **Stack**: Properly aligned (16-byte), sufficient size (1MB+)

**Status**: Implemented in Phase 4

---

### Enhanced Console Apps (Phase 5)

**Additional kernel32.dll functions**:
- 📋 **Heap management**:
  - `GetProcessHeap`, `HeapAlloc`, `HeapFree`, `HeapReAlloc`
  - `HeapSize`, `HeapValidate`
- 📋 **File system**:
  - `CreateFileW`, `ReadFile`, `WriteFile`, `CloseHandle`
  - `GetFileSize`, `SetFilePointer`, `FlushFileBuffers`
  - `DeleteFile`, `CopyFile`, `MoveFile`
  - `CreateDirectory`, `RemoveDirectory`
  - `GetCurrentDirectory`, `SetCurrentDirectory`
  - `GetFileAttributes`, `SetFileAttributes`
- 📋 **Command line and environment**:
  - `GetCommandLineA/W`
  - `GetEnvironmentVariableA/W`, `SetEnvironmentVariableA/W`
  - `GetEnvironmentStrings`, `FreeEnvironmentStrings`
- 📋 **Time and date**:
  - `GetSystemTime`, `GetLocalTime`, `GetTickCount`, `GetTickCount64`
  - `FileTimeToSystemTime`, `SystemTimeToFileTime`
- 📋 **System information**:
  - `GetSystemInfo`, `GetComputerNameW`
  - `GetVersionExW`, `GetWindowsDirectoryW`

**Environment variables expected**:
```
PATH=C:\Windows\System32
TEMP=C:\Windows\Temp
TMP=C:\Windows\Temp
USERPROFILE=C:\Users\Default
COMPUTERNAME=UNIKERNEL
USERNAME=User
OS=Windows_NT
PROCESSOR_ARCHITECTURE=AMD64
NUMBER_OF_PROCESSORS=1
```

**File system structure** (minimal):
```
C:\
├── Windows\
│   ├── System32\      # System DLLs location
│   └── Temp\          # Temporary files
└── Users\
    └── Default\       # User profile
        └── AppData\
            └── Local\
```

**Status**: Planned for Phase 5

---

### Advanced Console Apps (Phase 6)

**Multi-threading**:
- 📋 `CreateThread`, `ExitThread`, `TerminateThread`
- 📋 `GetCurrentThreadId`, `GetThreadId`
- 📋 `SuspendThread`, `ResumeThread`
- 📋 `Sleep`, `SleepEx`
- 📋 `GetCurrentThread`, `GetCurrentProcess`

**Synchronization**:
- 📋 **Mutexes**: `CreateMutexW`, `OpenMutexW`, `ReleaseMutex`
- 📋 **Events**: `CreateEventW`, `SetEvent`, `ResetEvent`, `PulseEvent`
- 📋 **Semaphores**: `CreateSemaphoreW`, `ReleaseSemaphore`
- 📋 **Critical sections**: `InitializeCriticalSection`, `EnterCriticalSection`, `LeaveCriticalSection`, `DeleteCriticalSection`
- 📋 **Wait functions**: `WaitForSingleObject`, `WaitForMultipleObjects`

**Advanced file I/O**:
- 📋 **Async I/O**: `ReadFileEx`, `WriteFileEx`, `GetOverlappedResult`
- 📋 **File mapping**: `CreateFileMappingW`, `MapViewOfFile`, `UnmapViewOfFile`
- 📋 **Directory enumeration**: `FindFirstFileW`, `FindNextFileW`, `FindClose`

**Process management**:
- 📋 `CreateProcessW`, `WaitForSingleObject` (for process)
- 📋 `GetExitCodeProcess`, `TerminateProcess`

**Console I/O** (advanced):
- 💡 `ReadConsoleW`, `WriteConsoleW`
- 💡 `SetConsoleCursorPosition`, `GetConsoleScreenBufferInfo`
- 💡 `SetConsoleTextAttribute`

**Status**: Planned for Phase 6

---

## GUI Application Requirements

### Minimal GUI (Phase 7)

**Essential DLLs**:
- ⚠️ **user32.dll** - Window management and input
- ⚠️ **gdi32.dll** - Graphics Device Interface
- 📋 **comctl32.dll** - Common controls (optional for basic windows)

**user32.dll core functions**:
- ⚠️ **Window management**:
  - `RegisterClassExW` - Register window class
  - `CreateWindowExW` - Create window
  - `ShowWindow` - Show/hide window
  - `UpdateWindow` - Force redraw
  - `DestroyWindow` - Destroy window
  - `DefWindowProcW` - Default window procedure
- ⚠️ **Message loop**:
  - `GetMessageW` - Retrieve message from queue
  - `TranslateMessage` - Translate keyboard messages
  - `DispatchMessageW` - Send message to window procedure
  - `PostQuitMessage` - Exit message loop
- ⚠️ **Message handling**:
  - `SendMessageW` - Send message synchronously
  - `PostMessageW` - Post message asynchronously
- 📋 **Input**:
  - Mouse: `GetCursorPos`, `SetCursorPos`, `SetCapture`, `ReleaseCapture`
  - Keyboard: `GetKeyState`, `GetAsyncKeyState`
- 📋 **Utility**:
  - `GetClientRect`, `GetWindowRect`
  - `InvalidateRect` - Mark for redraw
  - `SetWindowTextW`, `GetWindowTextW`

**gdi32.dll core functions**:
- ⚠️ **Device context**:
  - `GetDC` - Get DC for window
  - `ReleaseDC` - Release DC
  - `BeginPaint`, `EndPaint` - For WM_PAINT
  - `CreateCompatibleDC` - For double buffering
- ⚠️ **Drawing**:
  - `SetPixel`, `GetPixel`
  - `MoveToEx`, `LineTo` - Line drawing
  - `Rectangle`, `Ellipse`, `Polygon`
  - `TextOutW` - Draw text
  - `BitBlt`, `StretchBlt` - Bitmap operations
- ⚠️ **Pens, brushes, fonts**:
  - `CreatePen`, `CreateSolidBrush`
  - `SelectObject`, `DeleteObject`
  - `CreateFontW`, `GetStockObject`
- ⚠️ **Colors**:
  - `SetTextColor`, `SetBkColor`
  - `RGB` macro (actually just a helper)

**Window Manager Requirements**:
- ⚠️ **Window class registry**: Track registered window classes
- ⚠️ **Window tree**: Parent-child relationships, Z-order
- ⚠️ **Message queue**: Per-thread message queue
- ⚠️ **Input routing**: Mouse/keyboard to correct window
- 📋 **Clipping**: Ensure windows don't draw outside bounds
- 📋 **Hit testing**: Determine what's under mouse cursor

**Graphics Subsystem**:
- ⚠️ **Framebuffer access**: Replace VGA text mode
  - Linear framebuffer (VESA/UEFI GOP)
  - Pixel format handling (RGB, BGR, etc.)
  - Resolution management
- ⚠️ **Font rendering**:
  - At least one bitmap font
  - Basic text rendering
- 📋 **Drawing primitives**:
  - Lines, rectangles, circles
  - Filled shapes
  - Bitmap blitting

**Input System**:
- ⚠️ **Keyboard driver**: PS/2 or USB keyboard
  - Scan code to virtual key translation
  - Keyboard layout (US English initially)
- ⚠️ **Mouse driver**: PS/2 or USB mouse
  - Relative movement → absolute position
  - Button state tracking

**Status**: Phase 7 (Ambitious)

---

### Advanced GUI (Phase 8+)

**Additional DLLs**:
- 📋 **comctl32.dll** - Common controls
  - Buttons, checkboxes, radio buttons
  - List views, tree views
  - Progress bars, sliders
  - Tab controls
- 📋 **comdlg32.dll** - Common dialogs
  - `GetOpenFileNameW` - File open dialog
  - `GetSaveFileNameW` - File save dialog
  - `ChooseColorW` - Color picker
  - `ChooseFontW` - Font selector
- 📋 **gdiplus.dll** - Enhanced graphics
  - Anti-aliased drawing
  - Alpha blending
  - Image loading (PNG, JPEG, GIF)
  - Gradients, textures
- 💡 **shell32.dll** - Shell functions
  - `ShellExecuteW` - Launch apps
  - `SHGetFolderPathW` - Get special folders
  - Icon extraction

**Advanced Features**:
- 📋 **Menu system**: `CreateMenu`, `AppendMenuW`, `TrackPopupMenu`
- 📋 **Dialogs**: `DialogBoxW`, `CreateDialogW`, custom dialogs
- 📋 **Clipboard**: `OpenClipboard`, `GetClipboardData`, `SetClipboardData`
- 📋 **Drag and drop**: OLE drag and drop (complex)
- 📋 **Timer**: `SetTimer`, `KillTimer`
- 💡 **Tooltips**: Tooltip controls
- 💡 **System tray**: Notification area icons

**Font System**:
- 📋 **Font loading**: TrueType/OpenType fonts
- 📋 **Font fallback**: When glyph not available
- 📋 **Text shaping**: For complex scripts (optional)

**Status**: Phase 8+ (Very Ambitious)

---

## System Services and Background Components

### Registry

**Why apps need it**:
- Configuration storage
- File associations (`.exe` → application)
- COM class registration
- Application settings
- System information queries

**Key registry hives**:
```
HKEY_LOCAL_MACHINE (HKLM)
├── SOFTWARE
│   ├── Microsoft
│   │   └── Windows
│   │       └── CurrentVersion
│   │           ├── ProgramFilesDir = "C:\Program Files"
│   │           └── SystemRoot = "C:\Windows"
│   └── Classes
│       └── .txt = "txtfile"
├── SYSTEM
│   └── CurrentControlSet
│       └── Control
│           └── ComputerName
│               └── ComputerName = "UNIKERNEL"
└── HARDWARE
    └── DESCRIPTION
        └── System
            └── CentralProcessor

HKEY_CURRENT_USER (HKCU)
├── Software
│   └── <App settings>
├── Environment
│   └── PATH, TEMP, etc.
└── Control Panel
    └── International
        └── Locale settings
```

**Implementation strategies**:
1. **In-memory registry**: Hash map structure (Phase 5)
2. **Registry files**: Parse Windows registry hives (Phase 6+)
3. **API stubbing**: Return reasonable defaults (Phase 5)

**Priority**: 📋 Important (Phase 5), Many apps query registry

---

### COM/RPC

**Component Object Model (COM)**:
- Binary interface standard
- Apps use for plugins, OLE, automation
- Can be embedded in single-process apps

**Functions**:
- `CoInitialize`, `CoUninitialize`
- `CoCreateInstance` - Create COM object
- `CoGetClassObject`
- Interface querying: `QueryInterface`

**Strategy**:
- 💡 Phase 6+: Stub for single-process COM
- 🔮 Future: Full COM runtime (complex)

**Priority**: 💡 Optional initially, 📋 Important for some apps

---

### Cryptographic Services

**CryptoAPI**:
- `CryptAcquireContext` - Get crypto provider
- `CryptGenRandom` - Generate random bytes
- `CryptHashData` - Hash data
- `CryptEncrypt`, `CryptDecrypt`

**Why apps use it**:
- Random number generation (common)
- Password hashing
- Data encryption
- Digital signatures

**Implementation**:
- Phase 5: `CryptGenRandom` using kernel PRNG
- Phase 6+: Full crypto API using Rust crypto crates

**Priority**: 📋 Important (at least random numbers)

---

### Time and Locale Services

**Time**:
- `GetSystemTime` - UTC time
- `GetLocalTime` - Local time
- `QueryPerformanceCounter` - High-resolution timer
- Timers: `SetTimer`, `CreateWaitableTimer`

**Implementation**:
- Use CPU timestamp counter (RDTSC)
- Maintain tick count since boot
- Fake current date/time

**Locale**:
- `GetLocaleInfoW` - Locale information
- `GetUserDefaultLCID` - User locale
- Character encoding conversions

**Priority**: 📋 Important (Phase 5-6)

---

### Windows Event Log

**Functions**:
- `RegisterEventSourceW`
- `ReportEventW`
- `DeregisterEventSource`

**Strategy**: Stub these (log to serial instead)

**Priority**: 💡 Optional (stub in Phase 5)

---

### Other Services

**Usually not needed for simple apps**:
- Windows Management Instrumentation (WMI)
- Background Intelligent Transfer Service (BITS)
- Windows Update
- Security services (LSA, SSPI) - unless doing authentication
- Service Control Manager (SCM) - for Windows services

**Strategy**: Return "not implemented" errors gracefully

---

## File System Expectations

### Directory Structure

**Critical directories**:
```
C:\                             # Root
├── Windows\                    # System directory
│   ├── System32\               # ⚠️ System DLLs (kernel32.dll, etc.)
│   │   └── drivers\            # Device drivers (if needed)
│   ├── SysWOW64\               # 32-bit DLLs (if supporting 32-bit)
│   ├── Fonts\                  # 📋 Font files (.ttf, .otf)
│   └── Temp\                   # ⚠️ Temporary files
├── Program Files\              # 📋 Application install location
├── Users\                      # User profiles
│   └── <Username>\
│       ├── AppData\
│       │   ├── Local\          # 📋 App-specific data
│       │   ├── Roaming\        # 📋 Roaming profile data
│       │   └── LocalLow\       # Low-integrity data
│       ├── Documents\          # 📋 User documents
│       ├── Desktop\
│       ├── Downloads\
│       └── Temp\               # 📋 User temp files
└── ProgramData\                # 📋 Shared app data
```

**Special paths queried by apps**:
- `%SystemRoot%` → `C:\Windows`
- `%ProgramFiles%` → `C:\Program Files`
- `%APPDATA%` → `C:\Users\<User>\AppData\Roaming`
- `%LOCALAPPDATA%` → `C:\Users\<User>\AppData\Local`
- `%TEMP%` → `C:\Windows\Temp` or user temp

**Implementation strategies**:

1. **Virtual file system (Phase 5)**:
   - In-memory file system
   - Redirect all paths to unikernel storage
   - Fake directory listings

2. **Real file system (Phase 6+)**:
   - FAT32, ext2, or custom FS
   - Persistent storage
   - Proper permissions

3. **Special files**:
   - `C:\Windows\System32\kernel32.dll` → Return fake metadata
   - System files → Pretend they exist

**Priority**: ⚠️ Critical (basic), 📋 Important (full)

---

### File Attributes and Metadata

**Attributes apps may query**:
- Archive, Hidden, System, Read-only
- Creation time, modification time, access time
- File size
- Short (8.3) filename

**Functions**:
- `GetFileAttributesW`, `SetFileAttributesW`
- `GetFileTime`, `SetFileTime`
- `GetShortPathNameW`

**Implementation**: Track attributes in file metadata

**Priority**: 📋 Important (Phase 5-6)

---

### Volume Information

**Functions**:
- `GetVolumeInformationW` - Volume label, filesystem type
- `GetDiskFreeSpaceExW` - Free space
- `GetLogicalDrives` - Drive bitmask

**Fake values**:
- Volume label: "UNIKERNEL"
- Filesystem: "NTFS"
- Total space: 1 GB
- Free space: 500 MB

**Priority**: 📋 Important (Phase 6)

---

## Registry Dependencies

### Common Registry Queries

**System information**:
```
HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion
├── ProductName = "Windows 10"
├── CurrentVersion = "10.0"
├── CurrentBuildNumber = "19045"
└── SystemRoot = "C:\Windows"
```

**File associations**:
```
HKLM\SOFTWARE\Classes
├── .txt = "txtfile"
├── txtfile
│   ├── (Default) = "Text Document"
│   └── shell
│       └── open
│           └── command
│               └── (Default) = "notepad.exe %1"
```

**Application settings**:
```
HKCU\Software\<Vendor>\<AppName>
└── <Various settings>
```

**Environment variables** (also in registry):
```
HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment
└── PATH, TEMP, etc.
```

### Implementation Strategy

**Phase 5 (Minimal)**:
```rust
pub struct FakeRegistry {
    data: HashMap<String, HashMap<String, String>>,
}

impl FakeRegistry {
    pub fn new() -> Self {
        let mut reg = Self { data: HashMap::new() };

        // Pre-populate critical keys
        reg.set("HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion",
                "ProductName", "Windows 10");
        reg.set("HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion",
                "SystemRoot", "C:\\Windows");

        reg
    }

    pub fn query(&self, key: &str, value: &str) -> Option<&str> {
        self.data.get(key)?.get(value).map(|s| s.as_str())
    }
}
```

**Phase 6+ (Advanced)**:
- Parse real registry hives
- Support data types (REG_SZ, REG_DWORD, REG_BINARY)
- Persistence

**Priority**: 📋 Important (Phase 5)

---

## Inter-Process Communication

### Named Objects

**Mutexes**:
- `CreateMutexW(NULL, FALSE, L"Global\\MyAppMutex")`
- Used for single-instance apps
- Must be tracked globally

**Events**:
- `CreateEventW(NULL, FALSE, FALSE, L"MyEvent")`
- Synchronization between threads/processes

**Semaphores**:
- `CreateSemaphoreW(NULL, 1, 10, L"MySemaphore")`
- Resource counting

**File mappings**:
- `CreateFileMappingW(..., L"Global\\MySharedMem")`
- Shared memory between processes

**Implementation**:
- Track named objects in kernel
- Single process → most IPC unnecessary
- Can stub for compatibility

**Priority**: 📋 Important for multi-process (Phase 6)

---

### Pipes

**Anonymous pipes**:
- `CreatePipe` - For parent-child communication
- Used with `CreateProcess` for stdin/stdout redirection

**Named pipes**:
- `CreateNamedPipeW(L"\\\\.\\pipe\\MyPipe", ...)`
- IPC mechanism
- Server-client model

**Implementation**: Phase 6+ (for process launching)

**Priority**: 💡 Optional initially

---

### Sockets

**Winsock2**:
- `WSAStartup`, `WSACleanup`
- `socket`, `bind`, `listen`, `accept`, `connect`
- `send`, `recv`, `sendto`, `recvfrom`
- `select`, `WSAPoll`

**Implementation**: Phase 8+ (networking)

**Priority**: 🔮 Future

---

## Dynamic Linking and DLL Dependencies

### LoadLibrary/GetProcAddress

**Runtime DLL loading**:
```c
HMODULE dll = LoadLibraryW(L"user32.dll");
FARPROC func = GetProcAddress(dll, "MessageBoxW");
```

**Implementation strategy**:

**Phase 5-6**: Pre-registered DLLs
```rust
pub struct DllRegistry {
    dlls: HashMap<String, DllInfo>,
}

pub struct DllInfo {
    name: String,
    exports: HashMap<String, usize>,  // Function name → address
}

impl DllRegistry {
    pub fn load_library(&self, name: &str) -> Option<usize> {
        // Return fake handle (index into dlls map)
        self.dlls.get(name).map(|_| name.as_ptr() as usize)
    }

    pub fn get_proc_address(&self, handle: usize, name: &str) -> Option<usize> {
        // Look up function by name in the DLL's export table
        let dll_name = unsafe { /* extract from handle */ };
        self.dlls.get(dll_name)?.exports.get(name).copied()
    }
}
```

**Phase 7+**: Dynamic PE loading
- Load DLL PE files
- Parse exports
- Support forwarding

**Priority**: 📋 Important (Phase 6)

---

### DLL Dependency Chains

**Common dependencies**:
```
application.exe
├── kernel32.dll (our implementation)
│   └── ntdll.dll (our implementation)
├── user32.dll (Phase 7+)
│   ├── gdi32.dll
│   └── kernel32.dll
└── advapi32.dll (Phase 6+)
    └── kernel32.dll
```

**Circular dependencies**: Windows DLLs often have circular imports
- Requires careful initialization order
- May need lazy binding

**Strategy**:
- Phase 5: Only kernel32.dll
- Phase 6: Add advapi32.dll (registry, security)
- Phase 7: Add user32.dll, gdi32.dll (GUI)
- Phase 8: Add more as needed

---

### Import by Ordinal

Some apps import functions by ordinal instead of name:
```c
// Import table entry: kernel32.dll ordinal 123
```

**Implementation**:
- Maintain ordinal-to-name mapping
- Common for system DLLs
- ReactOS/Wine have ordinal lists

**Priority**: 💡 Optional (Phase 6+)

---

## Discovery and Compatibility Strategy

### Missing Import Tracking

**Implementation**:
```rust
pub struct MissingImportTracker {
    missing: Arc<Mutex<HashMap<String, Vec<String>>>>,  // DLL → [functions]
}

impl SymbolResolver {
    pub fn resolve_or_stub(&mut self, dll: &str, func: &str) -> usize {
        if let Some(addr) = self.resolve(dll, func) {
            return addr;
        }

        // Track missing import
        self.missing_tracker.add(dll, func);

        // Return logging stub
        missing_api_stub as usize
    }
}

extern "C" fn missing_api_stub() -> usize {
    // Log which function was called (can be determined via return address)
    serial_println!("STUB: Missing API function called!");
    // Return safe default
    0
}
```

**Benefits**:
- Discover required APIs by running real binaries
- Graceful degradation
- Prioritize implementation by frequency of use

---

### Progressive Compatibility

**Testing binaries** (in order of complexity):

1. **Phase 4-5**: Simple console apps
   - `hello.exe` - Hello world
   - `echo.exe` - Echo arguments
   - Custom test apps

2. **Phase 5**: Console utilities
   - `busybox-w32` - Unix utilities for Windows
   - `cat.exe`, `grep.exe` - Text processing
   - `curl.exe` - HTTP client (needs Winsock)

3. **Phase 6**: Advanced console
   - `tcc.exe` - Tiny C Compiler
   - `sqlite3.exe` - Database
   - `7z.exe` - Archiver

4. **Phase 7**: Simple GUI
   - Custom test GUI apps
   - Minimal Win32 apps
   - `notepad.exe` (might be too complex)

5. **Phase 8+**: Complex GUI
   - `notepad++.exe` - Text editor (uses modern controls)
   - `putty.exe` - SSH client
   - Simple games

**Incremental approach**:
- Run binary
- Note missing imports
- Implement most critical functions
- Repeat

---

### API Compatibility Matrix

Track compatibility for each binary:

| Binary | Phase | kernel32 | user32 | gdi32 | advapi32 | Status |
|--------|-------|----------|--------|-------|----------|--------|
| target-zero.exe | 4 | ✅ | ❌ | ❌ | ❌ | ✅ Working |
| busybox.exe | 5 | ⚠️ | ❌ | ❌ | ❌ | 🚧 Partial |
| tcc.exe | 6 | ⚠️ | ❌ | ❌ | ⚠️ | 📋 Planned |
| notepad.exe | 8 | ✅ | ⚠️ | ⚠️ | ⚠️ | 🔮 Future |

Legend:
- ✅ Fully implemented
- ⚠️ Partially implemented
- ❌ Not implemented
- 📋 Planned
- 🔮 Future goal

---

## Implementation Priorities

### Phase 4-5: Console Foundation
- ✅ kernel32.dll core functions
- ✅ Heap management
- ✅ File I/O (basic)
- ✅ Environment variables
- ✅ Command line processing
- 📋 Fake registry (minimal)

### Phase 6: Advanced Console
- 📋 Multi-threading
- 📋 Synchronization primitives
- 📋 Process creation
- 📋 Registry (expanded)
- 📋 More file operations
- 📋 advapi32.dll (basic)
- 💡 Dynamic DLL loading

### Phase 7: Basic GUI
- 📋 user32.dll (core window management)
- 📋 gdi32.dll (basic drawing)
- 📋 Framebuffer graphics
- 📋 Keyboard/mouse drivers
- 📋 Message loop

### Phase 8+: Advanced GUI
- 💡 Common controls (comctl32.dll)
- 💡 Common dialogs (comdlg32.dll)
- 💡 Enhanced graphics (gdiplus.dll)
- 💡 Clipboard, drag-and-drop
- 💡 Menus, dialogs, icons

### Phase 9+: Networking
- 🔮 Winsock2 API
- 🔮 TCP/IP stack
- 🔮 Network drivers

### Phase 10+: Exotic Features
- 🔮 Full SEH/VEH exception handling
- 🔮 COM runtime
- 🔮 Security/authentication
- 🔮 More DLLs as needed

---

## Summary

### What We Must Implement

**Absolutely critical** (Phase 4-5):
1. kernel32.dll core API
2. TEB/PEB structures
3. Basic file system
4. Minimal registry stub
5. Standard handles and console I/O

**Important for real apps** (Phase 6):
1. Multi-threading and synchronization
2. Heap management (beyond basic)
3. More file operations
4. Registry (expanded)
5. Dynamic DLL loading

**For GUI apps** (Phase 7+):
1. user32.dll window management
2. gdi32.dll drawing
3. Framebuffer graphics mode
4. Input drivers (keyboard, mouse)
5. Font rendering

### What We Can Fake/Stub

**Can provide fake values**:
- Registry queries (return reasonable defaults)
- System information (fake version, computer name, etc.)
- File existence checks (pretend system files exist)
- Volume information (fake disk space)

**Can stub (no-op)**:
- Event log functions
- Performance counters
- WMI queries
- Exotic APIs rarely used

**Can implement minimally**:
- Time functions (fake current time)
- Cryptographic functions (just random numbers initially)
- Locale functions (US English only)

### Discovery is Key

Don't try to implement everything upfront. Instead:
1. Run real binaries
2. Track missing imports
3. Implement most-requested APIs first
4. Iterate based on actual usage

This lets you focus effort where it matters most.

---

## See Also

- [DESIGN.md](DESIGN.md) - System architecture
- [PHASE5.md](PHASE5.md) - Expansion features
- [PHASE6.md](PHASE6.md) - Advanced console features
- [PHASE7.md](PHASE7.md) - GUI support
- [AI_AGENT_GUIDE.md](AI_AGENT_GUIDE.md) - Navigation guide

---

**Last updated**: 2026-01-19
