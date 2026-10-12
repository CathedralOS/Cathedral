//! Borrow core-owned page objects; reserve translation tables at admission.
use super::*;
use cathedral_contracts::memory as wire;
impl UserSpace {
    /// # Safety
    /// Exclusively owned inactive hierarchy; retire the space on allocation failure.
    pub unsafe fn prepare_regions(
        &mut self,
        frames: &mut impl StackFrames,
    ) -> Result<(), MemoryError> {
        for address in (wire::USER_BASE..wire::address(wire::MAX_REGIONS)).step_by(PAGE as usize) {
            // SAFETY: Reserved user branch, inactive tree and tracked frame source.
            unsafe {
                tables::leaf(self.root, address, &mut || self.frame(frames))?;
            }
        }
        Ok(())
    }
    /// # Safety
    /// Inactive prepared tree, IRQs masked; caller retains backing until every alias is retired.
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
            || physical
                .iter()
                .any(|p| !p.is_multiple_of(PAGE) || !(0x4000_0000..0x8000_0000).contains(p))
        {
            return Err(MemoryError::InvalidFrame);
        }
        let base = wire::address(index);
        let flags =
            tables::NORMAL | tables::USER | tables::NX | if writable { 0 } else { tables::RO };
        for (offset, &frame) in physical.iter().enumerate() {
            // SAFETY: Inactive reserved branch; no allocation, all prior edits rolled back on failure.
            unsafe {
                if let Err(error) = tables::map(
                    self.root,
                    base + offset as u64 * PAGE,
                    frame,
                    flags,
                    &mut || Err(MemoryError::OutOfFrames),
                ) {
                    for undo in 0..offset {
                        self.entry(base + undo as u64 * PAGE).write(0);
                    }
                    return Err(error);
                }
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
    /// IRQs masked, inactive tree, fully mapped region; no mutable user reference crosses this change.
    pub unsafe fn seal_region(&mut self, index: usize, pages: usize) {
        for offset in 0..pages {
            let address = wire::address(index) + offset as u64 * PAGE;
            let position = self.backing[..self.pages]
                .iter()
                .position(|p| p.address == address)
                .unwrap();
            // SAFETY: Existing inactive leaf loses write permission; next root load flushes translations.
            unsafe {
                let entry = self.entry(address);
                entry.write(entry.read() | tables::RO);
            }
            self.writable &= !(1 << position);
        }
    }
    /// # Safety
    /// IRQs masked, inactive tree; caller retains backing until all aliases are gone.
    pub unsafe fn unmap_region(&mut self, index: usize, pages: usize) {
        for offset in 0..pages {
            let address = wire::address(index) + offset as u64 * PAGE;
            let position = self.backing[..self.pages]
                .iter()
                .position(|p| p.address == address)
                .unwrap();
            // SAFETY: Existing inactive leaf; backing is still live and not freed here.
            unsafe {
                self.entry(address).write(0);
            }
            self.pages -= 1;
            self.backing[position] = self.backing[self.pages];
            self.writable = (self.writable & !(1 << position))
                | (((self.writable >> self.pages) & 1) << position);
            self.writable &= !(1 << self.pages);
            self.backing[self.pages] = Backing::default();
        }
    }
    unsafe fn entry(&self, address: u64) -> *mut u64 {
        // SAFETY: Caller supplies an existing leaf in this inactive owned tree.
        unsafe { tables::leaf(self.root, address, &mut || Err(MemoryError::OutOfFrames)).unwrap() }
    }
}
