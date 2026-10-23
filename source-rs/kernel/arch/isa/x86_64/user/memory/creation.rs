//! Build supervisor entry mappings in bounded batches into a private root.
use super::paging::{Tables, map};
use super::{PAGE, UserMappings};
use crate::{BootLayout, FrameSource, ImageRange, MemoryError};
use ::x86_64::{
    VirtAddr,
    structures::paging::{
        FrameAllocator, OffsetPageTable, PageTable, PageTableFlags as Flags, Translate,
        mapper::TranslateResult,
    },
};
impl UserMappings {
    /// # Safety
    /// Inactive unique root, live boot mappings, IRQs off. Caller retains every allocated
    /// frame and reclaims them all if construction fails. Image range is page aligned.
    /// Complete prepare_kernel before publishing or activating this root.
    pub unsafe fn create(
        layout: &BootLayout,
        image: ImageRange,
        frames: &mut impl FrameSource,
    ) -> Result<Self, MemoryError> {
        let mut source = Tables(frames);
        let root = source
            .allocate_frame()
            .ok_or(MemoryError::OutOfFrames)?
            .start_address()
            .as_u64();
        Ok(Self {
            root,
            kernel: Kernel {
                boot: layout.root_address(),
                ranges: [
                    (image.base, image.base + image.bytes),
                    (layout.stack_bottom, layout.stack_top),
                    (
                        layout.heap_start,
                        layout.heap_start + layout.heap_bytes as u64,
                    ),
                    (
                        layout.emergency_tops[0] - 4 * PAGE,
                        layout.emergency_tops[0],
                    ),
                    (
                        layout.emergency_tops[1] - 4 * PAGE,
                        layout.emergency_tops[1],
                    ),
                    (
                        layout.emergency_tops[2] - 4 * PAGE,
                        layout.emergency_tops[2],
                    ),
                    (
                        layout.emergency_tops[3] - 4 * PAGE,
                        layout.emergency_tops[3],
                    ),
                ],
                range: 0,
                offset: 0,
            },
        })
    }

    /// Install at most budget supervisor leaves, preserving their boot permissions.
    /// # Safety
    /// Private inactive root, sole CPU/kernel root/IRQs off. Boot mappings and
    /// their backing remain permanent and unchanged across calls. Caller tracks
    /// every table frame and retires the entire root on error; do not retry it.
    pub unsafe fn prepare_kernel(
        &mut self,
        frames: &mut impl FrameSource,
        budget: usize,
    ) -> Result<bool, MemoryError> {
        assert!(budget != 0);
        self.assert_inactive();
        let mut source = Tables(frames);
        // SAFETY: Distinct live boot and private target roots; borrows end before yielding.
        unsafe {
            let mut target =
                OffsetPageTable::new(&mut *(self.root as *mut PageTable), VirtAddr::zero());
            let boot =
                OffsetPageTable::new(&mut *(self.kernel.boot as *mut PageTable), VirtAddr::zero());
            for _ in 0..budget {
                let Some(address) = self.kernel.next() else {
                    return Ok(true);
                };
                let TranslateResult::Mapped {
                    frame,
                    offset,
                    flags,
                } = boot.translate(VirtAddr::new(address))
                else {
                    return Err(MemoryError::MapFailed);
                };
                let physical = frame.start_address().as_u64() + offset;
                // The same leaf as the kernel root's, global like it (memory::shared):
                // its translation is right under either root and outlives the switch.
                let flags = (flags & (Flags::PRESENT | Flags::WRITABLE | Flags::NO_EXECUTE))
                    | Flags::GLOBAL;
                map(&mut target, address, physical, flags, &mut source)?;
            }
        }
        Ok(false)
    }
}

// Owned values only: no mapper, mutable table borrow or boot-stack reference
// survives suspension. Core retains the root and its frame ledger on failure.
pub(super) struct Kernel {
    boot: u64,
    ranges: [(u64, u64); 7],
    range: usize,
    offset: u64,
}
impl Kernel {
    pub(super) fn done(&self) -> bool {
        self.range == self.ranges.len()
    }
    fn next(&mut self) -> Option<u64> {
        while let Some(&(bottom, top)) = self.ranges.get(self.range) {
            if self.offset < top - bottom {
                let address = bottom + self.offset;
                self.offset += PAGE;
                return Some(address);
            }
            self.range += 1;
            self.offset = 0;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn kernel_cursor_preserves_gaps_and_high_ranges_across_batches() {
        let high = 0xffff_8000_0000_0000;
        for budget in [1, 2, 8, 64] {
            let mut cursor = Kernel {
                boot: 0,
                ranges: [
                    (PAGE, 3 * PAGE),
                    (high, high + 2 * PAGE),
                    (0, 0),
                    (8 * PAGE, 9 * PAGE),
                    (high + 8 * PAGE, high + 9 * PAGE),
                    (0, 0),
                    (0, 0),
                ],
                range: 0,
                offset: 0,
            };
            let expected = [PAGE, 2 * PAGE, high, high + PAGE, 8 * PAGE, high + 8 * PAGE];
            let mut visited = 0;
            while !cursor.done() {
                for _ in 0..budget {
                    if let Some(address) = cursor.next() {
                        assert_eq!(address, expected[visited]);
                        visited += 1;
                    }
                }
            }
            assert_eq!(visited, expected.len());
            assert_eq!(cursor.next(), None);
        }
    }
}
