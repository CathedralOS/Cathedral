//! Adapt physical-frame custody to x86 page tables. Ownership remains with core.
use crate::{FrameSource, MemoryError};
use ::x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags as Flags, PhysFrame, Size4KiB,
    },
};
pub(super) struct Tables<'a, F>(pub &'a mut F);
// SAFETY: FrameSource supplies exclusive valid frames; all allocations remain in the caller's ledger.
unsafe impl<F: FrameSource> FrameAllocator<Size4KiB> for Tables<'_, F> {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let address = self.0.allocate()?;
        assert!(address != 0 && address < (1 << 48) && address.is_multiple_of(4096));
        // SAFETY: Caller-provided unused physical RAM; initialize a new table before linking it.
        unsafe {
            core::ptr::write_bytes(address as *mut u8, 0, 4096);
        }
        Some(PhysFrame::from_start_address(PhysAddr::new(address)).unwrap())
    }
}
pub(super) unsafe fn map(
    mapper: &mut OffsetPageTable<'_>,
    address: u64,
    physical: u64,
    flags: Flags,
    frames: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MemoryError> {
    // SAFETY: Caller owns inactive tables, valid disjoint backing and validated permissions.
    unsafe {
        mapper.map_to(
            Page::from_start_address(VirtAddr::new(address)).unwrap(),
            PhysFrame::<Size4KiB>::from_start_address(PhysAddr::new(physical)).unwrap(),
            flags,
            frames,
        )
    }
    .map_err(|error| match error {
        ::x86_64::structures::paging::mapper::MapToError::FrameAllocationFailed => {
            MemoryError::OutOfFrames
        }
        _ => MemoryError::OccupiedVirtualRange,
    })?
    .ignore();
    Ok(())
}
