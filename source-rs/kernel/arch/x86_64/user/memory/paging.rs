//! Frame ledger and inactive-table mapping shared by admission, page regions and devices.
use super::{MAX_FRAMES, MemoryError, PAGE, StackFrames};
use ::x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags as Flags, PhysFrame, Size4KiB,
    },
};

pub(super) struct Tracked<'a, F> {
    pub(super) frames: &'a mut F,
    pub(super) addresses: &'a mut [u64; MAX_FRAMES],
    pub(super) len: &'a mut usize,
}
// SAFETY: StackFrames supplies unique unused physical frames. Ledger retains all
// backing and intermediate tables, including those from partial failed mappings.
unsafe impl<F: StackFrames> FrameAllocator<Size4KiB> for Tracked<'_, F> {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        if *self.len == MAX_FRAMES {
            return None;
        }
        let address = self.frames.allocate()?;
        assert!(address != 0 && address < (1 << 48) && address.is_multiple_of(PAGE));
        self.addresses[*self.len] = address;
        *self.len += 1;
        // SAFETY: Fresh physical frame is writable through kernel identity mapping.
        unsafe {
            core::ptr::write_bytes(address as *mut u8, 0, PAGE as usize);
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
    // SAFETY: Caller owns the inactive hierarchy and provides valid backing and permissions.
    unsafe {
        mapper.map_to(
            Page::from_start_address(VirtAddr::new(address)).unwrap(),
            PhysFrame::<Size4KiB>::from_start_address(PhysAddr::new(physical)).unwrap(),
            flags,
            frames,
        )
    }
    .map_err(|_| MemoryError::OutOfFrames)?
    .ignore();
    Ok(())
}
