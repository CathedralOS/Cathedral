// Inactive x86 user translations. Core owns frames, executable layout, copies and rollback.
// creation borrows entry mappings; device handles PAT; regions batches ordinary RAM edits.
use crate::{
    FrameSource, MemoryError,
    memory::{PAGE, Permissions},
};
use ::x86_64::{
    VirtAddr,
    registers::control::Cr3,
    structures::paging::{
        Mapper, OffsetPageTable, Page, PageTable, PageTableFlags as Flags, Size4KiB,
    },
};
pub struct UserMappings {
    root: u64,
    kernel: creation::Kernel,
}
impl UserMappings {
    pub fn root(&self) -> u64 {
        assert!(self.kernel.done(), "incomplete supervisor mappings");
        self.root
    }
    pub fn assert_inactive(&self) {
        assert_ne!(Cr3::read().0.start_address().as_u64(), self.root);
    }
    /// # Safety
    /// IRQs off, kernel root active, exclusive inactive tree; VA/frame aligned and valid.
    /// Caller owns backing and all newly allocated table frames; retire the tree on allocation error.
    pub unsafe fn map_page(
        &mut self,
        address: u64,
        physical: u64,
        permissions: Permissions,
        frames: &mut impl FrameSource,
    ) -> Result<(), MemoryError> {
        let flags = Flags::PRESENT
            | Flags::USER_ACCESSIBLE
            | if permissions.writable {
                Flags::WRITABLE
            } else {
                Flags::empty()
            }
            | if permissions.executable {
                Flags::empty()
            } else {
                Flags::NO_EXECUTE
            };
        // SAFETY: Caller supplies an inactive exclusively owned hierarchy and pinned backing.
        unsafe {
            let mut target =
                OffsetPageTable::new(&mut *(self.root as *mut PageTable), VirtAddr::zero());
            paging::map(
                &mut target,
                address,
                physical,
                flags,
                &mut paging::Tables(frames),
            )
        }
    }
    /// # Safety
    /// Same exclusive inactive-tree custody; scratch is pinned RAM. Range is page aligned,
    /// vacant, and within a disjoint user branch. Temporary aliases are never executed.
    pub unsafe fn reserve(
        &mut self,
        start: u64,
        end: u64,
        scratch: u64,
        frames: &mut impl FrameSource,
    ) -> Result<(), MemoryError> {
        // SAFETY: Map/unmap one scratch alias per leaf-table span; all tables remain owned.
        unsafe {
            let mut address = start;
            while address < end {
                self.map_page(
                    address,
                    scratch,
                    Permissions {
                        // Reserve writable parent tables now. The temporary
                        // leaf is removed before publication; runtime mapping
                        // must never widen parent permissions as a side effect.
                        writable: true,
                        executable: false,
                    },
                    frames,
                )?;
                self.unmap_page(address);
                address = (address | ((1 << 21) - 1)) + 1;
            }
        }
        Ok(())
    }
    /// # Safety
    /// Inactive tree, IRQs off; existing leaf, backing retained until every alias is removed.
    unsafe fn unmap_page(&mut self, address: u64) {
        // SAFETY: Caller retains backing; no table/frame ownership is consumed here.
        unsafe {
            let mut target =
                OffsetPageTable::new(&mut *(self.root as *mut PageTable), VirtAddr::zero());
            target
                .unmap(Page::<Size4KiB>::from_start_address(VirtAddr::new(address)).unwrap())
                .unwrap()
                .1
                .ignore();
        }
    }
    /// # Safety
    /// Newly loaded private executable RAM, not published to any running task.
    pub unsafe fn publish_code(_physical: u64) { /* Coherent x86 instruction/data caches. */
    }
}
