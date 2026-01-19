# justfile for Windows NT Unikernel
# Run `just --list` to see all available commands

# Default recipe (shown when running `just` without arguments)
default:
    @just --list

# Build all workspace crates
build:
    cargo build --workspace

# Build in release mode
build-release:
    cargo build --workspace --release

# Build the PE loader (Phase 1)
build-loader:
    cargo build -p pe-loader

# Build the kernel (Phase 2+)
build-kernel:
    cargo build -p kernel

# Build Target Zero Windows binary
build-target-zero:
    @echo "Building Target Zero..."
    @cd target-zero && make

# Build everything (Rust workspace + Target Zero)
build-all: build build-target-zero
    @echo "All builds complete!"

# Clean all build artifacts
clean:
    cargo clean
    cd target-zero && make clean

# Run the PE loader with Target Zero binary
run-loader: build-loader build-target-zero
    cargo run -p pe-loader -- target-zero/target-zero.exe

# Run tests
test:
    cargo test --workspace

# Check code without building (fast)
check:
    cargo check --workspace

# Format code
fmt:
    cargo fmt --all

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Run clippy linter
clippy:
    cargo clippy --workspace -- -D warnings

# Build kernel binary image
build-kernel-image: build-kernel
    @echo "Building kernel image..."
    # TODO: Configure bootimage or similar tool

# Run kernel in QEMU (Phase 2+)
run-qemu: build-kernel-image
    @echo "Starting QEMU..."
    qemu-system-x86_64 \
        -drive format=raw,file=target/x86_64-unknown-none/debug/bootimage-kernel.bin \
        -serial stdio \
        -display none
    # TODO: Update path when kernel builds successfully

# Run kernel in QEMU with debugging support
debug-qemu: build-kernel-image
    @echo "Starting QEMU with GDB server..."
    qemu-system-x86_64 \
        -drive format=raw,file=target/x86_64-unknown-none/debug/bootimage-kernel.bin \
        -serial stdio \
        -display none \
        -s -S
    # Connect with: gdb -ex "target remote :1234"

# Install dependencies for building Target Zero (MinGW)
install-mingw:
    @echo "Installing MinGW-w64..."
    @if [ -f /etc/debian_version ]; then \
        sudo apt-get update && sudo apt-get install -y mingw-w64; \
    elif [ -f /etc/redhat-release ]; then \
        sudo dnf install -y mingw64-gcc; \
    elif [ -f /etc/arch-release ]; then \
        sudo pacman -S --noconfirm mingw-w64-gcc; \
    else \
        echo "Unsupported distribution. Please install mingw-w64 manually."; \
    fi

# Install QEMU (for running the kernel)
install-qemu:
    @echo "Installing QEMU..."
    @if [ -f /etc/debian_version ]; then \
        sudo apt-get update && sudo apt-get install -y qemu-system-x86; \
    elif [ -f /etc/redhat-release ]; then \
        sudo dnf install -y qemu-system-x86; \
    elif [ -f /etc/arch-release ]; then \
        sudo pacman -S --noconfirm qemu-system-x86; \
    else \
        echo "Unsupported distribution. Please install qemu-system-x86 manually."; \
    fi

# Install all development dependencies
install-deps: install-mingw install-qemu
    @echo "Installing Rust components..."
    rustup component add rust-src
    rustup component add llvm-tools-preview
    cargo install bootimage
    @echo "All dependencies installed!"

# Show project structure
tree:
    @tree -I target -L 3

# Show project statistics
stats:
    @echo "=== Lines of Code ==="
    @find crates -name "*.rs" | xargs wc -l | tail -1
    @echo ""
    @echo "=== Crate Structure ==="
    @cargo tree --depth 1
    @echo ""
    @echo "=== Build Artifacts ==="
    @du -sh target 2>/dev/null || echo "No build artifacts yet"

# Generate documentation
doc:
    cargo doc --workspace --no-deps --open

# Watch and rebuild on changes (requires cargo-watch)
watch:
    cargo watch -x "check --workspace" -x "test --workspace"

# Phase 0: Complete Phase 0 checklist
phase-0: build-all
    @echo "✓ Phase 0 Complete!"
    @echo "  - Target Zero binary created"
    @echo "  - Rust workspace initialized"
    @echo "  - Build system configured"

# Phase 1: Run Phase 1 prototype
phase-1: run-loader
    @echo "Running Phase 1: Userspace PE Loader"

# Phase 2: Boot kernel
phase-2: run-qemu
    @echo "Running Phase 2: Bare-metal Kernel"
