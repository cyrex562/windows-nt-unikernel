# Target Zero - Minimal Windows Test Binary

This is a minimal Windows executable used to test the Windows API shim in the unikernel.

## What it does

Target Zero performs three simple operations:

1. **GetStdHandle(STD_OUTPUT_HANDLE)** - Gets a handle to standard output
2. **WriteFile()** - Writes "Hello from Target Zero!" to stdout
3. **ExitProcess(0)** - Exits the process with code 0

## Building on Windows

### Using MinGW-w64

```bash
x86_64-w64-mingw32-gcc -o target-zero.exe target-zero.c -lkernel32 -nostdlib -Os -s
```

### Using MinGW with static linking (recommended)

```bash
x86_64-w64-mingw32-gcc -o target-zero.exe target-zero.c ^
    -lkernel32 ^
    -static ^
    -Os ^
    -s ^
    -Wl,--subsystem,console
```

### Using MSVC (Visual Studio)

```bash
cl /O1 /MT /Fe:target-zero.exe target-zero.c /link /SUBSYSTEM:CONSOLE kernel32.lib
```

## Building on Linux (Cross-compilation)

### Install MinGW-w64

**Debian/Ubuntu:**
```bash
sudo apt-get install mingw-w64
```

**Fedora/RHEL:**
```bash
sudo dnf install mingw64-gcc
```

**Arch Linux:**
```bash
sudo pacman -S mingw-w64-gcc
```

### Compile

```bash
x86_64-w64-mingw32-gcc -o target-zero.exe target-zero.c \
    -lkernel32 \
    -static \
    -Os \
    -s \
    -Wl,--subsystem,console
```

## Compiler Flags Explained

- `-o target-zero.exe` - Output filename
- `-lkernel32` - Link against kernel32.dll
- `-static` - Statically link the C runtime (minimize dependencies)
- `-Os` - Optimize for size
- `-s` - Strip symbols
- `-Wl,--subsystem,console` - Create a console application

## Verifying the Binary

### On Windows

Simply run the executable:
```bash
target-zero.exe
```

Expected output:
```
Hello from Target Zero!
```

### On Linux (using Wine)

```bash
wine target-zero.exe
```

### Inspecting with PE tools

**On Linux:**
```bash
objdump -x target-zero.exe | less
```

**On Windows:**
```bash
dumpbin /headers target-zero.exe
```

## Expected Imports

The binary should import only from `kernel32.dll`:
- `GetStdHandle`
- `WriteFile`
- `ExitProcess`

You can verify this with:

**Linux:**
```bash
objdump -p target-zero.exe | grep "DLL Name"
```

**Windows:**
```bash
dumpbin /imports target-zero.exe
```

## File Size

The compiled binary should be very small:
- **With MinGW (static)**: ~8-15 KB
- **With MSVC**: ~5-10 KB

## Testing in the Unikernel

Once the PE loader is implemented, this binary will be used to verify:

1. **Phase 1 (Userspace)**: The loader can parse PE headers, map sections, and resolve imports
2. **Phase 4 (Bare-metal)**: The unikernel can execute the binary and produce the correct output

## Troubleshooting

### "cannot find -lkernel32"

Make sure you have MinGW-w64 properly installed. The library should be at:
```
/usr/x86_64-w64-mingw32/lib/libkernel32.a
```

### Binary is too large (>100KB)

This usually means you're linking against the dynamic CRT. Use `-static` flag.

### "undefined reference to WinMain"

Add `-Wl,--subsystem,console` to tell the linker this is a console app with `main()`, not a GUI app with `WinMain()`.
