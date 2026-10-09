//! Bootstrap frame accounting over the inventory moved in by boot.
//! Bootstrap allocations become permanent when the reclaiming bitmap is enabled.
//! Later frames can be returned after their mappings and users are gone.

use alloc::{vec, vec::Vec};
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

/// Reserved regions and physical page zero are never allocated. The optional
/// bitmap needs the heap, so bootstrap allocation remains allocation-free.
pub struct FrameAllocator {
    memory: BootMemory,
    region_index: usize,
    page_index: u64,
    allocated: usize,
    reclaim: Option<Reclamation>,
}

struct Reclamation {
    used: Vec<u64>,
    total: usize,
    permanent: usize,
    cursor: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReleaseError {
    NotEnabled,
    InvalidFrame,
    PermanentFrame,
    AlreadyFree,
}

impl FrameAllocator {
    pub fn new(memory: BootMemory) -> Self {
        Self {
            memory,
            region_index: 0,
            page_index: 0,
            allocated: 0,
            reclaim: None,
        }
    }

    pub fn allocate(&mut self) -> Option<PhysicalFrame> {
        if let Some(reclaim) = &mut self.reclaim {
            let index = (0..reclaim.total)
                .map(|offset| (reclaim.cursor + offset) % reclaim.total)
                .find(|index| reclaim.used[index / 64] & (1 << (index % 64)) == 0)?;
            reclaim.used[index / 64] |= 1 << (index % 64);
            reclaim.cursor = (index + 1) % reclaim.total;
            self.allocated += 1;
            return Some(PhysicalFrame {
                address: self.address_at(index),
            });
        }
        while let Some(region) = self.memory.regions().get(self.region_index) {
            if region.kind == MemoryKind::Usable && self.page_index < region.page_count {
                let address = region.base + self.page_index * PAGE_SIZE;
                self.page_index += 1;
                if address != 0 {
                    self.allocated += 1;
                    return Some(PhysicalFrame { address });
                }
            } else {
                self.region_index += 1;
                self.page_index = 0;
            }
        }
        None
    }

    /// Enable once, after installing the heap. Existing boot frames stay pinned.
    pub fn enable_reclamation(&mut self) {
        assert!(self.reclaim.is_none(), "frame reclamation already enabled");
        let total = self.usable_regions().map(|(_, count)| count).sum::<usize>();
        let mut used = vec![0u64; total.div_ceil(64)];
        for index in 0..self.allocated {
            used[index / 64] |= 1 << (index % 64);
        }
        self.reclaim = Some(Reclamation {
            used,
            total,
            permanent: self.allocated,
            cursor: self.allocated,
        });
    }

    pub fn allocated(&self) -> usize {
        self.allocated
    }

    /// Return an owned frame. Mapping users must relinquish it before this call.
    pub fn release(&mut self, frame: PhysicalFrame) -> Result<(), ReleaseError> {
        self.release_address(frame.address)
    }

    // Architecture adapters recover frame addresses after unmapping. This only
    // changes accounting; their unsafe contracts establish actual machine custody.
    pub(crate) fn release_address(&mut self, address: u64) -> Result<(), ReleaseError> {
        if !address.is_multiple_of(PAGE_SIZE) {
            return Err(ReleaseError::InvalidFrame);
        }
        let mut offset = 0;
        let mut found = None;
        for (base, count) in self.usable_regions() {
            if (base..base + count as u64 * PAGE_SIZE).contains(&address) {
                found = Some(offset + ((address - base) / PAGE_SIZE) as usize);
                break;
            }
            offset += count;
        }
        let index = found.ok_or(ReleaseError::InvalidFrame)?;
        let reclaim = self.reclaim.as_mut().ok_or(ReleaseError::NotEnabled)?;
        if index < reclaim.permanent {
            return Err(ReleaseError::PermanentFrame);
        }
        let mask = 1 << (index % 64);
        if reclaim.used[index / 64] & mask == 0 {
            return Err(ReleaseError::AlreadyFree);
        }
        reclaim.used[index / 64] &= !mask;
        reclaim.cursor = reclaim.cursor.min(index);
        self.allocated -= 1;
        Ok(())
    }

    fn usable_regions(&self) -> impl Iterator<Item = (u64, usize)> + '_ {
        self.memory
            .regions()
            .iter()
            .filter(|r| r.kind == MemoryKind::Usable)
            .map(|r| {
                let base = r.base.max(PAGE_SIZE);
                (base, ((r.end().unwrap() - base) / PAGE_SIZE) as usize)
            })
    }

    fn address_at(&self, mut index: usize) -> u64 {
        for (base, count) in self.usable_regions() {
            if index < count {
                return base + index as u64 * PAGE_SIZE;
            }
            index -= count;
        }
        unreachable!("bitmap index outside inventory")
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

    #[test]
    fn reclaim_reuses_frames_but_rejects_boot_reserved_null_and_double_free() {
        let mut map = BootMemory::new();
        // Deliberately unsorted, crossing a bitmap word boundary.
        for (base, page_count, kind) in [
            (0x100000, 65, MemoryKind::Usable),
            (0, 3, MemoryKind::Usable),
            (0x3000, 5, MemoryKind::Reserved),
        ] {
            map.push(MemoryRegion {
                base,
                page_count,
                kind,
            })
            .unwrap();
        }
        let mut allocator = FrameAllocator::new(map);
        let boot = allocator.allocate().unwrap();
        allocator.enable_reclamation();
        let mut frames = Vec::new();
        while let Some(frame) = allocator.allocate() {
            frames.push(frame);
        }
        assert_eq!(frames.len(), 66);
        assert_eq!(allocator.release(boot), Err(ReleaseError::PermanentFrame));
        for bad in [0, 0x3000, 0x100001] {
            assert_eq!(
                allocator.release_address(bad),
                Err(ReleaseError::InvalidFrame)
            );
        }
        let frame = frames.pop().unwrap();
        let address = frame.address();
        allocator.release(frame).unwrap();
        assert_eq!(
            allocator.release_address(address),
            Err(ReleaseError::AlreadyFree)
        );
        assert_eq!(allocator.allocate().unwrap().address(), address);
        assert!(allocator.allocate().is_none());
        for frame in frames {
            allocator.release(frame).unwrap();
        }
        assert_eq!(allocator.allocated(), 2);
    }
}
