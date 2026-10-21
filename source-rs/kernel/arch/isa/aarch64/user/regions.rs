//! Contiguous ordinary-RAM edits visit each 4-KiB-granule leaf table once.
use super::super::memory::tables;
use super::mappings::UserMappings;
use crate::{MemoryError, memory::PAGE};

impl UserMappings {
    /// # Safety
    /// Inactive owned tree, IRQs off; aligned, fully mapped user RAM with no
    /// mutable borrowers. Next root activation publishes edits and flushes TLB.
    pub unsafe fn seal_pages(&mut self, address: u64, pages: usize) {
        // SAFETY: Retained translations and backing, only removing write access.
        unsafe { edit(self.root(), address, pages, true) };
    }

    /// # Safety
    /// Inactive owned tree, IRQs off; aligned, fully mapped user RAM. Caller
    /// retains backing until all aliases are removed; table ownership is unchanged.
    pub unsafe fn unmap_pages(&mut self, address: u64, pages: usize) {
        // SAFETY: Caller owns the entire range and retains all backing.
        unsafe { edit(self.root(), address, pages, false) };
    }
}

unsafe fn edit(root: u64, mut address: u64, mut pages: usize, seal: bool) {
    assert!(address.is_multiple_of(PAGE));
    while pages != 0 {
        let count = pages.min(512 - ((address / PAGE) & 511) as usize);
        // SAFETY: Existing inactive tables; refusing allocation also rejects any
        // missing branch. Stop at the table boundary before walking again.
        unsafe {
            let first = tables::leaf(root, address, &mut || Err(MemoryError::OutOfFrames))
                .expect("missing user table");
            for offset in 0..count {
                let entry = first.add(offset);
                let descriptor = entry.read();
                assert_eq!(descriptor & (3 | tables::USER), 3 | tables::USER);
                entry.write(if seal { descriptor | tables::RO } else { 0 });
            }
        }
        pages -= count;
        if pages != 0 {
            address = address.checked_add(count as u64 * PAGE).unwrap();
        }
    }
}
