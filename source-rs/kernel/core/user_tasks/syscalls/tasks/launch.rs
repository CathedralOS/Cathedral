//! Copy a bounded launch request now; lifecycle::spawn admits and commits later.
use super::{Event, Session, abi};
pub(super) fn request(
    session: &mut Session,
    slot: usize,
    index: usize,
    call: (u64, u64, u64, u64),
    event: &mut Event,
) -> Result<u64, u64> {
    let (number, grant, second, third) = call;
    let mut initial = [0; abi::MAX_INITIAL_ENDPOINTS];
    let mut count = 0;
    let mut delegated = [0; abi::MAX_LAUNCHES];
    let mut delegated_count = 0;
    let argument = if matches!(
        number,
        abi::TASK_SPAWN_WITH | abi::TASK_SPAWN_TREE | abi::TASK_SPAWN_BEGIN
    ) {
        if third != 0 {
            return Err(abi::INVALID_ARGUMENT);
        }
        let mut bytes = [0; abi::SPAWN_TREE_BYTES];
        let length = if matches!(number, abi::TASK_SPAWN_TREE | abi::TASK_SPAWN_BEGIN) {
            abi::SPAWN_TREE_BYTES
        } else {
            abi::SPAWN_BYTES
        };
        // SAFETY: Serialized kernel root; checked copy from this task's backing.
        if !unsafe {
            session.tasks[slot]
                .space
                .as_ref()
                .unwrap()
                .copy_from_user(second, &mut bytes[..length])
        } {
            return Err(abi::BAD_ADDRESS);
        }
        let requested = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        if requested > abi::MAX_INITIAL_ENDPOINTS as u64 {
            return Err(abi::INVALID_ARGUMENT);
        }
        count = requested as usize;
        for (handle, bytes) in initial
            .iter_mut()
            .zip(bytes[16..abi::SPAWN_BYTES].chunks_exact(8))
        {
            *handle = u64::from_le_bytes(bytes.try_into().unwrap());
        }
        if initial[count..].iter().any(|&h| h != 0) {
            return Err(abi::INVALID_ARGUMENT);
        }
        let requested = u64::from_le_bytes(
            bytes[abi::SPAWN_BYTES..abi::SPAWN_BYTES + 8]
                .try_into()
                .unwrap(),
        );
        if requested > abi::MAX_SPAWN_LOANS as u64 {
            return Err(abi::INVALID_ARGUMENT);
        }
        delegated_count = requested as usize;
        for (handle, bytes) in delegated
            .iter_mut()
            .zip(bytes[abi::SPAWN_BYTES + 8..].chunks_exact(8))
        {
            *handle = u64::from_le_bytes(bytes.try_into().unwrap());
        }
        if delegated[delegated_count..].iter().any(|&h| h != 0) {
            return Err(abi::INVALID_ARGUMENT);
        }
        u64::from_le_bytes(bytes[..8].try_into().unwrap())
    } else {
        second
    };
    let receipt = if number == abi::TASK_SPAWN_BEGIN {
        session.names.mint(slot)?
    } else {
        0
    };
    let launch = &mut session.launches[index];
    if receipt == 0 {
        launch.model.request(slot, grant, argument)?;
    } else {
        launch.model.request_async(slot, grant, argument, receipt)?;
    }
    launch.initial = initial;
    launch.initial_count = count;
    launch.delegated = delegated;
    launch.delegated_count = delegated_count;
    session.deferred.insert(
        crate::user_tasks::execution::deferred::Work::Admission,
        index,
    );
    if receipt == 0 {
        *event = Event::Block;
    }
    Ok(receipt) // Blocking calls receive their result later; others poll this receipt.
}

pub(super) fn result(
    session: &mut Session,
    slot: usize,
    receipt: u64,
    address: u64,
    cancel: u64,
) -> Result<u64, u64> {
    if cancel > 1 {
        return Err(abi::INVALID_ARGUMENT);
    }
    let index = session
        .launches
        .iter()
        .position(|launch| launch.model.has_receipt(slot, receipt))
        .ok_or(abi::BAD_HANDLE)?;
    let space = session.tasks[slot].space.as_ref().unwrap();
    if !space.writable_range(address, 16) {
        return Err(abi::BAD_ADDRESS);
    }
    let Some(result) = session.launches[index]
        .model
        .poll_admission(slot, receipt, cancel == 1)?
    else {
        return Err(abi::WOULD_BLOCK);
    };
    let (status, child) = match result {
        Ok(child) => (0u64, child),
        Err(code) => (code, 0u64),
    };
    let mut bytes = [0; 16];
    bytes[..8].copy_from_slice(&status.to_le_bytes());
    bytes[8..].copy_from_slice(&child.to_le_bytes());
    // SAFETY: Serialized kernel root; preserve the receipt on failed copyout.
    if !unsafe { space.copy_to_user(address, &bytes) } {
        return Err(abi::BAD_ADDRESS);
    }
    session.launches[index].model.consume_admission();
    Ok(0)
}
pub(super) fn environment(
    session: &mut Session,
    slot: usize,
    address: u64,
    second: u64,
    third: u64,
) -> Result<u64, u64> {
    if second != 0 || third != 0 {
        return Err(abi::INVALID_ARGUMENT);
    }
    let handles = session.ipc.initial(slot)?;
    let mut bytes = [0; abi::ENVIRONMENT_BYTES];
    bytes[..8].copy_from_slice(&(handles.len() as u64).to_le_bytes());
    for (&handle, bytes) in handles.iter().zip(bytes[8..].chunks_exact_mut(8)) {
        bytes.copy_from_slice(&handle.to_le_bytes());
    }
    // SAFETY: Complete writable range validated by copy_to_user. No authority
    // changes on either success or failure: these are already-installed local names.
    if !unsafe {
        session.tasks[slot]
            .space
            .as_ref()
            .unwrap()
            .copy_to_user(address, &bytes)
    } {
        return Err(abi::BAD_ADDRESS);
    }
    Ok(0)
}
