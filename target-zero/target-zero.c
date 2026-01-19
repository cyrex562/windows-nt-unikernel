/**
 * Target Zero - Minimal Windows Test Binary
 *
 * This is a minimal Windows executable that tests basic kernel32.dll functions:
 * - GetStdHandle: Get handle to stdout
 * - WriteFile: Write text to stdout
 * - ExitProcess: Terminate the process
 *
 * Compile with MinGW or MSVC to create a Windows PE executable.
 */

#include <windows.h>

// Message to print
const char MESSAGE[] = "Hello from Target Zero!\r\n";

/**
 * Main entry point
 *
 * Note: We use the standard main() instead of WinMain to keep it simple.
 */
int main(void) {
    HANDLE hStdOut;
    DWORD bytesWritten;
    BOOL success;

    // Get handle to standard output
    hStdOut = GetStdHandle(STD_OUTPUT_HANDLE);
    if (hStdOut == INVALID_HANDLE_VALUE) {
        // Can't write error message if we can't get stdout handle
        ExitProcess(1);
    }

    // Write message to stdout
    success = WriteFile(
        hStdOut,                          // Handle to stdout
        MESSAGE,                          // Buffer to write
        sizeof(MESSAGE) - 1,              // Bytes to write (exclude null terminator)
        &bytesWritten,                    // Bytes actually written
        NULL                              // No overlapped I/O
    );

    if (!success) {
        ExitProcess(2);
    }

    // Exit successfully
    ExitProcess(0);

    // Never reached, but prevents compiler warnings
    return 0;
}
