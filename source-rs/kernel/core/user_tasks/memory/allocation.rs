//! Unpublished allocation: reserve once, zero a page batch, then publish the full
//! mapping. The caller stays blocked; cancellation refunds both backing and the
//! still-unbacked reservation before the task account can be reused.
use super::*;
use cathedral_arch::FrameSource;

pub(super) const PAGE_BATCH: usize = 8;
#[derive(Clone, Copy)]
pub(super) struct Staging {
    region: usize,
    initialized: usize,
    pages: usize,
}
impl Staging {
    pub(super) fn reserved(&self) -> usize {
        self.pages - self.initialized
    }
}

pub(super) unsafe fn step(
    session: &mut Session,
    frames: &mut Frames<'_>,
    slot: usize,
    pages: usize,
) -> Option<Result<u64, u64>> {
    if session.memory.allocations[slot].is_none()
        && let Err(error) = begin(session, slot, pages)
    {
        return Some(Err(error));
    }
    let mut staging = session.memory.allocations[slot].unwrap();
    let index = staging.region;
    for _ in 0..PAGE_BATCH.min(staging.reserved()) {
        let Some(physical) = frames.allocate() else {
            // SAFETY: No mapping or handle has been published.
            unsafe {
                cancel(session, frames, slot);
            }
            return Some(Err(abi::NO_MEMORY));
        };
        // SAFETY: Fresh exclusive frame in the kernel identity map; not exposed.
        unsafe {
            core::ptr::write_bytes(physical as *mut u8, 0, wire::PAGE_BYTES);
        }
        session.memory.backing[index][staging.initialized] = physical;
        staging.initialized += 1;
        session.memory.allocations[slot] = Some(staging);
    }
    if staging.reserved() != 0 {
        return None;
    }
    // SAFETY: All pages are zeroed; inactive producer root has preallocated tables.
    // Mapping still commits atomically (at most MAX_PAGES leaf edits).
    let result = unsafe {
        session.tasks[slot].space.as_mut().unwrap().map_region(
            session.memory.model.mapping(slot, index),
            &session.memory.backing[index][..pages],
            true,
        )
    };
    if result.is_err() {
        // SAFETY: map_region rolls back every leaf on failure.
        unsafe {
            cancel(session, frames, slot);
        }
        return Some(Err(abi::NO_MEMORY));
    }
    session.memory.model.entries[index].owner_mapped = true;
    session.memory.allocations[slot] = None;
    session.tasks[slot].report.memory.allocated_pages += pages;
    session.tasks[slot].report.memory.mapped_pages += pages;
    Some(Ok(session.memory.model.handle(slot, index)))
}

fn begin(session: &mut Session, slot: usize, pages: usize) -> Result<(), u64> {
    let index = session.memory.model.allocate(slot, pages)?;
    session.memory.model.entries[index].owner_mapped = false;
    if let Err(error) = session.memory.funding.charge(slot, pages) {
        session.memory.model.entries[index].owner = None;
        return Err(error);
    }
    if session.memory.backing[index]
        .try_reserve_exact(pages)
        .is_err()
    {
        session.memory.funding.refund(slot, pages);
        session.memory.model.entries[index].owner = None;
        return Err(abi::NO_MEMORY);
    }
    session.memory.backing[index].resize(pages, 0);
    session.memory.allocations[slot] = Some(Staging {
        region: index,
        initialized: 0,
        pages,
    });
    Ok(())
}

pub(super) unsafe fn cancel(session: &mut Session, frames: &mut Frames<'_>, slot: usize) {
    let Some(staging) = session.memory.allocations[slot].take() else {
        return;
    };
    session.memory.funding.refund(slot, staging.reserved());
    session.memory.model.entries[staging.region].owner = None;
    // SAFETY: Staging never publishes an alias. Refund allocated pages exactly once.
    unsafe {
        session.memory.reclaim(staging.region, frames);
    }
}

#[cfg(feature = "memory-lab")]
pub(super) unsafe fn allocate(
    session: &mut Session,
    frames: &mut Frames<'_>,
    slot: usize,
    pages: usize,
) -> Result<u64, u64> {
    loop {
        // SAFETY: Fixture owns an inactive root and frame custody for every step.
        if let Some(result) = unsafe { step(session, frames, slot, pages) } {
            return result;
        }
    }
}
