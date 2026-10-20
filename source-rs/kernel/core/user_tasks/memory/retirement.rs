//! Remove surrendered aliases before refunding unreferenced physical backing.
use super::{Frames, Memory, Session, wire};
use cathedral_arch::FrameSource;

impl Memory {
    pub(super) unsafe fn reclaim(&mut self, index: usize, frames: &mut Frames<'_>) {
        if !self.model.entries[index].live() {
            let mut returned = 0;
            for frame in core::mem::take(&mut self.backing[index]) {
                if frame != 0 {
                    // SAFETY: All mappings were removed or never published; core holds unique backing custody.
                    unsafe {
                        frames.release(frame);
                    }
                    returned += 1;
                }
            }
            if returned != 0 {
                self.funding
                    .refund(self.model.entries[index].account.unwrap(), returned);
            }
        }
    }
}

pub(super) unsafe fn release(
    session: &mut Session,
    frames: &mut Frames<'_>,
    slot: usize,
    handle: u64,
) -> Result<u64, u64> {
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
    // SAFETY: Removed this alias before final-reference reclamation.
    unsafe {
        session.memory.reclaim(index, frames);
    }
    Ok(0)
}

/// # Safety
/// Retired task cannot resume; boot stack/kernel root/IRQs off. Remove its aliases before freeing backing.
pub(in crate::user_tasks) unsafe fn close(
    session: &mut Session,
    slot: usize,
    frames: &mut Frames<'_>,
) {
    // SAFETY: Dead task cannot resume; unfinished allocation has no published aliases.
    unsafe {
        super::allocation::cancel(session, frames, slot);
    }
    for read in 0..wire::MAX_READ_HANDLES {
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
    }
    for index in 0..wire::MAX_REGIONS {
        let region = session.memory.model.entries[index];
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
    }
}
