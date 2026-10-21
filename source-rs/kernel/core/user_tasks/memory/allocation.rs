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
    cancelling: bool,
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
    if session.memory.allocations[slot].is_some_and(|staging| staging.cancelling) {
        // SAFETY: A previous refusal retained the unpublished allocation for cleanup.
        return unsafe { cancel_step(session, frames, slot) }.then_some(Err(abi::NO_MEMORY));
    }
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
            return unsafe { cancel_step(session, frames, slot) }.then_some(Err(abi::NO_MEMORY));
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
        return unsafe { cancel_step(session, frames, slot) }.then_some(Err(abi::NO_MEMORY));
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
        cancelling: false,
    });
    Ok(())
}

pub(super) unsafe fn cancel_step(
    session: &mut Session,
    frames: &mut Frames<'_>,
    slot: usize,
) -> bool {
    let Some(mut staging) = session.memory.allocations[slot] else {
        return true;
    };
    if !staging.cancelling {
        session.memory.funding.refund(slot, staging.reserved());
        session.memory.model.entries[staging.region].owner = None;
        session.memory.backing[staging.region].truncate(staging.initialized);
        staging.pages = staging.initialized;
        staging.cancelling = true;
        session.memory.allocations[slot] = Some(staging);
    }
    // SAFETY: Staging never publishes an alias. Refund allocated pages exactly once.
    let done = unsafe { session.memory.reclaim_step(staging.region, frames) };
    if done {
        session.memory.allocations[slot] = None;
    }
    done
}

#[cfg(feature = "memory-lab")]
pub(super) unsafe fn allocate(
    session: &mut Session,
    frames: &mut Frames<'_>,
    slot: usize,
    pages: usize,
) -> Result<u64, u64> {
    loop {
        let before = frames.allocated();
        // SAFETY: Fixture owns an inactive root and frame custody for every step.
        let result = unsafe { step(session, frames, slot, pages) };
        assert!(before.saturating_sub(frames.allocated()) <= super::retirement::FRAME_BATCH);
        if let Some(result) = result {
            return result;
        }
    }
}
