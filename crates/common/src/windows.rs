//! Windows API structures and types
//!
//! This module contains definitions for Windows internal structures like
//! TEB, PEB, and related types.

use crate::{HANDLE, DWORD};

/// Thread Environment Block (TEB) - Simplified version
///
/// This is a highly simplified version of the Windows TEB structure.
/// In Windows x64, GS:[0x30] points to the TEB.
#[repr(C)]
pub struct TEB {
    /// Pointer to the Process Environment Block
    pub peb: *mut PEB,
    /// Last error code for this thread
    pub last_error: DWORD,
    /// Thread ID
    pub thread_id: DWORD,
    /// Process ID
    pub process_id: DWORD,
}

/// Process Environment Block (PEB) - Simplified version
///
/// This structure contains process-wide information.
#[repr(C)]
pub struct PEB {
    /// Base address of the loaded executable
    pub image_base_address: *mut u8,
    /// Process heap handle
    pub process_heap: HANDLE,
    /// Pointer to process parameters
    pub process_parameters: *mut RTL_USER_PROCESS_PARAMETERS,
}

/// Process Parameters structure
#[repr(C)]
pub struct RTL_USER_PROCESS_PARAMETERS {
    /// Length of this structure
    pub length: u32,
    /// Maximum length
    pub maximum_length: u32,
    /// Reserved
    pub flags: u32,
    /// Debug flags
    pub debug_flags: u32,
    /// Console handle
    pub console_handle: HANDLE,
    /// Console flags
    pub console_flags: u32,
    /// Standard input handle
    pub standard_input: HANDLE,
    /// Standard output handle
    pub standard_output: HANDLE,
    /// Standard error handle
    pub standard_error: HANDLE,
}

impl TEB {
    /// Create a new TEB with default values
    pub fn new() -> Self {
        Self {
            peb: core::ptr::null_mut(),
            last_error: 0,
            thread_id: 1,
            process_id: 1,
        }
    }
}

impl PEB {
    /// Create a new PEB with default values
    pub fn new(image_base: *mut u8) -> Self {
        Self {
            image_base_address: image_base,
            process_heap: 1, // Fake heap handle
            process_parameters: core::ptr::null_mut(),
        }
    }
}

impl Default for TEB {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PEB {
    fn default() -> Self {
        Self::new(core::ptr::null_mut())
    }
}
