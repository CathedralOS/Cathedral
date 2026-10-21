//! Contiguous ordinary-RAM edits, with whole-range validation before installation.
//! No allocation, ownership transfer or active-root edits occur here.
use super::{Flags, PAGE, PageTable, UserMappings, VirtAddr};
use crate::MemoryError;
use x86_64::PhysAddr;

impl UserMappings {
    /// Install RW/NX or RO/NX leaves into reserved user tables. On error neither
    /// parent nor leaf entries change; no table allocation is attempted.
    /// # Safety
    /// Exclusive inactive owned tree, IRQs off; valid user VA range and retained
    /// ordinary RAM backing. Writable mappings require exclusive producer custody.
    /// Reserved ancestors must already permit the requested user access.
    pub unsafe fn map_pages(
        &mut self,
        address: u64,
        physical: &[u64],
        writable: bool,
    ) -> Result<(), MemoryError> {
        // SAFETY: Caller excludes execution/edits throughout preflight and commit.
        unsafe { install(self.root, address, physical, writable) }
    }

    /// # Safety
    /// Inactive exclusively owned tree, IRQs off; aligned, fully mapped user
    /// RAM range. All mutable borrowers have retired. Next root load flushes TLB.
    pub unsafe fn seal_pages(&mut self, address: u64, pages: usize) {
        // SAFETY: Caller retains all mappings/backing throughout this atomic edit.
        unsafe { edit(self.root, address, pages, true) };
    }

    /// # Safety
    /// Inactive exclusively owned tree, IRQs off; aligned, fully mapped user
    /// RAM range. Backing remains pinned until every alias has retired.
    pub unsafe fn unmap_pages(&mut self, address: u64, pages: usize) {
        // SAFETY: Remove translations only; tables and backing remain caller-owned.
        unsafe { edit(self.root, address, pages, false) };
    }
}

pub(super) unsafe fn edit(root: u64, mut address: u64, mut pages: usize, seal: bool) {
    assert!(address.is_multiple_of(PAGE));
    while pages != 0 {
        let va = VirtAddr::new(address);
        let first = usize::from(va.p1_index());
        let count = pages.min(512 - first);
        // SAFETY: Exclusive inactive hierarchy. Each parent must be a present
        // user table, never a huge-page leaf; all child pointers are identity RAM.
        unsafe {
            let table = leaf_table(root, va, false).expect("missing user table");
            let leaves = &mut *table;
            for index in first..first + count {
                let entry = &mut leaves[index];
                assert!(
                    entry
                        .flags()
                        .contains(Flags::PRESENT | Flags::USER_ACCESSIBLE)
                );
                if seal {
                    entry.set_flags(Flags::PRESENT | Flags::USER_ACCESSIBLE | Flags::NO_EXECUTE);
                } else {
                    entry.set_unused();
                }
            }
        }
        pages -= count;
        if pages != 0 {
            address = address.checked_add(count as u64 * PAGE).unwrap();
        }
    }
}

pub(super) unsafe fn install(
    root: u64,
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
        || VirtAddr::try_new(address).is_err()
        || last.is_none_or(|last| VirtAddr::try_new(last).is_err())
        || physical.iter().any(|&frame| {
            frame == 0 || !frame.is_multiple_of(PAGE) || PhysAddr::try_new(frame).is_err()
        })
    {
        return Err(MemoryError::InvalidFrame);
    }
    let flags = Flags::PRESENT
        | Flags::USER_ACCESSIBLE
        | Flags::NO_EXECUTE
        | if writable {
            Flags::WRITABLE
        } else {
            Flags::empty()
        };
    // SAFETY: Caller retains exclusive custody throughout both passes. Preflight
    // rejects missing/restrictive parents and occupied leaves without writing.
    // Commit cannot fail after validation; no allocator or external work intervenes.
    unsafe {
        visit(
            root,
            address,
            physical.len(),
            writable,
            |_, table, range| {
                for index in range {
                    if !table[index].is_unused() {
                        return Err(MemoryError::OccupiedVirtualRange);
                    }
                }
                Ok(())
            },
        )?;
        visit(
            root,
            address,
            physical.len(),
            writable,
            |offset, table, range| {
                for (index, &frame) in range.zip(&physical[offset..]) {
                    table[index].set_addr(PhysAddr::new(frame), flags);
                }
                Ok(())
            },
        )
        .expect("validated user tables changed during installation");
    }
    Ok(())
}

unsafe fn visit(
    root: u64,
    mut address: u64,
    pages: usize,
    writable: bool,
    mut action: impl FnMut(usize, &mut PageTable, core::ops::Range<usize>) -> Result<(), MemoryError>,
) -> Result<(), MemoryError> {
    let mut offset = 0;
    while offset < pages {
        let va = VirtAddr::new(address);
        let first = usize::from(va.p1_index());
        let count = (pages - offset).min(512 - first);
        // SAFETY: Owned inactive tables; range ends at the current table boundary.
        unsafe {
            let table = &mut *leaf_table(root, va, writable)?;
            action(offset, table, first..first + count)?;
        }
        offset += count;
        if offset < pages {
            address += count as u64 * PAGE;
        }
    }
    Ok(())
}

unsafe fn leaf_table(
    root: u64,
    va: VirtAddr,
    writable: bool,
) -> Result<*mut PageTable, MemoryError> {
    let required = Flags::PRESENT
        | Flags::USER_ACCESSIBLE
        | if writable {
            Flags::WRITABLE
        } else {
            Flags::empty()
        };
    let mut table = root as *mut PageTable;
    for index in [va.p4_index(), va.p3_index(), va.p2_index()] {
        // SAFETY: Caller owns the inactive tree; follow only validated table entries.
        unsafe {
            let parent = &*table;
            let entry = &parent[index];
            if !entry.flags().contains(Flags::PRESENT) {
                return Err(MemoryError::OutOfFrames);
            }
            if !entry.flags().contains(required) || entry.flags().contains(Flags::HUGE_PAGE) {
                return Err(MemoryError::OccupiedVirtualRange);
            }
            table = entry.addr().as_u64() as *mut PageTable;
        }
    }
    Ok(table)
}
