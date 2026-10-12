//! Remove surrendered aliases before refunding unreferenced physical backing.
use super::{Frames, Memory, Session, wire};
use cathedral_arch::FrameSource;

impl Memory {
    pub(super) unsafe fn reclaim(&mut self, index: usize, frames: &mut Frames<'_>) {
        if !self.model.entries[index].live() {
            let mut returned = 0;
            for frame in &mut self.backing[index] {
                if *frame != 0 {
                    // SAFETY: All mappings were removed or never published; core holds unique backing custody.
                    unsafe {
                        frames.release(*frame);
                    }
                    *frame = 0;
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
    let mapping = session.memory.model.mapping(slot, index);
    session.memory.model.release(slot, handle)?;
    let pages = session.memory.model.entries[index].pages;
    // SAFETY: Calling task surrendered this mapping; no other mapping is withdrawn.
    unsafe {
        session.tasks[slot]
            .space
            .as_mut()
            .unwrap()
            .unmap_region(mapping, pages);
        session.memory.reclaim(index, frames);
    }
    session.tasks[slot].report.memory.unmapped_pages += pages;
    Ok(0)
}

/// # Safety
/// Retired task cannot resume; boot stack/kernel root/IRQs off. Remove its aliases before freeing backing.
pub(in crate::user_tasks) unsafe fn close(
    session: &mut Session,
    slot: usize,
    frames: &mut Frames<'_>,
) {
    for index in 0..wire::MAX_REGIONS {
        let region = session.memory.model.entries[index];
        if region.owner == Some(slot) || (region.peer == Some(slot) && region.accepted) {
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
