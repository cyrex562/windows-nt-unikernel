# VSCode Configuration for Windows NT Unikernel

This directory contains VSCode configuration for working with both Rust and MinGW cross-compilation.

## Quick Setup

### 1. Install Required Extensions

When you open the project in VSCode, you'll be prompted to install recommended extensions. Accept the prompt, or manually install:

- **C/C++** (ms-vscode.cpptools) - For C IntelliSense
- **rust-analyzer** (rust-lang.rust-analyzer) - For Rust IntelliSense
- **Just** (skellock.just) - For justfile syntax highlighting

### 2. Install MinGW-w64

If you haven't already:

```bash
# From project root
just install-mingw

# Or manually
sudo apt-get install mingw-w64
```

### 3. Verify Configuration

Run the helper script to verify paths:

```bash
chmod +x .vscode/find-mingw-paths.sh
.vscode/find-mingw-paths.sh
```

This will:
- Find your MinGW installation
- Show include paths
- Test that headers work
- Show what to add to `c_cpp_properties.json` (if needed)

### 4. Reload VSCode

After installing MinGW, reload VSCode:
- Press `Ctrl+Shift+P`
- Type "Reload Window"
- Press Enter

## Configuration Files

### c_cpp_properties.json

Configures IntelliSense for C/C++ files. Tells VSCode:
- Where to find `windows.h` and other MinGW headers
- Which compiler to use (`x86_64-w64-mingw32-gcc`)
- Which defines to set (Windows-specific)
- IntelliSense mode (windows-gcc-x64)

**If IntelliSense doesn't work:**
1. Run `.vscode/find-mingw-paths.sh`
2. Update `includePath` with the paths it shows
3. Reload VSCode

### settings.json

Project-wide VSCode settings:
- Enables C/C++ error squiggles
- Configures Rust analyzer
- Sets up file associations
- Excludes `target/` from search

### tasks.json

Build tasks accessible via `Ctrl+Shift+B`:
- **Build Rust Workspace** (default) - `cargo build --workspace`
- **Build Target Zero** - Build the test binary
- **Build All Fixtures** - Build all test binaries
- **Build Everything** - Rust + all fixtures
- **Run PE Loader** - Run Phase 1 loader
- **Clean All** - Clean build artifacts

### extensions.json

Recommended extensions that make development easier.

## Common Issues

### "Cannot find windows.h"

**Symptoms:**
- Red squiggles under `#include <windows.h>`
- IntelliSense errors

**Solutions:**

1. **Install MinGW:**
   ```bash
   sudo apt-get install mingw-w64
   ```

2. **Find the correct path:**
   ```bash
   .vscode/find-mingw-paths.sh
   ```

3. **Update c_cpp_properties.json:**
   Add the paths shown by the script to `includePath`.

4. **Reload VSCode:**
   `Ctrl+Shift+P` → "Reload Window"

### IntelliSense shows wrong errors

**Symptoms:**
- Code compiles fine but VSCode shows errors
- Functions like `GetStdHandle` show as undefined

**Solutions:**

1. **Check IntelliSense mode:**
   In `c_cpp_properties.json`, ensure `intelliSenseMode` is `"windows-gcc-x64"`

2. **Check defines:**
   Make sure these are set in `c_cpp_properties.json`:
   ```json
   "defines": [
       "WIN32",
       "_WIN32",
       "_WIN64",
       "UNICODE",
       "_UNICODE"
   ]
   ```

3. **Reset IntelliSense:**
   `Ctrl+Shift+P` → "C/C++: Reset IntelliSense Database"

### Rust-analyzer is slow

**Symptoms:**
- VSCode laggy when editing Rust files
- CPU usage high

**Solutions:**

1. **Disable some features in settings.json:**
   ```json
   "rust-analyzer.checkOnSave.allTargets": false
   ```

2. **Exclude target/ from file watcher:**
   Already configured in `settings.json`

3. **Use `cargo check` instead of `cargo clippy`:**
   ```json
   "rust-analyzer.check.command": "check"
   ```

### WSL-specific issues

**Symptoms:**
- VSCode doesn't recognize files
- Git integration broken

**Solutions:**

1. **Make sure you're using WSL extension:**
   - Install "WSL" extension (ms-vscode-remote.remote-wsl)
   - Open project via `code .` from WSL terminal

2. **Don't open project from Windows:**
   - Always open from within WSL
   - Path should be `/home/user/...` not `\\wsl$\...`

## Testing the Setup

### Test C IntelliSense

1. Open `target-zero/target-zero.c`
2. Hover over `GetStdHandle` - should show documentation
3. `Ctrl+Click` on `HANDLE` - should jump to definition
4. Type `Write` - should autocomplete to `WriteFile`

### Test Rust IntelliSense

1. Open any `.rs` file
2. Hover over a function - should show documentation
3. `Ctrl+Click` on a type - should jump to definition
4. Errors should appear in real-time

### Test Build Tasks

1. Press `Ctrl+Shift+B`
2. Select "Build Target Zero"
3. Should compile successfully
4. Terminal shows output

## File Type Support

The configuration recognizes these file types:

- **C/C++**: `.c`, `.h`, `.cpp`, `.hpp`
- **Rust**: `.rs`
- **Windows**: `windows.h`, `winbase.h`, etc.
- **Build**: `Makefile`, `justfile`, `Cargo.toml`

## Keyboard Shortcuts

With this configuration, you can use:

- `Ctrl+Shift+B` - Build (shows task menu)
- `F5` - Debug (if launch.json configured)
- `Ctrl+K Ctrl+I` - Show hover information
- `F12` - Go to definition
- `Ctrl+Shift+O` - Go to symbol in file
- `Ctrl+T` - Go to symbol in workspace
- `Ctrl+P` - Quick file open
- `Ctrl+Shift+P` - Command palette

## Updating Configuration

If you add new test fixtures or change the project structure:

1. **For new C files**: No action needed, IntelliSense uses `**` wildcards
2. **For new include paths**: Update `includePath` in `c_cpp_properties.json`
3. **For new tasks**: Add to `tasks.json`

## Troubleshooting Checklist

- [ ] MinGW installed? (`which x86_64-w64-mingw32-gcc`)
- [ ] C/C++ extension installed?
- [ ] Rust-analyzer extension installed?
- [ ] Reloaded VSCode after installing MinGW?
- [ ] Run `.vscode/find-mingw-paths.sh` to verify paths?
- [ ] Updated `c_cpp_properties.json` if paths differ?
- [ ] Opened project from WSL (not Windows)?

## Additional Resources

- [C/C++ Extension Docs](https://code.visualstudio.com/docs/cpp/config-linux)
- [Rust-Analyzer User Manual](https://rust-analyzer.github.io/manual.html)
- [VSCode WSL Docs](https://code.visualstudio.com/docs/remote/wsl)

---

**Need help?** Run `.vscode/find-mingw-paths.sh` and check the output.
