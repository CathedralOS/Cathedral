//! Dead-task aliases are removed before their references and backing are released.
//! Each step scans at most SCAN_BATCH entries and cleans at most one page object.
use super::{Frames, Session};
const SCAN_BATCH: usize = 32;

#[derive(Default)]
pub(in crate::user_tasks) struct Closing {
    started: bool,
    read: usize,
    region: usize,
    retiring: Option<usize>,
}

/// # Safety
/// Dead task, never resumed; boot stack/kernel root/IRQs off. Its slot cannot be
/// reused across steps, nor may new endpoints or memory grants target it.
pub(in crate::user_tasks) unsafe fn step(
    session: &mut Session,
    slot: usize,
    frames: &mut Frames<'_>,
    cursor: &mut Closing,
) -> bool {
    if let Some(index) = cursor.retiring {
        // SAFETY: The dead task's last alias is gone; cursor retains the retiring slot.
        if unsafe { session.memory.reclaim_step(index, frames) } {
            cursor.retiring = None;
        }
        return false;
    }
    if session.memory.releases[slot].is_some() {
        // SAFETY: Death inherits an explicit release whose alias was already removed.
        let _ = unsafe { super::retirement::step(session, frames, slot, 0) };
        return false;
    }
    if !cursor.started {
        // SAFETY: An unpublished allocation has no user aliases.
        cursor.started = unsafe { super::allocation::cancel_step(session, frames, slot) };
        return false;
    }
    for _ in 0..SCAN_BATCH {
        if cursor.read < session.memory.model.reads.len() {
            let read = cursor.read;
            cursor.read += 1;
            let Some(grant) = session.memory.model.reads[read].filter(|r| r.task == slot) else {
                continue;
            };
            // The escrow snapshot owns this unpublished slot until it drops the
            // reference. Do not free/reuse it under a pending discard bit.
            if session.memory.escrow_slots[read / 64] & (1 << (read % 64)) != 0
                || session.ipc.memory_retiring(read)
            {
                assert!(!grant.published && grant.mapping.is_none());
                continue;
            }
            if let Some(mapping) = grant.mapping {
                let pages = session.memory.model.entries[grant.object].pages;
                // SAFETY: Retired task cannot resume; no other process mapping changes.
                unsafe {
                    session.tasks[slot]
                        .space
                        .as_mut()
                        .unwrap()
                        .unmap_region(mapping, pages);
                }
                session.tasks[slot].report.memory.unmapped_pages += pages;
                session.memory.model.reads[read].as_mut().unwrap().mapping = None;
            }
            session.memory.model.drop_read(read);

            // SAFETY: The last dead reader's alias was removed before dropping its ref.
            if !unsafe { session.memory.reclaim_step(grant.object, frames) } {
                cursor.retiring = Some(grant.object);
            }
            return false;
        }
        if cursor.region < session.memory.model.entries.len() {
            let index = cursor.region;
            cursor.region += 1;
            let region = session.memory.model.entries[index];
            if region.owner != Some(slot) && region.peer != Some(slot) {
                continue;
            }
            if (region.owner == Some(slot) && region.owner_mapped)
                || (region.peer == Some(slot) && region.accepted)
            {
                // SAFETY: Only the dead task's mappings disappear. Accepted live readers keep their physical lease.
                unsafe {
                    session.tasks[slot]
                        .space
                        .as_mut()
                        .unwrap()
                        .unmap_region(session.memory.model.mapping(slot, index), region.pages);
                }
                session.tasks[slot].report.memory.unmapped_pages += region.pages;
            }
            session.memory.model.close(index, slot);
            // SAFETY: Model has no remaining live alias when reclamation proceeds.
            if !unsafe { session.memory.reclaim_step(index, frames) } {
                cursor.retiring = Some(index);
            }

            return false;
        }
        return true;
    }
    false
}

#[cfg(feature = "memory-lab")]
pub(super) unsafe fn close(session: &mut Session, slot: usize, frames: &mut Frames<'_>) {
    let mut cursor = Closing::default();
    loop {
        let before = frames.allocated();
        // SAFETY: Synchronous test wrapper with the same inactive-root custody.
        let done = unsafe { step(session, slot, frames, &mut cursor) };
        assert!(before - frames.allocated() <= super::retirement::FRAME_BATCH);
        if done {
            break;
        }
    }
}
