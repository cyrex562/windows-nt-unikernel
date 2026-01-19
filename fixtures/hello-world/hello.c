#include <windows.h>

const char MESSAGE[] = "Hello, World!\r\n";

int main(void) {
    HANDLE hStdOut;
    DWORD bytesWritten;
    BOOL success;

    hStdOut = GetStdHandle(STD_OUTPUT_HANDLE);
    if (hStdOut == INVALID_HANDLE_VALUE) {
        ExitProcess(1);
    }

    success = WriteFile(
        hStdOut,
        MESSAGE,
        sizeof(MESSAGE) - 1,
        &bytesWritten,
        NULL
    );

    if (!success) {
        ExitProcess(2);
    }

    ExitProcess(0);
    return 0;
}