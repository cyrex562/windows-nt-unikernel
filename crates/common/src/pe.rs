//! PE (Portable Executable) format structures and constants
//!
//! This module defines structures for parsing and working with Windows PE files.

/// PE section characteristics
#[derive(Debug, Clone, Copy)]
pub struct SectionCharacteristics(pub u32);

impl SectionCharacteristics {
    pub const IMAGE_SCN_CNT_CODE: u32 = 0x00000020;
    pub const IMAGE_SCN_CNT_INITIALIZED_DATA: u32 = 0x00000040;
    pub const IMAGE_SCN_CNT_UNINITIALIZED_DATA: u32 = 0x00000080;
    pub const IMAGE_SCN_MEM_EXECUTE: u32 = 0x20000000;
    pub const IMAGE_SCN_MEM_READ: u32 = 0x40000000;
    pub const IMAGE_SCN_MEM_WRITE: u32 = 0x80000000;

    pub fn is_executable(self) -> bool {
        self.0 & Self::IMAGE_SCN_MEM_EXECUTE != 0
    }

    pub fn is_readable(self) -> bool {
        self.0 & Self::IMAGE_SCN_MEM_READ != 0
    }

    pub fn is_writable(self) -> bool {
        self.0 & Self::IMAGE_SCN_MEM_WRITE != 0
    }
}
