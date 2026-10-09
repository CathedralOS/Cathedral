//! Bootstrap frame accounting over the inventory moved in by boot.
//! No mapping, dereferencing, reclaim, or Omega authority qualification yet.

use cathedral_contracts::boot::{BootMemory, MemoryKind, PAGE_SIZE};

#[derive(Debug)]
pub struct PhysicalFrame {
    address: u64,
}

impl PhysicalFrame {
    pub fn address(&self) -> u64 {
        self.address
    }
}

/// Monotonic allocation. Reserved regions and physical page zero are skipped.
/// No free operation until mapping and lifetime tracking exist.
pub struct FrameAllocator {
    memory: BootMemory,
    region_index: usize,
    page_index: u64,
}

impl FrameAllocator {
    pub fn new(memory: BootMemory) -> Self {
        Self {
            memory,
            region_index: 0,
            page_index: 0,
        }
    }

    pub fn allocate(&mut self) -> Option<PhysicalFrame> {
        while let Some(region) = self.memory.regions().get(self.region_index) {
            if region.kind == MemoryKind::Usable && self.page_index < region.page_count {
                let address = region.base + self.page_index * PAGE_SIZE;
                self.page_index += 1;
                if address != 0 {
                    return Some(PhysicalFrame { address });
                }
            } else {
                self.region_index += 1;
                self.page_index = 0;
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cathedral_contracts::boot::MemoryRegion;

    #[test]
    fn allocations_skip_reserved_and_null_frames_and_exhaust_without_reuse() {
        let mut map = BootMemory::new();
        for (base, page_count, kind) in [
            (0, 2, MemoryKind::Usable),
            (8192, 2, MemoryKind::Reserved),
            (16384, 2, MemoryKind::Usable),
        ] {
            map.push(MemoryRegion {
                base,
                page_count,
                kind,
            })
            .unwrap();
        }
        let mut allocator = FrameAllocator::new(map);
        for expected in [4096, 16384, 20480] {
            assert_eq!(allocator.allocate().unwrap().address(), expected);
        }
        assert!(allocator.allocate().is_none());
        assert!(allocator.allocate().is_none());
    }

    #[test]
    fn empty_inventory_is_exhausted() {
        assert!(FrameAllocator::new(BootMemory::new()).allocate().is_none());
    }
}
