# Windows NT Unikernel Test Fixtures

Recommended Approach: Cross-Compilation from Linux
Why Cross-Compilation is Better
✅ Keep everything in one environment (Linux)
✅ Already set up (MinGW-w64 in target-zero/Makefile)
✅ Better CI/CD integration
✅ Consistent with bare-metal development workflow

You can build Windows binaries entirely from Linux using MinGW-w64, which is a complete Windows development toolchain that runs on Linux.

Test Fixture Structure
Here's how I recommend organizing test binaries:

```
windows-nt-unikernel/
├── fixtures/                    # Test binaries directory
│   ├── README.md               # Documentation for fixtures
│   ├── Makefile                # Build all fixtures
│   │
│   ├── hello-world/            # Simplest possible test
│   │   ├── hello.c
│   │   ├── Makefile
│   │   └── hello.exe (built)
│   │
│   ├── target-zero/            # Basic I/O test (current)
│   │   ├── target-zero.c
│   │   ├── Makefile
│   │   └── target-zero.exe
│   │
│   ├── heap-test/              # Heap allocation test
│   │   ├── heap-test.c
│   │   ├── Makefile
│   │   └── heap-test.exe
│   │
│   ├── file-io-test/           # File I/O test
│   │   ├── file-io.c
│   │   ├── Makefile
│   │   └── file-io.exe
│   │
│   ├── multi-thread-test/      # Threading test
│   │   ├── threads.c
│   │   ├── Makefile
│   │   └── threads.exe
│   │
│   ├── registry-test/          # Registry test
│   │   ├── registry.c
│   │   ├── Makefile
│   │   └── registry.exe
│   │
│   ├── rust-fixtures/          # Rust-based test binaries
│   │   ├── Cargo.toml
│   │   ├── rust-hello/
│   │   │   └── src/main.rs
│   │   └── rust-complex/
│   │       └── src/main.rs
│   │
│   └── external/               # Third-party binaries for testing
│       ├── busybox-w32.exe
│       ├── tcc.exe
│       └── sqlite3.exe
│
└── target-zero/                # Keep for backward compatibility
    └── (move to fixtures/ eventually)
```

Setting Up Cross-Compilation
Install MinGW-w64
Already in your justfile:

just install-mingw

Or manually:

# Debian/Ubuntu
sudo apt-get install mingw-w64

# Fedora/RHEL
sudo dnf install mingw64-gcc

# Arch Linux
sudo pacman -S mingw-w64-gcc

Verify Installation
x86_64-w64-mingw32-gcc --version
# Should output: x86_64-w64-mingw32-gcc (GCC) ...

Writing C Test Fixtures
Example: fixtures/hello-world/hello.c
#include <windows.h>

int main(void) {
    HANDLE hStdOut = GetStdHandle(STD_OUTPUT_HANDLE);
    const char* message = "Hello from MinGW!\n";
    DWORD written;
    
    WriteFile(hStdOut, message, 19, &written, NULL);
    
    ExitProcess(0);
    return 0;  // Never reached
}

Example: fixtures/hello-world/Makefile
CC = x86_64-w64-mingw32-gcc
CFLAGS = -Wall -Wextra -O2 -static -municode
LDFLAGS = -Wl,--subsystem,console
LIBS = -lkernel32

TARGET = hello.exe
SOURCE = hello.c

.PHONY: all clean info

all: $(TARGET)

$(TARGET): $(SOURCE)
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS) $(LIBS)
	@echo "Built: $(TARGET)"
	@ls -lh $(TARGET)

info: $(TARGET)
	@echo "=== File Info ==="
	@file $(TARGET)
	@echo ""
	@echo "=== PE Headers ==="
	@objdump -f $(TARGET)
	@echo ""
	@echo "=== Imports ==="
	@objdump -p $(TARGET) | grep -A 5 "DLL Name"

clean:
	rm -f $(TARGET)

MinGW Headers
MinGW-w64 includes all Windows headers you need:

windows.h - Main header
winbase.h - Base definitions
winnt.h - NT definitions
Individual headers for specific APIs
You don't need to copy headers from Windows!

Using Rust for Test Fixtures
Yes, Rust can also work! Here's how:

Setup: fixtures/rust-fixtures/Cargo.toml
[
workspace
]
members = ["rust-hello", "rust-complex"]

[
workspace.package
]
version = "0.1.0"
edition = "2021"

[
profile.release
]
opt-level = "z"  # Optimize for size
lto = true
codegen-units = 1
panic = "abort"
strip = true

Example: fixtures/rust-fixtures/rust-hello/Cargo.toml
[
package
]
name = "rust-hello"
version.workspace = true
edition.workspace = true

[
dependencies
]
windows = { version = "0.52", features = ["Win32_Foundation", "Win32_System_Console", "Win32_System_Threading"] }

[[
bin
]]
name = "rust-hello"
path = "src/main.rs"

[
profile.release
]
opt-level = "z"
lto = true
panic = "abort"
strip = true

Example: fixtures/rust-fixtures/rust-hello/src/main.rs
use windows::Win32::Foundation::*;
use windows::Win32::System::Console::*;
use windows::Win32::System::Threading::*;

fn main() {
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE).unwrap();
        let message = b"Hello from Rust!\n";
        let mut written = 0;
        
        WriteFile(
            handle,
            message.as_ptr() as *const _,
            message.len() as u32,
            Some(&mut written),
            None,
        );
        
        ExitProcess(0);
    }
}

Build Rust for Windows Target
# Add Windows target
rustup target add x86_64-pc-windows-gnu

# Build
cd fixtures/rust-fixtures/rust-hello
cargo build --release --target x86_64-pc-windows-gnu

# Binary at: target/x86_64-pc-windows-gnu/release/rust-hello.exe

Master Fixtures Makefile
fixtures/Makefile
.PHONY: all clean c-fixtures rust-fixtures info

# Subdirectories with C fixtures
C_FIXTURES = hello-world target-zero heap-test file-io-test multi-thread-test registry-test

all: c-fixtures rust-fixtures

c-fixtures:
	@echo "=== Building C Fixtures ==="
	@for dir in $(C_FIXTURES); do \
		echo "Building $$dir..."; \
		$(MAKE) -C $$dir || exit 1; \
	done

rust-fixtures:
	@echo "=== Building Rust Fixtures ==="
	@cd rust-fixtures && cargo build --release --target x86_64-pc-windows-gnu

clean:
	@echo "=== Cleaning C Fixtures ==="
	@for dir in $(C_FIXTURES); do \
		$(MAKE) -C $$dir clean; \
	done
	@echo "=== Cleaning Rust Fixtures ==="
	@cd rust-fixtures && cargo clean

info:
	@echo "=== Fixture Information ==="
	@for dir in $(C_FIXTURES); do \
		if [ -f $$dir/*.exe ]; then \
			echo ""; \
			echo "$$dir:"; \
			file $$dir/*.exe; \
			ls -lh $$dir/*.exe; \
		fi; \
	done

# Copy all binaries to a single location for testing
collect:
	@mkdir -p bin
	@for dir in $(C_FIXTURES); do \
		if [ -f $$dir/*.exe ]; then \
			cp $$dir/*.exe bin/; \
		fi; \
	done
	@if [ -d rust-fixtures/target/x86_64-pc-windows-gnu/release ]; then \
		find rust-fixtures/target/x86_64-pc-windows-gnu/release -name "*.exe" -type f -exec cp {} bin/ \;; \
	fi
	@echo "All fixtures collected in bin/"

Integration with Project Build System
Update justfile
# Build all test fixtures
build-fixtures:
    @echo "Building test fixtures..."
    cd fixtures && make all

# Build C fixtures only
build-c-fixtures:
    cd fixtures && make c-fixtures

# Build Rust fixtures only
build-rust-fixtures:
    cd fixtures && make rust-fixtures

# Clean fixtures
clean-fixtures:
    cd fixtures && make clean

# Collect all fixture binaries
collect-fixtures:
    cd fixtures && make collect

# Build everything including fixtures
build-all: build build-fixtures
    @echo "All builds complete!"

# Run PE loader with specific fixture
run-fixture FIXTURE:
    cargo run -p pe-loader -- fixtures/bin/{{FIXTURE}}.exe

# Test all fixtures
test-all-fixtures: build-fixtures collect-fixtures
    @echo "Testing all fixtures..."
    @for fixture in fixtures/bin/*.exe; do \
        echo "Testing $$fixture..."; \
        cargo run -p pe-loader -- "$$fixture" || echo "FAILED: $$fixture"; \
    done

Recommended Test Fixtures by Phase
Phase 1-4: Basic Fixtures
// fixtures/hello-world/hello.c - Absolute minimum
int main(void) {
    ExitProcess(0);
    return 0;
}

// fixtures/target-zero/target-zero.c - Basic I/O (existing)
// GetStdHandle → WriteFile → ExitProcess

// fixtures/error-test/error.c - Error handling
GetLastError();
SetLastError(42);
GetLastError(); // Should be 42

Phase 5: Heap and File I/O
// fixtures/heap-test/heap.c
HANDLE heap = GetProcessHeap();
void* ptr = HeapAlloc(heap, 0, 1024);
// Use memory
HeapFree(heap, 0, ptr);

// fixtures/file-io-test/file-io.c
HANDLE file = CreateFileW(L"test.txt", GENERIC_WRITE, ...);
WriteFile(file, data, size, &written, NULL);
CloseHandle(file);

Phase 6: Multi-threading
// fixtures/multi-thread-test/threads.c
DWORD WINAPI ThreadProc(LPVOID param) {
    // Thread work
    return 0;
}

int main(void) {
    HANDLE thread = CreateThread(NULL, 0, ThreadProc, NULL, 0, NULL);
    WaitForSingleObject(thread, INFINITE);
    CloseHandle(thread);
}

When to Use Rust vs C Fixtures
Use C fixtures when:
✅ Testing compatibility with typical C-compiled Windows apps
✅ Want minimal, predictable binaries
✅ Need to match real-world app behavior
✅ Testing specific API call sequences
Use Rust fixtures when:
✅ Want type safety in test code
✅ Testing complex scenarios (easier in Rust)
✅ Need both Windows and native Linux versions
✅ Want to reuse Rust code/crates
Recommendation:
Mix both! Most fixtures in C, some in Rust for variety.

Verification Without Windows
You can verify your Windows binaries work correctly by:

Run in Wine (on Linux):

wine fixtures/hello-world/hello.exe

Inspect with Windows tools on Linux:

objdump -p hello.exe          # View PE headers
x86_64-w64-mingw32-objdump -d hello.exe  # Disassemble
strings hello.exe              # View strings

Use PE analysis tools:

# Install
sudo apt-get install pev

# Analyze
readpe hello.exe
pedis hello.exe

fixtures/README.md Template
# Test Fixtures

This directory contains test binaries for validating the Windows NT Unikernel.

## Structure

- `hello-world/` - Minimal test (just ExitProcess)
- `target-zero/` - Basic I/O test
- `heap-test/` - Heap allocation
- `file-io-test/` - File operations
- `multi-thread-test/` - Threading
- `rust-fixtures/` - Rust-based tests
- `external/` - Third-party binaries

## Building

### All Fixtures
```bash
make all

Individual Fixture
cd hello-world && make

From Project Root
just build-fixtures

Running
With PE Loader (Phase 1)
cargo run -p pe-loader -- fixtures/bin/hello.exe

With Kernel (Phase 2+)
Fixtures are embedded via include_bytes! in kernel code.

Adding New Fixtures
Create directory: fixtures/my-test/
Add source: my-test.c
Copy Makefile from hello-world
Update fixtures/Makefile to include new directory
Build: make
Cross-Compilation
All fixtures are built using MinGW-w64:

x86_64-w64-mingw32-gcc -o test.exe test.c -lkernel32

No Windows environment required!


---

## Summary & Recommendations

### ✅ Recommended Approach

1. **Use MinGW-w64 on Linux** for all C test fixtures
2. **Use Rust with windows-gnu target** for Rust fixtures
3. **Structure as shown above** (fixtures/ directory)
4. **Integrate with justfile** for easy building
5. **Mix C and Rust fixtures** for comprehensive testing

### 🚫 Don't Do This

- ❌ Don't require Windows for development
- ❌ Don't manually copy Windows headers
- ❌ Don't commit large binaries to git (add to .gitignore, build on demand)
