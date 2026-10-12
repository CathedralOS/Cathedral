//! Reserve real RAM before userspace; lend frames without touching global capacity.
//! The bank is exclusive to this session and returns all frames on teardown/failure.
use crate::{extent::FrameAllocator, user_tasks::Error};
use alloc::vec::Vec;
use cathedral_arch::{MemoryError, StackFrames};

pub(super) struct Frames<'a> {
    frames: &'a mut FrameAllocator,
    free: Vec<u64>,
    capacity: usize,
    pub remaining: usize,
}
impl<'a> Frames<'a> {
    pub fn new(
        frames: &'a mut FrameAllocator,
        capacity: usize,
        remaining: usize,
    ) -> Result<Self, Error> {
        let mut free = Vec::new();
        free.try_reserve_exact(capacity)
            .map_err(|_| Error::OutOfHeap)?;
        let mut bank = Self {
            frames,
            free,
            capacity: 0,
            remaining,
        };
        for _ in 0..capacity {
            let frame = bank
                .frames
                .allocate()
                .ok_or(Error::Memory(MemoryError::OutOfFrames))?;
            bank.free.push(frame.address());
            bank.capacity += 1;
        }
        Ok(bank)
    }
    /// Live mappings/backing plus the pre-session baseline, excluding idle reserves.
    pub fn allocated(&self) -> usize {
        self.frames.allocated() - self.free.len()
    }
}
// SAFETY: Sole boot CPU and kernel identity map; every frame was reserved exclusively.
unsafe impl StackFrames for Frames<'_> {
    fn allocate(&mut self) -> Option<u64> {
        if self.remaining == 0 {
            return None;
        }
        let frame = self.free.pop()?;
        self.remaining -= 1;
        Some(frame)
    }
    unsafe fn release(&mut self, address: u64) {
        assert!(self.free.len() < self.capacity);
        self.free.push(address);
    }
}
impl Drop for Frames<'_> {
    fn drop(&mut self) {
        assert_eq!(
            self.free.len(),
            self.capacity,
            "live user frames at bank teardown"
        );
        for address in self.free.drain(..) {
            self.frames
                .release_address(address)
                .expect("invalid bank frame");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cathedral_contracts::boot::{BootMemory, MemoryKind, MemoryRegion};
    fn allocator() -> FrameAllocator {
        let mut memory = BootMemory::new();
        memory
            .push(MemoryRegion {
                base: 4096,
                page_count: 4,
                kind: MemoryKind::Usable,
            })
            .unwrap();
        let mut frames = FrameAllocator::new(memory);
        frames.allocate().unwrap(); // permanent boot frame
        frames.enable_reclamation();
        frames
    }
    #[test]
    fn bank_reserves_real_capacity_recycles_and_returns_it() {
        let mut allocator = allocator();
        {
            let mut bank = Frames::new(&mut allocator, 3, usize::MAX).unwrap();
            assert_eq!(bank.frames.allocated(), 4);
            assert!(bank.frames.allocate().is_none());
            let first = bank.allocate().unwrap();
            let second = bank.allocate().unwrap();
            let third = bank.allocate().unwrap();
            assert!(bank.allocate().is_none());
            assert_eq!(bank.allocated(), 4);
            // SAFETY: Host fixture never maps or dereferences these frames.
            unsafe {
                bank.release(first);
            }
            assert_eq!(bank.allocate(), Some(first));
            // SAFETY: All three distinct frames remain exclusively owned here.
            unsafe {
                for frame in [first, second, third] {
                    bank.release(frame);
                }
            }
            assert_eq!(bank.allocated(), 1);
        }
        assert_eq!(allocator.allocated(), 1);
    }
    #[test]
    fn incomplete_physical_reservation_rolls_back_before_any_user_can_run() {
        let mut allocator = allocator();
        assert!(matches!(
            Frames::new(&mut allocator, 4, usize::MAX),
            Err(Error::Memory(MemoryError::OutOfFrames))
        ));
        assert_eq!(allocator.allocated(), 1);
    }
}
