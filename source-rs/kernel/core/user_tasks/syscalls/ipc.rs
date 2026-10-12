//! IPC request entrance: delivery, capability transfer, peer links and readiness.
//! Copied delivery and pending receive completion share the checked-buffer path below.
mod capabilities;
mod links;
pub(in crate::user_tasks) mod readiness;
use super::{Session, complete};
use crate::{ipc::Right, scheduler::Event};
use cathedral_contracts::user as abi;
#[derive(Clone, Copy)]
pub(in crate::user_tasks) struct Receive {
    handle: u64,
    address: u64,
    capacity: usize,
    attachment: bool,
    deadline: Option<u64>,
}

pub(super) fn dispatch(
    session: &mut Session,
    slot: usize,
    [number, first, second, third]: [u64; 4],
    now: u64,
    event: &mut Event,
) -> Result<u64, u64> {
    match number {
        abi::IPC_BUDGET => capabilities::budget(session, slot, first, second, third),
        abi::IPC_CREATE => capabilities::create(session, slot, first, second, third),
        abi::IPC_CLOSE => {
            if second != 0 || third != 0 {
                Err(abi::INVALID_ARGUMENT)
            } else {
                session.ipc.close(slot, first).map(|()| 0)
            }
        }
        abi::IPC_SEND_CAP => capabilities::send_cap(session, slot, first, second, third),
        abi::IPC_RECEIVE_CAP => {
            if third != 0 {
                return Err(abi::INVALID_ARGUMENT);
            }
            receive_call(
                session,
                slot,
                (first, second, abi::MAX_MESSAGE as u64, true),
                None,
                now,
                event,
            )
        }
        abi::IPC_HANDLE => session.ipc.handle(slot, first),
        abi::IPC_SEND => send(session, slot, first, second, third),
        abi::IPC_RECEIVE | abi::IPC_RECEIVE_UNTIL => {
            let deadline = (number == abi::IPC_RECEIVE_UNTIL).then_some(third);
            let capacity = if deadline.is_some() {
                abi::MAX_MESSAGE as u64
            } else {
                third
            };
            receive_call(
                session,
                slot,
                (first, second, capacity, false),
                deadline,
                now,
                event,
            )
        }
        abi::LINK_HANDLE | abi::LINK_CONNECT => links::syscall(session, slot, number, first),
        abi::IPC_WAIT_TWO => readiness::syscall(
            session,
            slot,
            readiness::Wait {
                handles: [first, second, 0, 0, 0, 0, 0, 0],
                count: 2,
                deadline: third,
            },
            now,
            event,
        ),
        abi::IPC_WAIT_MANY => readiness::many(session, slot, [first, second, third], now, event),
        abi::IPC_REVOKE => session.ipc.revoke(slot, first).map(|()| 0),
        _ => Err(abi::UNKNOWN),
    }
}

// The deadline variant has a fixed 64-byte destination, leaving the third ABI
// argument for the absolute deadline. Check authority and the full destination
// before observing expiry; an available message or terminal error wins a tie.
pub(super) fn receive_call(
    session: &mut Session,
    slot: usize,
    call: (u64, u64, u64, bool),
    deadline: Option<u64>,
    now: u64,
    event: &mut Event,
) -> Result<u64, u64> {
    let (handle, address, capacity, attachment) = call;
    if let Some(deadline) = deadline {
        session.clock.handle(slot)?;
        if !crate::user_tasks::time::valid(now, deadline) {
            return Err(abi::INVALID_ARGUMENT);
        }
    }
    let wait = Receive {
        handle,
        address,
        capacity: usize::try_from(capacity).map_err(|_| abi::INVALID_ARGUMENT)?,
        attachment,
        deadline,
    };
    if let Some(length) = receive(session, slot, wait)? {
        return Ok(length);
    }
    if crate::user_tasks::time::timed_out(now, deadline, false) {
        return Err(abi::TIMED_OUT);
    }
    session.tasks[slot].receive = Some(wait);
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
fn receive(session: &mut Session, slot: usize, wait: Receive) -> Result<Option<u64>, u64> {
    let Receive {
        handle,
        address,
        capacity,
        attachment,
        ..
    } = wait;
    session.ipc.check(slot, handle, Right::Receive)?;
    if capacity > abi::MAX_MESSAGE {
        return Err(abi::INVALID_ARGUMENT);
    }
    let task = &mut session.tasks[slot];
    let space = task.space.as_ref().unwrap();
    let size = if attachment {
        abi::IPC_PACKET_BYTES
    } else {
        capacity
    };
    if !space.writable_range(address, size) {
        return Err(abi::BAD_ADDRESS);
    }
    let Some(message) = session
        .ipc
        .receive_cap(slot, handle, capacity, attachment)?
    else {
        return Ok(None);
    };
    if attachment {
        let mut packet = [0; abi::IPC_PACKET_BYTES];
        packet[..8].copy_from_slice(&(message.len as u64).to_le_bytes());
        packet[8..16].copy_from_slice(&message.attachment.unwrap_or(0).to_le_bytes());
        packet[16..16 + message.len].copy_from_slice(&message.bytes[..message.len]);
        // SAFETY: Complete writable destination checked before dequeue/install. Serialized trap.
        assert!(unsafe { space.copy_to_user(address, &packet) });
    } else {
        // SAFETY: Complete writable destination checked before dequeue. Serialized trap.
        assert!(unsafe { space.copy_to_user(address, &message.bytes[..message.len]) });
    }
    task.report.ipc_received += 1;
    Ok(Some(message.len as u64))
}
pub(in super::super) fn wake_receivers(session: &mut Session) {
    readiness::wake(session);
    for slot in 0..session.tasks.len() {
        let Some(wait) = session.tasks[slot].receive else {
            continue;
        };
        let result = match receive(session, slot, wait) {
            Ok(None)
                if crate::user_tasks::time::timed_out(
                    cathedral_arch::ticks(),
                    wait.deadline,
                    false,
                ) =>
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
