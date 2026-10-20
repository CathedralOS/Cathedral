//! Dead-task aliases are removed before their references and backing are released.
//! Each step scans at most SCAN_BATCH entries and cleans at most one page object.
use super::{Frames, Session};
const SCAN_BATCH: usize = 32;

#[derive(Default)]
pub(in crate::user_tasks) struct Closing {
    started: bool,
    read: usize,
    region: usize,
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
    if !cursor.started {
        // SAFETY: An unpublished allocation has no user aliases.
        unsafe {
            super::allocation::cancel(session, frames, slot);
        }
        cursor.started = true;
        return false;
    }
    for _ in 0..SCAN_BATCH {
        if cursor.read < session.memory.model.reads.len() {
            let read = cursor.read;
            cursor.read += 1;
            let Some(grant) = session.memory.model.reads[read].filter(|r| r.task == slot) else {
                continue;
            };
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
            unsafe {
                session.memory.reclaim(grant.object, frames);
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
            unsafe {
                session.memory.reclaim(index, frames);
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
    // SAFETY: Synchronous test wrapper with the same inactive-root custody.
    while !unsafe { step(session, slot, frames, &mut cursor) } {}
}
