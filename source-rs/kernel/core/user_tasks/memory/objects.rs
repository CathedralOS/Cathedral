//! Deferred mapping edits for frozen and live shared objects. A read capability can exist and
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
    // SAFETY: Same inactive-root custody; immutable backing only.
    unsafe { map_kind(session, task, name, false) }
}
pub(super) unsafe fn map_kind(
    session: &mut Session,
    task: usize,
    name: u64,
    mutable: bool,
) -> Result<u64, u64> {
    let object = session.memory.model.access(task, name)?;
    let region = session.memory.model.entries[object];
    if !(if mutable {
        region.shared_mutable
    } else {
        region.sealed
    }) {
        return Err(abi::DENIED);
    }
    if let Some(mapping) = session.memory.model.mapped(task, name)? {
        return Ok(wire::address(mapping));
    }
    let (object, mapping) = session.memory.model.prepare_mapping(task, name)?;
    let pages = session.memory.model.entries[object].pages;
    // SAFETY: Retained backing; inactive root. Only its producer may map writable.
    // Delegated views remain RO/NX, even while their producer continues writing.
    unsafe {
        session.tasks[task].space.as_mut().unwrap().map_region(
            mapping,
            &session.memory.backing[object][..pages],
            mutable && region.owner == Some(task) && region.owner_name == name,
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
    if !(region.sealed || region.shared_mutable)
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
