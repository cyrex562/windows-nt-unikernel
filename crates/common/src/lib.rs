//! Common types and structures shared across the Windows NT Unikernel project
//!
//! This crate contains Windows API types, structures, and constants that are
//! used by both the PE loader and the kernel/API shim.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod pe;
pub mod windows;

/// Windows HANDLE type
pub type HANDLE = usize;

/// Windows DWORD type (32-bit unsigned)
pub type DWORD = u32;

/// Windows BOOL type
pub type BOOL = i32;

/// Windows error codes
pub const ERROR_SUCCESS: DWORD = 0;
pub const ERROR_INVALID_HANDLE: DWORD = 6;
pub const ERROR_NOT_ENOUGH_MEMORY: DWORD = 8;
pub const ERROR_INVALID_PARAMETER: DWORD = 87;

/// Standard handles
pub const STD_INPUT_HANDLE: DWORD = 0xFFFFFFF6_u32;
pub const STD_OUTPUT_HANDLE: DWORD = 0xFFFFFFF5_u32;
pub const STD_ERROR_HANDLE: DWORD = 0xFFFFFFF4_u32;

/// Invalid handle value
pub const INVALID_HANDLE_VALUE: HANDLE = !0;
