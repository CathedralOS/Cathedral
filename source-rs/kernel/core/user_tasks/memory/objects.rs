//! Deferred mapping edits for sealed objects. A read capability can exist and
//! travel without a virtual mapping; removing a mapping never withdraws peers.
use super::*;
pub(super) unsafe fn freeze(session: &mut Session, task: usize, name: u64) -> Result<u64, u64> {
    let index = session.memory.model.freeze(task, name)?;
    let pages = session.memory.model.entries[index].pages;
    // SAFETY: Sole writable producer root is inactive; no reader can exist before sealing.
    unsafe {
        session.tasks[task]
            .space
            .as_mut()
            .unwrap()
            .seal_region(session.memory.model.mapping(task, index), pages);
    }
    session.tasks[task].report.memory.sealed_pages += pages;
    Ok(0)
}
pub(super) unsafe fn map(session: &mut Session, task: usize, name: u64) -> Result<u64, u64> {
    let object = session.memory.model.access(task, name)?;
    if !session.memory.model.entries[object].sealed {
        return Err(abi::DENIED);
    }
    if let Some(mapping) = session.memory.model.mapped(task, name)? {
        return Ok(wire::address(mapping));
    }
    let (object, mapping) = session.memory.model.prepare_mapping(task, name)?;
    let pages = session.memory.model.entries[object].pages;
    // SAFETY: Retained sealed backing; root inactive and tables reserved at admission.
    unsafe {
        session.tasks[task].space.as_mut().unwrap().map_region(
            mapping,
            &session.memory.backing[object][..pages],
            false,
        )
    }
    .map_err(|_| abi::NO_MEMORY)?;
    session.memory.model.set_mapping(task, name, Some(mapping));
    session.tasks[task].report.memory.mapped_pages += pages;
    Ok(wire::address(mapping))
}
pub(super) unsafe fn unmap(session: &mut Session, task: usize, name: u64) -> Result<u64, u64> {
    let object = session.memory.model.access(task, name)?;
    let region = session.memory.model.entries[object];
    if !region.sealed
        || (session.memory.model.read_slot(task, name).is_none() && region.owner != Some(task))
    {
        return Err(abi::DENIED);
    }
    if let Some(mapping) = session.memory.model.mapped(task, name)? {
        // SAFETY: Caller surrendered its mapping, not its handle or any other reader.
        unsafe {
            session.tasks[task]
                .space
                .as_mut()
                .unwrap()
                .unmap_region(mapping, region.pages);
        }
        session.memory.model.set_mapping(task, name, None);
        session.tasks[task].report.memory.unmapped_pages += region.pages;
    }
    Ok(0)
}
pub(super) unsafe fn retire_escrow(session: &mut Session, frames: &mut Frames<'_>) {
    let pending = session.ipc.take_retired_memory();
    for (word, mut bits) in pending.into_iter().enumerate() {
        while bits != 0 {
            let slot = word * 64 + bits.trailing_zeros() as usize;
            bits &= bits - 1;
            if let Some(grant) = session.memory.model.reads[slot] {
                assert!(!grant.published && grant.mapping.is_none());
            }
            if let Some(object) = session.memory.model.drop_read(slot) {
                // SAFETY: Discarded escrow was never mapped; reclaim only after final reference.
                unsafe {
                    session.memory.reclaim(object, frames);
                }
            }
        }
    }
}
