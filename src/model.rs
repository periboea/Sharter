#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub name: String,
    pub origin: u64,
    pub length: u64,
    pub attributes: Option<String>
}

impl MemoryRegion {
    pub fn end(&self) -> u64{
        self.origin.saturating_add(self.length)
    }

    pub fn contains(&self, s: &MemorySection) -> bool {
        s.addr >= self.origin && s.addr < self.end()
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
    addr: u64,
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
