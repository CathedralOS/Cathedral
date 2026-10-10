//! Copied-message delivery, checked buffers and pending receive completion.
use super::{Session, complete};
use crate::{ipc::Right, scheduler::Event, user_tasks::syscall as abi};
#[derive(Clone, Copy)]
pub(in super::super) struct Receive {
    handle: u64,
    address: u64,
    capacity: usize,
    deadline: Option<u64>,
}

// The deadline variant has a fixed 64-byte destination, leaving the third ABI
// argument for the absolute deadline. Check authority and the full destination
// before observing expiry; an available message or terminal error wins a tie.
pub(super) fn receive_call(
    session: &mut Session,
    slot: usize,
    call: (u64, u64, u64),
    deadline: Option<u64>,
    now: u64,
    event: &mut Event,
) -> Result<u64, u64> {
    let (handle, address, capacity) = call;
    if let Some(deadline) = deadline {
        session.clock.handle(slot)?;
        if !crate::deadline::valid(now, deadline) {
            return Err(abi::INVALID_ARGUMENT);
        }
    }
    if let Some(length) = receive(session, slot, handle, address, capacity)? {
        return Ok(length);
    }
    if crate::deadline::timed_out(now, deadline, false) {
        return Err(abi::TIMED_OUT);
    }
    session.tasks[slot].receive = Some(Receive {
        handle,
        address,
        capacity: capacity as usize,
        deadline,
    });
    session.tasks[slot].report.receives_blocked += 1;
    *event = Event::Block;
    Ok(0)
}

pub(super) fn send(
    session: &mut Session,
    slot: usize,
    handle: u64,
    address: u64,
    length: u64,
) -> Result<u64, u64> {
    session.ipc.check(slot, handle, Right::Send)?;
    if length > abi::MAX_MESSAGE as u64 {
        return Err(abi::INVALID_ARGUMENT);
    }
    let mut buffer = [0; abi::MAX_MESSAGE];
    // SAFETY: Kernel root, IRQs off; checked task-owned physical backing.
    if !unsafe {
        session.tasks[slot]
            .space
            .as_ref()
            .unwrap()
            .copy_from_user(address, &mut buffer[..length as usize])
    } {
        return Err(abi::BAD_ADDRESS);
    }
    session.ipc.send(slot, handle, &buffer[..length as usize])?;
    session.tasks[slot].report.ipc_sent += 1;
    Ok(length)
}
fn receive(
    session: &mut Session,
    slot: usize,
    handle: u64,
    address: u64,
    capacity: u64,
) -> Result<Option<u64>, u64> {
    session.ipc.check(slot, handle, Right::Receive)?;
    if capacity > abi::MAX_MESSAGE as u64 {
        return Err(abi::INVALID_ARGUMENT);
    }
    let task = &mut session.tasks[slot];
    let space = task.space.as_ref().unwrap();
    if !space.writable_range(address, capacity as usize) {
        return Err(abi::BAD_ADDRESS);
    }
    let Some(message) = session.ipc.receive(slot, handle, capacity as usize)? else {
        return Ok(None);
    };
    // SAFETY: Validated writable range above; no map changes/concurrency while parked.
    assert!(unsafe { space.copy_to_user(address, &message.bytes[..message.len]) });
    task.report.ipc_received += 1;
    Ok(Some(message.len as u64))
}
pub(in super::super) fn wake_receivers(session: &mut Session) {
    super::readiness::wake(session);
    for slot in 0..session.tasks.len() {
        let Some(wait) = session.tasks[slot].receive else {
            continue;
        };
        let result = match receive(
            session,
            slot,
            wait.handle,
            wait.address,
            wait.capacity as u64,
        ) {
            Ok(None)
                if crate::deadline::timed_out(cathedral_arch::ticks(), wait.deadline, false) =>
            {
                Err(abi::TIMED_OUT)
            }
            Ok(None) => continue,
            Ok(Some(length)) => Ok(length),
            Err(error) => Err(error),
        };
        session.tasks[slot].receive = None;
        complete(session, slot, result);
        session.scheduler.unblock(slot);
    }
}
