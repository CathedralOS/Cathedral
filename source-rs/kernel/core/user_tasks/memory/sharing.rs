//! Seal producer mappings before offering; accept only through a granted peer link.
use super::{Session, abi};
use crate::scheduler::TaskState;

pub(super) unsafe fn seal(
    session: &mut Session,
    slot: usize,
    handle: u64,
    port: u64,
) -> Result<u64, u64> {
    let peer = session
        .links
        .iter()
        .find(|link| link.model.spec.client == slot && link.model.port(slot) == Ok(port))
        .map(|link| link.model.spec.service)
        .ok_or(abi::BAD_HANDLE)?;
    if matches!(
        session.scheduler.states()[peer],
        TaskState::Vacant | TaskState::Exited
    ) {
        return Err(abi::PEER_CLOSED);
    }
    let index = session.memory.model.offer(slot, handle, peer)?;
    let pages = session.memory.model.entries[index].pages;
    // SAFETY: No consumer has mapped yet; downgrade the inactive producer before returning the offer.
    unsafe {
        session.tasks[slot]
            .space
            .as_mut()
            .unwrap()
            .seal_region(session.memory.model.mapping(slot, index), pages);
    }
    session.tasks[slot].report.memory.sealed_pages += pages;
    Ok(0)
}

pub(super) unsafe fn map(
    session: &mut Session,
    slot: usize,
    handle: u64,
    port: u64,
) -> Result<u64, u64> {
    // A producer-local offer is meaningful only through the receiving link.
    let owner = session
        .links
        .iter()
        .find(|link| link.model.spec.service == slot && link.model.port(slot) == Ok(port))
        .map(|link| link.model.spec.client)
        .ok_or(abi::BAD_HANDLE)?;
    let index = session.memory.model.accept(slot, owner, handle)?;
    let pages = session.memory.model.entries[index].pages;
    // SAFETY: Sealed owner is read-only; backing is pinned until this accepted reader releases/dies.
    let result = unsafe {
        session.tasks[slot].space.as_mut().unwrap().map_region(
            session.memory.model.mapping(slot, index),
            &session.memory.backing[index][..pages],
            false,
        )
    };
    if result.is_err() {
        session.memory.model.entries[index].accepted = false;
        return Err(abi::NO_MEMORY);
    }
    session.tasks[slot].report.memory.mapped_pages += pages;
    Ok(session.memory.model.handle(slot, index))
}
