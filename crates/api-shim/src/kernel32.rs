//! kernel32.dll API implementations

use common::{HANDLE, DWORD, BOOL, STD_OUTPUT_HANDLE, STD_INPUT_HANDLE, STD_ERROR_HANDLE};
use crate::{set_last_error, get_last_error};

/// GetStdHandle implementation
///
/// Returns a handle to the specified standard device (stdin, stdout, or stderr).
#[no_mangle]
pub extern "C" fn GetStdHandle(std_handle: DWORD) -> HANDLE {
    match std_handle {
        STD_INPUT_HANDLE => 0x10,  // Fake stdin handle
        STD_OUTPUT_HANDLE => 0x11, // Fake stdout handle
        STD_ERROR_HANDLE => 0x12,  // Fake stderr handle
        _ => {
            set_last_error(common::ERROR_INVALID_PARAMETER);
            common::INVALID_HANDLE_VALUE
        }
    }
}

/// WriteFile implementation
///
/// Writes data to a file or device. For stdout/stderr, we write to the console.
#[no_mangle]
pub extern "C" fn WriteFile(
    handle: HANDLE,
    buffer: *const u8,
    bytes_to_write: DWORD,
    bytes_written: *mut DWORD,
    overlapped: *mut u8, // OVERLAPPED pointer, unused
) -> BOOL {
    // Validate parameters
    if buffer.is_null() {
        set_last_error(common::ERROR_INVALID_PARAMETER);
        return 0; // FALSE
    }

    // For now, we only handle stdout and stderr
    if handle != 0x11 && handle != 0x12 {
        set_last_error(common::ERROR_INVALID_HANDLE);
        return 0; // FALSE
    }

    // Create a slice from the buffer
    let data = unsafe {
        core::slice::from_raw_parts(buffer, bytes_to_write as usize)
    };

    // Write to output
    #[cfg(feature = "std")]
    {
        use std::io::Write;
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        if handle.write_all(data).is_ok() {
            if !bytes_written.is_null() {
                unsafe { *bytes_written = bytes_to_write; }
            }
            set_last_error(common::ERROR_SUCCESS);
            return 1; // TRUE
        } else {
            set_last_error(common::ERROR_INVALID_PARAMETER);
            return 0; // FALSE
        }
    }

    #[cfg(not(feature = "std"))]
    {
        // In bare-metal mode, write to serial port or VGA buffer
        // TODO: Implement serial/VGA output
        if !bytes_written.is_null() {
            unsafe { *bytes_written = bytes_to_write; }
        }
        set_last_error(common::ERROR_SUCCESS);
        1 // TRUE
    }
}

/// ExitProcess implementation
///
/// Terminates the current process.
#[no_mangle]
pub extern "C" fn ExitProcess(exit_code: u32) -> ! {
    #[cfg(feature = "std")]
    {
        std::process::exit(exit_code as i32);
    }

    #[cfg(not(feature = "std"))]
    {
        // In bare-metal mode, halt the CPU or trigger QEMU shutdown
        // TODO: Implement proper shutdown
        loop {
            #[cfg(target_arch = "x86_64")]
            unsafe {
                core::arch::asm!("hlt");
            }
        }
    }
}

/// GetLastError implementation
///
/// Returns the last error code for the current thread.
#[no_mangle]
pub extern "C" fn GetLastError() -> DWORD {
    get_last_error()
}

/// SetLastError implementation
///
/// Sets the last error code for the current thread.
#[no_mangle]
pub extern "C" fn SetLastError(error: DWORD) {
    set_last_error(error);
}
