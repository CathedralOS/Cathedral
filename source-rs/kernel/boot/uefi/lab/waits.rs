//! Shared completion burst with exact allocator recovery around the core fixture.
use crate::{diagnostics::user_output, heap::HEAP, memory::PreparedMemory};

pub(super) fn exercise(memory: &mut PreparedMemory) {
    let baseline = memory.frames.allocated();
    // SAFETY: Sole boot CPU outside a userspace session owns both allocators.
    unsafe {
        let heap = HEAP.used();
        cathedral_core::user_tasks::exercise_wait_burst(
            &mut memory.frames,
            &memory.layout,
            memory.image,
            user_output,
        );
        assert_eq!(HEAP.used(), heap);
    }
    assert_eq!(memory.frames.allocated(), baseline);
}
