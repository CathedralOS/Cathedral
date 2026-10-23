//! Supervisor ranges every root maps identically are global: the kernel image,
//! the boot and emergency stacks and the context heap. A user root carries the
//! same leaves (user::memory::creation), so a translation of one of them is
//! right under any root and may outlive the two root switches of a trap. Every
//! other kernel-root leaf, the firmware's aliases of image frames included,
//! stays per root and is retired by each root switch.
use super::{MemoryError, PAGE_SIZE};
use crate::memory::KernelImage;
use ::x86_64::{
    VirtAddr,
    structures::paging::{
        Mapper, OffsetPageTable, Page, PageTableFlags as Flags, Size4KiB, Translate,
        mapper::{MappedFrame, TranslateResult},
    },
};

/// Mark the image's identity-mapped leaves global in an inactive kernel root.
/// # Safety
/// `mapper` covers an inactive root the caller owns exclusively, whose image
/// pages inherited::copy split to 4-KiB leaves.
pub(super) unsafe fn share_image(
    mapper: &mut OffsetPageTable<'_>,
    image: &KernelImage<'_>,
) -> Result<(), MemoryError> {
    let range = image.range;
    for address in (range.base..range.base + range.bytes).step_by(PAGE_SIZE as usize) {
        let TranslateResult::Mapped {
            frame: MappedFrame::Size4KiB(_),
            flags,
            ..
        } = mapper.translate(VirtAddr::new(address))
        else {
            return Err(MemoryError::MapFailed);
        };
        let page = Page::<Size4KiB>::from_start_address(VirtAddr::new(address)).unwrap();
        // SAFETY: The caller's inactive root; the leaf keeps its frame and permissions.
        unsafe { mapper.update_flags(page, flags | Flags::GLOBAL) }
            .map_err(|_| MemoryError::MapFailed)?
            .ignore();
    }
    Ok(())
}
