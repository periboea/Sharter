//   MEMORY {
//     FLASH (rx)  : ORIGIN = 0x08000000, LENGTH = 512K
//     RAM   (rwx) : ORIGIN = 0x20000000, LENGTH = 128K
//   }
//
//   SECTIONS {
//     .text   : { *(.text*)   } > FLASH
//     .data   : { *(.data*)   } > RAM AT > FLASH
//     .bss    : { *(.bss*)    } > RAM
//   }

use anyhow::Result;
use std::path::Path;
use std::option::Option;

use crate::model::{MemoryMap, MemoryRegion, MemorySection, SectionKind};

pub fn parse_linker(path: &Path, src: &str) -> Result<MemoryMap> {
    let mut map = MemoryMap {
        source: path.display().to_string(),
        ..Default::default()
    };

    map.regions  = parse_memory_block(src);
    map.sections = parse_sections_block(src, &map.regions);

    Ok(map)
}

fn parse_memory_block(src: &str) -> Vec<MemoryRegion> {
    let mut regions = Vec::new();
    let block = match extract_block(src, "MEMORY") {
        Some(b) => b,
        None    => return regions,
    };

    for line in block.lines() {
        let line = strip_comments(line).trim().to_string();
        if line.is_empty() { continue; }

        // NAME (attrs) : ORIGIN = 0x..., LENGTH = ...
        // NAME         : ORIGIN = 0x..., LENGTH = ...
        let (name, rest) = match line.split_once(':') {
            Some(p) => p,
            None    => continue,
        };

        let raw_name = name.trim();
        let (name, attributes) = if let Some(p) = raw_name.split_once('(') {
            let a = p.1.trim_end_matches(')').trim().to_string();
            (p.0.trim().to_string(), Some(a))
        } else {
            (raw_name.to_string(), None)
        };

        let origin = parse_kv(rest, "ORIGIN").or_else(|| parse_kv(rest, "org"))
            .unwrap_or(0);
        let length = parse_length_kv(rest).unwrap_or(0);

        if !name.is_empty() {
            regions.push(MemoryRegion { name, origin, length, attributes });
        }
    }
    regions
}

fn parse_sections_block(src: &str, regions: &[MemoryRegion]) -> Vec<MemorySection> {
    let mut sections = Vec::new();
    let block = match extract_block(src, "SECTIONS") {
        Some(b) => b,
        None    => return sections,
    };

    let mut cursor: std::collections::HashMap<String, u64> = regions
        .iter()
        .map(|r| (r.name.clone(), r.origin))
        .collect();

    for line in block.lines() {
        let line = strip_comments(line).trim().to_string();
        if line.is_empty() { continue; }

        // match lines like:  .text : { ... } > FLASH
        //                    .data : { ... } > RAM AT > FLASH
        if !line.starts_with('.') { continue; }

        let sec_name = line
            .split(|c: char| c.is_whitespace() || c == ':')
            .next()
            .unwrap_or("")
            .trim()
            .to_string();
        if sec_name.is_empty() { continue; }

        // target region is after '>'
        let target_region = line
            .split('>')
            .nth(1)
            .map(|s| s.split_whitespace().next().unwrap_or("").to_string());

        if let Some(region_name) = target_region {
            if let Some(region) = regions.iter().find(|r| r.name == region_name) {
                let address = *cursor.get(&region_name).unwrap_or(&region.origin);
                // placeholder: 4 KB per unknown section so the diagram isn't empty.
                let size: u64 = 4 * 1024;
                cursor.insert(region_name.clone(), address + size);

                sections.push(MemorySection {
                    kind: SectionKind::from_name(&sec_name),
                    name: sec_name,
                    address,
                    size,
                });
            }
        }
    }
    sections
}

// extract the brace-delimited body after 'keyword'.
fn extract_block<'a>(src: &'a str, keyword: &str) -> Option<&'a str> {
    let idx = src.find(keyword)?;
    let after = &src[idx + keyword.len()..];
    let open  = after.find('{')?;
    let body  = &after[open + 1..];

    // find matching closing brace (depth aware)
    let mut depth = 1usize;
    for (i, ch) in body.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&body[..i]);
                }
            }
            _ => {}
        }
    }
    None
}

fn strip_comments(s: &str) -> &str {
    // remove /* ... */ inline comments (single-line only for brevity.)
    if let Some(i) = s.find("/*") {
        &s[..i]
    } else if let Some(i) = s.find("//") {
        &s[..i]
    } else {
        s
    }
}

// parse 'KEY = VALUE' from a string, returning the numeric value.
fn parse_kv(s: &str, key: &str) -> Option<u64> {
    let idx = s.to_uppercase().find(&key.to_uppercase())?;
    let after = s[idx + key.len()..].trim_start();
    let after = after.strip_prefix('=')?.trim();
    parse_number(after.split(|c: char| c == ',' || c.is_whitespace()).next()?)
}

fn parse_length_kv(s: &str) -> Option<u64> {
    let key = if s.to_uppercase().contains("LENGTH") { "LENGTH" } else { "len" };
    let idx = s.to_uppercase().find(&key.to_uppercase())?;
    let after = s[idx + key.len()..].trim_start();
    let after = after.strip_prefix('=')?.trim();
    let token = after.split(|c: char| c == ',' || c.is_whitespace()).next()?;
    parse_size(token)
}

fn parse_number(s: &str) -> Option<u64> {
    let s = s.trim();
    if let Some(h) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u64::from_str_radix(h, 16).ok()
    } else {
        s.parse().ok()
    }
}

fn parse_size(s: &str) -> Option<u64> {
    let s = s.trim();
    // handles K / M / G suffixes
    if let Some(n) = s.strip_suffix('K').or_else(|| s.strip_suffix('k')) {
        return parse_number(n).map(|v| v * 1024);
    }
    if let Some(n) = s.strip_suffix('M').or_else(|| s.strip_suffix('m')) {
        return parse_number(n).map(|v| v * 1024 * 1024);
    }
    if let Some(n) = s.strip_suffix('G').or_else(|| s.strip_suffix('g')) {
        return parse_number(n).map(|v| v * 1024 * 1024 * 1024);
    }
    parse_number(s)
}