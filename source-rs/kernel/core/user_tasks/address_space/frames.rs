//! Capture every allocation, including partial table construction; retirement refunds in reverse order.
use super::MAX_FRAMES;
use cathedral_arch::FrameSource;
pub(super) struct Tracked<'a, F> {
    pub frames: &'a mut F,
    pub addresses: &'a mut alloc::vec::Vec<u64>,
    pub len: &'a mut usize,
}
// SAFETY: The underlying source supplies exclusive physical custody. Every allocation
// remains in this ledger; callers serialize it and release only after retiring the root.
unsafe impl<F: FrameSource> FrameSource for Tracked<'_, F> {
    fn allocate(&mut self) -> Option<u64> {
        if *self.len == MAX_FRAMES {
            return None;
        }
        self.addresses.try_reserve(1).ok()?;
        let address = self.frames.allocate()?;
        self.addresses.push(address);
        *self.len += 1;
        Some(address)
    }
    unsafe fn release(&mut self, _address: u64) {
        unreachable!("mapping backend cannot refund owned frames");
    }
}
