//! PE binary loading and parsing

use anyhow::{Result, Context};
use goblin::pe::PE;
use std::fs;

/// Load a PE binary from disk
pub fn load_binary(path: &str) -> Result<Vec<u8>> {
    fs::read(path).context("Failed to read PE binary")
}

/// Parse PE headers
pub fn parse_pe(data: &[u8]) -> Result<PE> {
    PE::parse(data).context("Failed to parse PE binary")
}
