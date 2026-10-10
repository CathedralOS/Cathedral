//! Runtime page-object mappings borrow core-owned frames. Their page tables are reserved at admission.
//! All edits occur on the boot stack with kernel CR3 active; the next CR3 load flushes user translations.
use super::*;
use cathedral_contracts::memory as wire;
struct NoFrames;
// SAFETY: Runtime mappings use only the preallocated hierarchy and never allocate a frame.
unsafe impl FrameAllocator<Size4KiB> for NoFrames {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}
impl UserSpace {
    /// Reserve the single page table covering all bounded runtime slots.
    /// # Safety
    /// Inactive exclusively owned hierarchy; same custody as from_segments. On error retire the space.
    pub unsafe fn prepare_regions(
        &mut self,
        frames: &mut impl StackFrames,
    ) -> Result<(), MemoryError> {
        const _: () = assert!(wire::address(wire::MAX_REGIONS) - wire::USER_BASE < 2 * 1024 * 1024);
        let physical = self
            .backing
            .iter()
            .find(|p| p.address == USER_STACK)
            .unwrap()
            .physical;
        let mut source = Tracked {
            frames,
            addresses: &mut self.owned,
            len: &mut self.len,
        };
        // SAFETY: Exclusive inactive tables; temporary alias is never published or recorded as owned RAM.
        unsafe {
            let mut target =
                OffsetPageTable::new(&mut *(self.root as *mut PageTable), VirtAddr::zero());
            map(
                &mut target,
                wire::USER_BASE,
                physical,
                Flags::PRESENT | Flags::USER_ACCESSIBLE | Flags::NO_EXECUTE,
                &mut source,
            )?;
            target
                .unmap(
                    Page::<Size4KiB>::from_start_address(VirtAddr::new(wire::USER_BASE)).unwrap(),
                )
                .unwrap()
                .1
                .ignore();
        }
        Ok(())
    }
    /// Map existing live frames without allocating tables or taking backing ownership.
    /// # Safety
    /// Kernel CR3, IRQs off, exclusive inactive tables prepared above. Caller retains the frames
    /// until every alias is removed; writable mappings require exclusive producer custody.
    pub unsafe fn map_region(
        &mut self,
        index: usize,
        physical: &[u64],
        writable: bool,
    ) -> Result<(), MemoryError> {
        if index >= wire::MAX_REGIONS
            || physical.is_empty()
            || physical.len() > wire::MAX_PAGES
            || self.pages + physical.len() > MAX_BACKING
        {
            return Err(MemoryError::InvalidFrame);
        }
        let base = wire::address(index);
        if self.backing[..self.pages]
            .iter()
            .any(|p| p.address >= base && p.address < base + (wire::MAX_PAGES as u64 * PAGE))
        {
            return Err(MemoryError::OccupiedVirtualRange);
        }
        let mut flags = Flags::PRESENT | Flags::USER_ACCESSIBLE | Flags::NO_EXECUTE;
        if writable {
            flags |= Flags::WRITABLE;
        }
        // SAFETY: Caller owns the inactive tables; all slots are in the preallocated branch.
        let mut target =
            unsafe { OffsetPageTable::new(&mut *(self.root as *mut PageTable), VirtAddr::zero()) };
        for (offset, &frame) in physical.iter().enumerate() {
            let address = base + offset as u64 * PAGE;
            // SAFETY: Valid frame supplied by caller, no running aliases; full range checked above.
            if let Err(error) = unsafe { map(&mut target, address, frame, flags, &mut NoFrames) } {
                for undo in 0..offset {
                    target
                        .unmap(
                            Page::<Size4KiB>::from_start_address(VirtAddr::new(
                                base + undo as u64 * PAGE,
                            ))
                            .unwrap(),
                        )
                        .unwrap()
                        .1
                        .ignore();
                }
                return Err(error);
            }
        }
        for (offset, &frame) in physical.iter().enumerate() {
            self.backing[self.pages] = Backing {
                address: base + offset as u64 * PAGE,
                physical: frame,
            };
            if writable {
                self.writable |= 1 << self.pages;
            }
            self.pages += 1;
        }
        Ok(())
    }
    /// # Safety
    /// Kernel root/IRQs off, exclusively owned inactive space; region is fully mapped.
    /// No mutable userspace Rust borrow may cross this ownership transition.
    pub unsafe fn seal_region(&mut self, index: usize, pages: usize) {
        // SAFETY: Caller serializes page-table mutation while this root cannot execute.
        let mut target =
            unsafe { OffsetPageTable::new(&mut *(self.root as *mut PageTable), VirtAddr::zero()) };
        for offset in 0..pages {
            let address = wire::address(index) + offset as u64 * PAGE;
            let position = self.backing[..self.pages]
                .iter()
                .position(|p| p.address == address)
                .unwrap();
            // SAFETY: Existing leaf retains its backing; permissions only lose WRITE.
            unsafe {
                target
                    .update_flags(
                        Page::<Size4KiB>::from_start_address(VirtAddr::new(address)).unwrap(),
                        Flags::PRESENT | Flags::USER_ACCESSIBLE | Flags::NO_EXECUTE,
                    )
                    .unwrap()
                    .ignore();
            }
            self.writable &= !(1 << position);
        }
    }
    /// # Safety
    /// Kernel root/IRQs off, inactive space. No resumed code or safe reference may use the removed mapping.
    pub unsafe fn unmap_region(&mut self, index: usize, pages: usize) {
        // SAFETY: Caller serializes edits to inactive tables; backing is retained until all aliases disappear.
        let mut target =
            unsafe { OffsetPageTable::new(&mut *(self.root as *mut PageTable), VirtAddr::zero()) };
        for offset in 0..pages {
            let address = wire::address(index) + offset as u64 * PAGE;
            let position = self.backing[..self.pages]
                .iter()
                .position(|p| p.address == address)
                .unwrap();
            target
                .unmap(Page::<Size4KiB>::from_start_address(VirtAddr::new(address)).unwrap())
                .unwrap()
                .1
                .ignore();
            self.pages -= 1;
            self.backing[position] = self.backing[self.pages];
            self.writable = (self.writable & !(1 << position))
                | (((self.writable >> self.pages) & 1) << position);
            self.writable &= !(1 << self.pages);
            self.backing[self.pages] = Backing::default();
        }
    }
}
