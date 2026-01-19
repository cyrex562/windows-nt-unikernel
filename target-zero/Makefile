# Makefile for Target Zero
# Cross-compiles a minimal Windows PE executable on Linux

CC = x86_64-w64-mingw32-gcc
CFLAGS = -static -Os -s
LDFLAGS = -Wl,--subsystem,console
LIBS = -lkernel32

TARGET = target-zero.exe
SOURCE = target-zero.c

.PHONY: all clean check

all: $(TARGET)

$(TARGET): $(SOURCE)
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS) $(LIBS)
	@echo "Built: $(TARGET)"
	@ls -lh $(TARGET)

# Check if binary was created and show info
check: $(TARGET)
	@echo "=== Binary Information ==="
	@file $(TARGET)
	@echo ""
	@echo "=== Imports ==="
	@objdump -p $(TARGET) | grep -A 20 "DLL Name"
	@echo ""
	@echo "=== Entry Point ==="
	@objdump -f $(TARGET) | grep "start address"

clean:
	rm -f $(TARGET)

# Install mingw-w64 if not present
install-deps:
	@echo "Installing MinGW-w64 cross-compiler..."
	@if [ -f /etc/debian_version ]; then \
		sudo apt-get update && sudo apt-get install -y mingw-w64; \
	elif [ -f /etc/redhat-release ]; then \
		sudo dnf install -y mingw64-gcc; \
	elif [ -f /etc/arch-release ]; then \
		sudo pacman -S --noconfirm mingw-w64-gcc; \
	else \
		echo "Unsupported distribution. Please install mingw-w64 manually."; \
	fi
