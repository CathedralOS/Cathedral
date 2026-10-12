//! Caller-local account inspection and atomic subdivision into an owned idle child.
use super::{Session, abi};
pub(super) fn dispatch(
    session: &mut Session,
    slot: usize,
    [operation, first, second, third]: [u64; 4],
) -> Result<u64, u64> {
    if third != 0 {
        return Err(abi::INVALID_ARGUMENT);
    }
    if operation == abi::PHYSICAL_BUDGET {
        return physical_budget(session, slot, first, second);
    }
    if operation == abi::RESOURCE_BUDGET {
        if second != 0 {
            return Err(abi::INVALID_ARGUMENT);
        }
        let values = crate::resources::snapshot(&session.ipc, &session.memory.model, slot);
        let mut bytes = [0; 48];
        for (value, target) in values.into_iter().zip(bytes.chunks_exact_mut(8)) {
            target.copy_from_slice(&(value as u64).to_le_bytes());
        }
        // SAFETY: Complete writable destination validated before copying; sole CPU.
        if !unsafe {
            session.tasks[slot]
                .space
                .as_ref()
                .unwrap()
                .copy_to_user(first, &bytes)
        } {
            return Err(abi::BAD_ADDRESS);
        }
    } else {
        let model = &session
            .launches
            .iter()
            .find(|launch| launch.model.launch(slot) == Ok(first))
            .ok_or(abi::BAD_HANDLE)?
            .model;
        if !model.idle() {
            return Err(abi::BUSY);
        }
        let child = model.child;
        if operation == abi::PHYSICAL_ASSIGN {
            session.memory.funding.assign(
                slot,
                child,
                usize::try_from(second).map_err(|_| abi::INVALID_ARGUMENT)?,
            )?;
            return Ok(0);
        }
        let mut bytes = [0; 24];
        // SAFETY: Complete readable source checked before any reservation changes.
        if !unsafe {
            session.tasks[slot]
                .space
                .as_ref()
                .unwrap()
                .copy_from_user(second, &mut bytes)
        } {
            return Err(abi::BAD_ADDRESS);
        }
        let mut limits = [0; 3];
        for (limit, bytes) in limits.iter_mut().zip(bytes.chunks_exact(8)) {
            *limit = usize::try_from(u64::from_le_bytes(bytes.try_into().unwrap()))
                .map_err(|_| abi::INVALID_ARGUMENT)?;
        }
        crate::resources::assign(
            &mut session.ipc,
            &mut session.memory.model,
            slot,
            child,
            limits,
        )?;
    }
    Ok(0)
}

fn physical_budget(
    session: &mut Session,
    caller: usize,
    address: u64,
    reserved: u64,
) -> Result<u64, u64> {
    if reserved != 0 {
        return Err(abi::INVALID_ARGUMENT);
    }
    let values = session.memory.funding.snapshot(caller);
    let mut bytes = [0; 16];
    for (value, target) in values.into_iter().zip(bytes.chunks_exact_mut(8)) {
        target.copy_from_slice(&(value as u64).to_le_bytes());
    }
    // SAFETY: The complete writable range is validated; no mutation before copyout.
    if !unsafe {
        session.tasks[caller]
            .space
            .as_ref()
            .unwrap()
            .copy_to_user(address, &bytes)
    } {
        return Err(abi::BAD_ADDRESS);
    }
    Ok(0)
}
