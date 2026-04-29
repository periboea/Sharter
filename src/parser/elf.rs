use anyhow::Result;
use goblin::elf::Elf;
use std::path::Path;
use goblin::elf::program_header::PT_LOAD;
use crate::model::{MemoryMap, MemoryRegion, MemorySection, SectionKind};

pub fn parse_elf(path: &Path, bytes: &[u8]) -> Result<MemoryMap> {
    let elf = Elf::parse(bytes)?;
    let mut map = MemoryMap {
        source: path.display().to_string(),
        ..Default::default()
    };

    // build regions from program headers.
    // each unique base region gets its own entry. i bucket by the high bits of
    // the physical address to separate FLASH like and RAM like segments.
    let mut flash_origin: Option<u64> = None;
    let mut flash_end:    u64 = 0;
    let mut ram_origin:   Option<u64> = None;
    let mut ram_end:      u64 = 0;

    for ph in &elf.program_headers {
        use goblin::elf::program_header::PT_LOAD;
        if ph.p_type != PT_LOAD || ph.p_memsz == 0 {
            continue;
        }

        let paddr = ph.p_paddr;
        let vaddr = ph.p_vaddr;
        let filesz = ph.p_filesz;
        let memsz  = ph.p_memsz;

        // heuristic. if vaddr != paddr the segment is copied from flash to ram
        // at runtime (.data).
        if paddr != vaddr {
            // flash portion
            flash_origin = Some(flash_origin.map_or(paddr, |o: u64| o.min(paddr)));
            flash_end = flash_end.max(paddr + filesz);
            // ram portion
            ram_origin = Some(ram_origin.map_or(vaddr, |o: u64| o.min(vaddr)));
            ram_end = ram_end.max(vaddr + memsz);
        } else if filesz > 0 {
            flash_origin = Some(flash_origin.map_or(paddr, |o: u64| o.min(paddr)));
            flash_end = flash_end.max(paddr + filesz);
        } else {
            // bss-only segment (filesz == 0, memsz > 0)
            ram_origin = Some(ram_origin.map_or(vaddr, |o: u64| o.min(vaddr)));
            ram_end = ram_end.max(vaddr + memsz);
        }
    }

    // holy shit at this point i wanna kill myself TODO DELETE THIS COMMENT

    if let Some(o) = flash_origin {
        map.regions.push(MemoryRegion {
            name:   "FLASH".into(),
            origin: o,
            length: flash_end.saturating_sub(o),
            attributes:  Some("rx".into()),
        });
    }
    if let Some(o) = ram_origin {
        map.regions.push(MemoryRegion {
            name:   "RAM".into(),
            origin: o,
            length: ram_end.saturating_sub(o),
            attributes:  Some("rwx".into()),
        });
    }

    // if no program headers gave regions fall back to killing myself i mean to section spans TODO DELETE THIS COMMENT DAMN
    if map.regions.is_empty(){
        infer_regions_from_sections(&mut map); //TODO write the function for this.
    }

    map.sections.sort_by_key(|s| s.address);
    Ok(map)
}

// TODO write the function to infer regions from sections. I'm going to bed.