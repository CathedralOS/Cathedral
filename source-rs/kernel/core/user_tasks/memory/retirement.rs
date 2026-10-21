//! Remove surrendered aliases before refunding unreferenced physical backing.
use super::{Frames, Memory, Session};
use cathedral_arch::FrameSource;

pub(super) const FRAME_BATCH: usize = 8;
impl Memory {
    pub(super) unsafe fn reclaim_step(&mut self, index: usize, frames: &mut Frames<'_>) -> bool {
        let region = &mut self.model.entries[index];
        if region.live() {
            assert!(!region.retiring);
            return true;
        }
        region.retiring = true;
        let mut returned = 0;
        for _ in 0..FRAME_BATCH {
            let Some(frame) = self.backing[index].pop() else {
                break;
            };
            assert_ne!(frame, 0, "unbacked reservation reached physical retirement");
            // SAFETY: Every alias/reference is gone; this ledger retains unique custody.
            unsafe { frames.release(frame) };
            returned += 1;
        }
        if returned != 0 {
            self.funding.refund(region.account.unwrap(), returned);
        }
        let done = self.backing[index].is_empty();
        if done {
            self.backing[index] = alloc::vec::Vec::new();
            region.retiring = false;
        }
        done
    }
}

pub(super) unsafe fn step(
    session: &mut Session,
    frames: &mut Frames<'_>,
    slot: usize,
    handle: u64,
) -> Option<Result<u64, u64>> {
    if session.memory.releases[slot].is_none() {
        // SAFETY: Inactive caller root; detachment changes only its own authority/alias.
        match unsafe { detach(session, slot, handle) } {
            Ok(index) => session.memory.releases[slot] = Some(index),
            Err(error) => return Some(Err(error)),
        }
    }
    let index = session.memory.releases[slot].unwrap();
    // SAFETY: Detachment removed this caller's alias. Remaining authorities pin
    // backing; otherwise the cursor owns its retiring slot across suspension.
    if unsafe { session.memory.reclaim_step(index, frames) } {
        session.memory.releases[slot] = None;
        Some(Ok(0))
    } else {
        None
    }
}

#[cfg(feature = "memory-lab")]
pub(super) unsafe fn release(
    session: &mut Session,
    frames: &mut Frames<'_>,
    slot: usize,
    handle: u64,
) -> Result<u64, u64> {
    loop {
        let before = frames.allocated();
        // SAFETY: Synchronous fixture retains the same inactive-root custody.
        let result = unsafe { step(session, frames, slot, handle) };
        assert!(before - frames.allocated() <= FRAME_BATCH);
        if let Some(result) = result {
            return result;
        }
    }
}

unsafe fn detach(session: &mut Session, slot: usize, handle: u64) -> Result<usize, u64> {
    let index = session.memory.model.access(slot, handle)?;
    let mapping = session.memory.model.mapped(slot, handle)?;
    let read = session.memory.model.read_slot(slot, handle);
    if read.is_none() {
        session.memory.model.release(slot, handle)?;
    }
    let pages = session.memory.model.entries[index].pages;
    if let Some(mapping) = mapping {
        // SAFETY: Caller surrendered only its own mapping; root inactive.
        unsafe {
            session.tasks[slot]
                .space
                .as_mut()
                .unwrap()
                .unmap_region(mapping, pages);
        }
        session.tasks[slot].report.memory.unmapped_pages += pages;
    }
    if let Some(read) = read {
        session.memory.model.reads[read].as_mut().unwrap().mapping = None;
        session.memory.model.drop_read(read);
    }
    Ok(index)
}
