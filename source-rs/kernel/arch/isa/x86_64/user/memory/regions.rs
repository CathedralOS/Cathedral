//! Contiguous ordinary-RAM permission/removal edits, one walk per leaf table.
//! No allocation, ownership transfer or active-root edits occur here.
use super::{Flags, PAGE, PageTable, UserMappings, VirtAddr};

impl UserMappings {
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
            let mut table = root as *mut PageTable;
            for index in [va.p4_index(), va.p3_index(), va.p2_index()] {
                let parent = &*table;
                let entry = &parent[index];
                assert!(
                    entry
                        .flags()
                        .contains(Flags::PRESENT | Flags::USER_ACCESSIBLE)
                );
                assert!(!entry.flags().contains(Flags::HUGE_PAGE));
                table = entry.addr().as_u64() as *mut PageTable;
            }
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
