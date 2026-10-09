//! Mirrors the responsibility of Cathedral's semantic boot handoff.
//! These records validate geometry; constructing them does not prove custody.

pub const PAGE_SIZE: u64 = 4096;
pub const MAX_MEMORY_REGIONS: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryKind {
    Usable,
    Reserved,
}

#[derive(Clone, Copy, Debug)]
pub struct MemoryRegion {
    pub base: u64,
    pub page_count: u64,
    pub kind: MemoryKind,
}

impl MemoryRegion {
    pub fn end(&self) -> Option<u64> {
        self.base
            .checked_add(self.page_count.checked_mul(PAGE_SIZE)?)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum MemoryMapError {
    Capacity,
    EmptyRegion,
    Misaligned,
    Overflow,
    Overlap,
}

/// Fixed storage so handoff needs no allocator after ExitBootServices.
/// Intentionally not Clone: the boot path moves its inventory to the core.
pub struct BootMemory {
    regions: [MemoryRegion; MAX_MEMORY_REGIONS],
    len: usize,
}

impl Default for BootMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl BootMemory {
    pub const fn new() -> Self {
        Self {
            regions: [MemoryRegion {
                base: 0,
                page_count: 0,
                kind: MemoryKind::Reserved,
            }; MAX_MEMORY_REGIONS],
            len: 0,
        }
    }

    pub fn regions(&self) -> &[MemoryRegion] {
        &self.regions[..self.len]
    }

    pub fn push(&mut self, region: MemoryRegion) -> Result<(), MemoryMapError> {
        if self.len == MAX_MEMORY_REGIONS {
            return Err(MemoryMapError::Capacity);
        }
        if region.page_count == 0 {
            return Err(MemoryMapError::EmptyRegion);
        }
        if !region.base.is_multiple_of(PAGE_SIZE) {
            return Err(MemoryMapError::Misaligned);
        }
        let end = region.end().ok_or(MemoryMapError::Overflow)?;
        for existing in self.regions() {
            if region.base < existing.end().unwrap() && existing.base < end {
                return Err(MemoryMapError::Overlap);
            }
        }
        self.regions[self.len] = region;
        self.len += 1;
        Ok(())
    }

    pub fn usable_bytes(&self) -> u64 {
        // Validated disjoint, non-wrapping regions cannot sum beyond u64.
        self.regions()
            .iter()
            .filter(|r| r.kind == MemoryKind::Usable)
            .map(|r| r.page_count * PAGE_SIZE)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_bad_geometry_and_reserved_overlap_without_changing_inventory() {
        let mut map = BootMemory::new();
        let region = MemoryRegion {
            base: 4096,
            page_count: 2,
            kind: MemoryKind::Reserved,
        };
        map.push(region).unwrap();
        assert_eq!(
            map.push(MemoryRegion {
                kind: MemoryKind::Usable,
                ..region
            }),
            Err(MemoryMapError::Overlap)
        );
        assert_eq!(
            map.push(MemoryRegion { base: 1, ..region }),
            Err(MemoryMapError::Misaligned)
        );
        assert_eq!(
            map.push(MemoryRegion {
                page_count: 0,
                ..region
            }),
            Err(MemoryMapError::EmptyRegion)
        );
        assert_eq!(
            map.push(MemoryRegion {
                base: u64::MAX - 4095,
                ..region
            }),
            Err(MemoryMapError::Overflow)
        );
        assert_eq!(
            map.push(MemoryRegion {
                page_count: u64::MAX,
                ..region
            }),
            Err(MemoryMapError::Overflow)
        );
        assert_eq!(map.regions().len(), 1);
        assert_eq!(map.usable_bytes(), 0);
    }

    #[test]
    fn accepts_adjacent_unsorted_regions_and_fails_closed_at_capacity() {
        let mut map = BootMemory::new();
        for i in (0..MAX_MEMORY_REGIONS).rev() {
            map.push(MemoryRegion {
                base: i as u64 * PAGE_SIZE,
                page_count: 1,
                kind: MemoryKind::Usable,
            })
            .unwrap();
        }
        assert_eq!(map.usable_bytes(), MAX_MEMORY_REGIONS as u64 * PAGE_SIZE);
        assert_eq!(
            map.push(MemoryRegion {
                base: MAX_MEMORY_REGIONS as u64 * PAGE_SIZE,
                page_count: 1,
                kind: MemoryKind::Usable
            }),
            Err(MemoryMapError::Capacity)
        );
    }
}
