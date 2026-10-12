//! Borrow only the kernel translations needed for entry, dispatch and return.
use super::paging::{Tables, map};
use super::{PAGE, UserMappings};
use crate::{BootLayout, FrameSource, ImageRange, MemoryError};
use ::x86_64::{
    VirtAddr,
    structures::paging::{
        FrameAllocator, OffsetPageTable, PageTable, PageTableFlags as Flags, Size4KiB, Translate,
        mapper::TranslateResult,
    },
};
impl UserMappings {
    /// # Safety
    /// Inactive unique root, live boot mappings, IRQs off. Caller retains every allocated
    /// frame and reclaims them all if construction fails. Image range is page aligned.
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
        // SAFETY: Fresh zeroed root, disjoint from the live boot tree; caller tracks allocation.
        unsafe {
            let mut target = OffsetPageTable::new(&mut *(root as *mut PageTable), VirtAddr::zero());
            let boot = OffsetPageTable::new(
                &mut *(layout.root_address() as *mut PageTable),
                VirtAddr::zero(),
            );
            map_kernel(
                &mut target,
                &boot,
                layout,
                image.base,
                image.base + image.bytes,
                &mut source,
            )?;
        }
        Ok(Self { root })
    }
}
unsafe fn map_kernel(
    target: &mut OffsetPageTable<'_>,
    boot: &OffsetPageTable<'_>,
    layout: &BootLayout,
    start: u64,
    end: u64,
    source: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MemoryError> {
    let ranges = [
        (start, end),
        (layout.stack_bottom, layout.stack_top),
        (
            layout.heap_start,
            layout.heap_start + layout.heap_bytes as u64,
        ),
    ];
    for (bottom, top) in ranges.into_iter().chain(
        layout
            .emergency_tops
            .iter()
            .map(|top| (top - 4 * PAGE, *top)),
    ) {
        for address in (bottom..top).step_by(PAGE as usize) {
            let TranslateResult::Mapped {
                frame,
                offset,
                flags,
            } = boot.translate(VirtAddr::new(address))
            else {
                return Err(MemoryError::MapFailed);
            };
            let physical = frame.start_address().as_u64() + offset;
            let flags = flags & (Flags::PRESENT | Flags::WRITABLE | Flags::NO_EXECUTE);
            // SAFETY: Explicit live kernel resource; only supervisor leaf permissions.
            unsafe {
                map(target, address, physical, flags, source)?;
            }
        }
    }
    Ok(())
}
