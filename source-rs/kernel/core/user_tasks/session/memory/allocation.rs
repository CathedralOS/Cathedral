//! All-or-nothing allocation: reserve budget, zero backing, map, or roll back every frame.
use super::*;
pub(super) unsafe fn allocate(
    session: &mut Session,
    frames: &mut Frames<'_>,
    slot: usize,
    pages: usize,
) -> Result<u64, u64> {
    let index = session
        .memory
        .model
        .allocate(slot, pages, next_epoch().ok_or(abi::NO_MEMORY)?)?;
    let result = (|| {
        for physical in &mut session.memory.backing[index][..pages] {
            *physical = frames.allocate().ok_or(abi::NO_MEMORY)?;
            // SAFETY: Fresh exclusive physical frame, writable in the kernel identity map; not yet exposed.
            unsafe {
                core::ptr::write_bytes(*physical as *mut u8, 0, wire::PAGE_BYTES);
            }
        }
        // SAFETY: Fresh zeroed frames; sole producer's inactive root has preallocated tables.
        unsafe {
            session.tasks[slot].space.as_mut().unwrap().map_region(
                index,
                &session.memory.backing[index][..pages],
                true,
            )
        }
        .map_err(|_| abi::NO_MEMORY)?;
        session.tasks[slot].report.memory.allocated_pages += pages;
        session.tasks[slot].report.memory.mapped_pages += pages;
        Ok(session.memory.model.entries[index].handle(index))
    })();
    if result.is_err() {
        session.memory.model.entries[index].owner = None;
        // SAFETY: map_region publishes the complete set or rolls back all leaves; no borrowed mappings exist.
        unsafe {
            session.memory.reclaim(index, frames);
        }
    }
    result
}
