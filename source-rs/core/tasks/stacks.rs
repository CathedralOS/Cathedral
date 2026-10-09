//! Bridge core frame custody to architecture mapping operations.

use super::SpawnError;
use crate::extent::FrameAllocator;
use cathedral_arch::{self as arch, StackFrames};

struct Source<'a> {
    frames: &'a mut FrameAllocator,
    remaining: usize,
}
// SAFETY: Constructed only by the unsafe mapping APIs below, whose callers own
// the physical inventory and serialize all users and mapping changes.
unsafe impl StackFrames for Source<'_> {
    fn allocate(&mut self) -> Option<u64> {
        if self.remaining == 0 {
            return None;
        }
        let frame = self.frames.allocate()?;
        self.remaining -= 1;
        Some(frame.address())
    }
    unsafe fn release(&mut self, address: u64) {
        self.frames
            .release_address(address)
            .expect("invalid stack frame release");
    }
}

pub(super) unsafe fn allocate(
    frames: &mut FrameAllocator,
    slot: usize,
    budget: usize,
) -> Result<arch::StackRange, SpawnError> {
    // SAFETY: Runtime has exclusive single-CPU ownership with IRQs masked.
    unsafe {
        arch::allocate_stack(
            slot,
            &mut Source {
                frames,
                remaining: budget,
            },
        )
    }
    .map_err(|error| match error {
        arch::MemoryError::OutOfFrames => SpawnError::OutOfFrames,
        _ => panic!("task stack arena corrupted: {error:?}"),
    })
}

pub(super) unsafe fn release(frames: &mut FrameAllocator, slot: usize) {
    // SAFETY: Runtime switched away and removed the task's saved context.
    unsafe {
        arch::release_stack(
            slot,
            &mut Source {
                frames,
                remaining: 0,
            },
        );
    }
}
