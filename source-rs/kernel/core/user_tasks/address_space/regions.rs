//! Borrow page-object backing; reserve tables before runtime edits and publish bookkeeping atomically.
use super::*;
use cathedral_arch::memory::{NoFrames, Permissions};
use cathedral_kernel_contracts::memory as wire;
impl UserSpace {
    /// # Safety
    /// IRQs masked, inactive owned tree. On failure retire this entire space.
    pub unsafe fn prepare_regions(
        &mut self,
        frames: &mut impl FrameSource,
    ) -> Result<(), MemoryError> {
        let scratch = self.backing[..self.pages]
            .iter()
            .find(|p| p.address == USER_STACK)
            .unwrap()
            .physical;
        let mut source = Tracked {
            frames,
            addresses: &mut self.owned,
            len: &mut self.len,
        };
        // SAFETY: Reserve inactive tables only; scratch stays pinned throughout construction.
        unsafe {
            self.mapping.as_mut().unwrap().reserve(
                wire::USER_BASE,
                wire::address(wire::MAX_MAPPING_SLOTS),
                scratch,
                &mut source,
            )
        }
    }
    /// # Safety
    /// Inactive prepared tree, IRQs masked. Caller retains all backing until every alias retires;
    /// writable mappings require exclusive producer custody.
    pub unsafe fn map_region(
        &mut self,
        index: usize,
        physical: &[u64],
        writable: bool,
    ) -> Result<(), MemoryError> {
        if index >= wire::MAX_MAPPING_SLOTS
            || physical.is_empty()
            || physical.len() > wire::MAX_PAGES
            || self.pages + physical.len() > MAX_BACKING
            || physical.iter().any(|p| *p == 0 || !p.is_multiple_of(PAGE))
        {
            return Err(MemoryError::InvalidFrame);
        }
        let base = wire::address(index);
        if self.backing[..self.pages]
            .iter()
            .any(|p| p.address >= base && p.address < base + wire::MAX_PAGES as u64 * PAGE)
        {
            return Err(MemoryError::OccupiedVirtualRange);
        }
        self.backing
            .try_reserve_exact(physical.len())
            .map_err(|_| MemoryError::OutOfFrames)?;
        let permissions = Permissions {
            writable,
            executable: false,
        };
        for (offset, &frame) in physical.iter().enumerate() {
            // SAFETY: Inactive reserved branch; no allocation or running aliases. Roll back earlier leaves on failure.
            unsafe {
                if let Err(error) = self.mapping.as_mut().unwrap().map_page(
                    base + offset as u64 * PAGE,
                    frame,
                    permissions,
                    &mut NoFrames,
                ) {
                    for undo in 0..offset {
                        self.mapping
                            .as_mut()
                            .unwrap()
                            .unmap_page(base + undo as u64 * PAGE);
                    }
                    return Err(error);
                }
            }
        }
        for (offset, &frame) in physical.iter().enumerate() {
            self.backing.push(Backing {
                address: base + offset as u64 * PAGE,
                physical: frame,
                writable,
            });
            self.pages += 1;
        }
        Ok(())
    }
    /// # Safety
    /// Inactive root, IRQs masked, fully mapped region; no mutable borrow crosses sealing.
    pub unsafe fn seal_region(&mut self, index: usize, pages: usize) {
        let base = wire::address(index);
        let end = base + pages as u64 * PAGE;
        let mut sealed = 0;
        // Visit the ledger once, rather than search it again for every page.
        // Backing order need not match virtual address order.
        for backing in &mut self.backing[..self.pages] {
            if !(base..end).contains(&backing.address) {
                continue;
            }
            // SAFETY: Existing ordinary user RAM leaf; only remove write permission.
            unsafe {
                self.mapping.as_mut().unwrap().seal_page(backing.address);
            }
            backing.writable = false;
            sealed += 1;
        }
        assert_eq!(sealed, pages, "incomplete region during sealing");
    }
    /// # Safety
    /// Inactive root, IRQs masked; backing retained until every alias is removed.
    pub unsafe fn unmap_region(&mut self, index: usize, pages: usize) {
        let base = wire::address(index);
        let end = base + pages as u64 * PAGE;
        let mapping = self.mapping.as_mut().unwrap();
        let mut removed = 0;
        assert_eq!(self.backing.len(), self.pages);
        // Compact once, preserving every unrelated image, stack and object leaf.
        self.backing.retain(|backing| {
            if !(base..end).contains(&backing.address) {
                return true;
            }
            // SAFETY: Existing inactive leaf; translation removal precedes any backing refund.
            unsafe {
                mapping.unmap_page(backing.address);
            }
            removed += 1;
            false
        });
        self.pages -= removed;
        assert_eq!(removed, pages, "incomplete region during unmapping");
    }
}
