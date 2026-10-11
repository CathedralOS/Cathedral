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
    let argument = if number == abi::TASK_SPAWN_WITH {
        if third != 0 {
            return Err(abi::INVALID_ARGUMENT);
        }
        let mut bytes = [0; abi::SPAWN_BYTES];
        // SAFETY: Serialized kernel root; checked copy from this task's backing.
        if !unsafe {
            session.tasks[slot]
                .space
                .as_ref()
                .unwrap()
                .copy_from_user(second, &mut bytes)
        } {
            return Err(abi::BAD_ADDRESS);
        }
        let requested = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        if requested > abi::MAX_INITIAL_ENDPOINTS as u64 {
            return Err(abi::INVALID_ARGUMENT);
        }
        count = requested as usize;
        for (handle, bytes) in initial.iter_mut().zip(bytes[16..].chunks_exact(8)) {
            *handle = u64::from_le_bytes(bytes.try_into().unwrap());
        }
        if initial[count..].iter().any(|&h| h != 0) {
            return Err(abi::INVALID_ARGUMENT);
        }
        u64::from_le_bytes(bytes[..8].try_into().unwrap())
    } else {
        second
    };
    let launch = &mut session.launches[index];
    launch.model.request(slot, grant, argument)?;
    launch.initial = initial;
    launch.initial_count = count;
    *event = Event::Block;
    Ok(0) // The boot-stack admission path returns the committed child or an error.
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
