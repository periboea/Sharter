pub mod elf;
pub mod linker;

use anyhow::{bail, Result};
use std::path::Path;

use crate::model::MemoryMap;

pub fn parse(path: &Path) -> Result<MemoryMap> {
    let bytes = std::fs::read(path)?;

    // ELF magic: 0x7f 'E' 'L' 'F'
    if bytes.starts_with(b"\x7fELF") {
        return elf::parse_elf(path, &bytes);
    }

    // linker script: treat as UTF-8 text
    if let Ok(text) = std::str::from_utf8(&bytes) {
        if text.contains("MEMORY") || text.contains("SECTIONS") || text.contains("ORIGIN") {
            return linker::parse_linker(path, text);
        }
    }

    bail!("unrecognised file format. expected ELF binary or GNU linker script (.ld)");
}