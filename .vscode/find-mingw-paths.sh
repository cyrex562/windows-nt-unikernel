#!/bin/bash
# Helper script to find MinGW include paths for VSCode configuration

echo "=== Finding MinGW-w64 Include Paths ==="
echo ""

# Find windows.h
echo "Searching for windows.h..."
WINDOWS_H=$(find /usr -name "windows.h" -type f 2>/dev/null | grep mingw | head -1)

if [ -z "$WINDOWS_H" ]; then
    echo "❌ windows.h not found!"
    echo "Please install MinGW-w64:"
    echo "  sudo apt-get install mingw-w64"
    exit 1
fi

echo "✅ Found: $WINDOWS_H"

# Extract base include directory
INCLUDE_DIR=$(dirname "$WINDOWS_H")
echo ""
echo "=== Include Directories ==="
echo "$INCLUDE_DIR"

# Find compiler
COMPILER=$(which x86_64-w64-mingw32-gcc)
if [ -z "$COMPILER" ]; then
    echo ""
    echo "❌ MinGW compiler not found!"
    exit 1
fi

echo ""
echo "=== Compiler ==="
echo "$COMPILER"

# Get compiler include search paths
echo ""
echo "=== Compiler Include Search Paths ==="
echo "" | x86_64-w64-mingw32-gcc -xc -E -v - 2>&1 | sed -n '/^#include <...>/,/^End of search/p' | grep '^ /'

echo ""
echo "=== VSCode Configuration ==="
echo "Add these paths to .vscode/c_cpp_properties.json in the 'includePath' array:"
echo "" | x86_64-w64-mingw32-gcc -xc -E -v - 2>&1 | sed -n '/^#include <...>/,/^End of search/p' | grep '^ /' | sed 's/^/  "/' | sed 's/$/\/**",/'

echo ""
echo "=== Testing ==="
echo "Creating test file..."

cat > /tmp/test_mingw.c <<EOF
#include <windows.h>

int main(void) {
    HANDLE h = GetStdHandle(STD_OUTPUT_HANDLE);
    return 0;
}
EOF

if x86_64-w64-mingw32-gcc -c /tmp/test_mingw.c -o /tmp/test_mingw.o 2>/dev/null; then
    echo "✅ MinGW headers working correctly!"
    rm /tmp/test_mingw.o
else
    echo "❌ MinGW headers test failed"
fi

rm /tmp/test_mingw.c
