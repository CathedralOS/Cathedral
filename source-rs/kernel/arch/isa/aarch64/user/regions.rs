//! Contiguous ordinary-RAM edits, validating the whole range before installation.
use super::super::memory::tables;
use super::mappings::UserMappings;
use crate::{MemoryError, memory::PAGE};

impl UserMappings {
    /// Install RW/NX or RO/NX leaves into reserved tables. Errors leave every
    /// parent and leaf unchanged; this operation never allocates tables.
    /// # Safety
    /// Exclusive inactive owned tree, IRQs off; valid user VA range and retained
    /// ordinary RAM backing. Writable mappings require exclusive producer custody.
    /// Reserved ancestors must already permit user access.
    pub unsafe fn map_pages(
        &mut self,
        address: u64,
        physical: &[u64],
        writable: bool,
    ) -> Result<(), MemoryError> {
        let last = physical
            .len()
            .checked_sub(1)
            .and_then(|count| (count as u64).checked_mul(PAGE))
            .and_then(|offset| address.checked_add(offset));
        if !address.is_multiple_of(PAGE)
            || address >= 1 << 48
            || last.is_none_or(|last| last >= 1 << 48)
            || physical.iter().any(|frame| {
                !frame.is_multiple_of(PAGE)
                    || !(crate::board::virt::RAM_START..crate::board::virt::RAM_END).contains(frame)
            })
        {
            return Err(MemoryError::InvalidFrame);
        }
        let flags =
            tables::NORMAL | tables::USER | tables::NX | 3 | if writable { 0 } else { tables::RO };
        // SAFETY: Exclusive inactive tree throughout both passes. A missing
        // branch returns before any write. Commit cannot fail after preflight;
        // there is no allocation or external work between validation and writes.
        unsafe {
            visit(self.root(), address, physical.len(), |_, first, count| {
                for offset in 0..count {
                    if first.add(offset).read() & 1 != 0 {
                        return Err(MemoryError::OccupiedVirtualRange);
                    }
                }
                Ok(())
            })?;
            visit(
                self.root(),
                address,
                physical.len(),
                |offset, first, count| {
                    for index in 0..count {
                        first.add(index).write(physical[offset + index] | flags);
                    }
                    Ok(())
                },
            )
            .expect("validated user tables changed during installation");
        }
        Ok(())
    }

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

unsafe fn visit(
    root: u64,
    mut address: u64,
    pages: usize,
    mut action: impl FnMut(usize, *mut u64, usize) -> Result<(), MemoryError>,
) -> Result<(), MemoryError> {
    let mut offset = 0;
    while offset < pages {
        let count = (pages - offset).min(512 - ((address / PAGE) & 511) as usize);
        // SAFETY: Existing inactive tables. Refusing allocation rejects a
        // missing branch without modifying it; the window ends inside one table.
        unsafe {
            let first = tables::leaf(root, address, &mut || Err(MemoryError::OutOfFrames))?;
            action(offset, first, count)?;
        }
        offset += count;
        if offset < pages {
            address += count as u64 * PAGE;
        }
    }
    Ok(())
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
