#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub name: String,
    pub origin: u64,
    pub length: u64,
    pub attributes: Option<String>
}

impl MemoryRegion {
    // computes where the region stops in an address space.
    pub fn end_of(&self) -> u64{
        self.origin.saturating_add(self.length)
    }

    pub fn contains(&self, s: &MemorySection) -> bool {
        // the section starts at or after where this region begins. the section starts before where this region ends.
        s.address >= self.origin && s.address < self.end_of()
    }

    // how many bytes in [origin, origin + length] are covered by sections.
    pub fn used_bytes(&self, sections: &[MemorySection]) -> u64 {
        sections
            .iter()
            .filter(|s| self.contains(s))
            .map(|s| s.size)
            .sum()
    }
}
#[derive(Clone, Debug)]
pub struct MemorySection {
    name: String,
    address: u64,
    size: u64,
    kind: SectionKind
}

#[derive(Clone, Debug, PartialEq)]
pub enum SectionKind{
    Code,
    Data,
    Bss,
    Rodata,
    Stack,
    Heap,
    Vector,
    Other,
}

impl SectionKind {
    pub fn from_name(name: &str) -> Self {
        let name_lower = name.to_lowercase();
        if name_lower.contains("text")                          { SectionKind::Code   }
        else if name_lower.contains("bss")                      { SectionKind::Bss    }
        else if name_lower.contains("rodata")                   { SectionKind::Rodata }
        else if name_lower.contains("data")                     { SectionKind::Data   }
        else if name_lower.contains("stack")                    { SectionKind::Stack  }
        else if name_lower.contains("heap")                     { SectionKind::Heap   }
        else if name_lower.contains("isr") || name_lower.contains("vector") { SectionKind::Vector }
        else                                                          { SectionKind::Other  }
    }
}

// the full parsed picture: regions + all sections.
#[derive(Debug, Default)]
pub struct MemoryMap{
    pub regions: Vec<MemoryRegion>,
    pub sections: Vec<MemorySection>,
    pub source: String // filename.
}

impl MemoryMap {
    // sections that fall inside a given region. sorted by address.
    pub fn section_in(&self, region: &MemoryRegion) -> Vec<&MemorySection> {
        let mut vector_section: Vec<&MemorySection> = self
            .sections
            .iter()
            .filter(|s| region.contains(s) && s.size > 0)
            .collect();
        vector_section.sort_by_key(|s| &s.address);
        vector_section
    }
}