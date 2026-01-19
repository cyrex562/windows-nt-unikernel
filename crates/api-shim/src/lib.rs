//! Windows API Shim Implementation
//!
//! This crate provides implementations of Windows API functions (kernel32.dll, ntdll.dll, etc.)
//! that work in both userspace (for Phase 1 testing) and bare-metal (Phase 4+).

#![cfg_attr(feature = "no_std", no_std)]

use common::{HANDLE, DWORD, BOOL};

pub mod kernel32;

/// Get the last error code for the current thread
///
/// In a real implementation, this would be stored in thread-local storage (TLS).
/// For now, we use a simple static variable.
static mut LAST_ERROR: DWORD = 0;

/// Set the last error code
pub fn set_last_error(error: DWORD) {
    unsafe {
        LAST_ERROR = error;
    }
}

/// Get the last error code
pub fn get_last_error() -> DWORD {
    unsafe { LAST_ERROR }
}
