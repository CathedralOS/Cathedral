//! Copied messages: authorize and validate buffers before dequeue, then complete waits.
use super::{Session, complete};
use crate::{ipc::Right, scheduler::Event};
use cathedral_kernel_contracts::user as abi;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ReceiveMode {
    Bytes,
    Endpoint,
    Memory,
    TryMemory,
    Notification,
}
#[derive(Clone, Copy)]
pub(in crate::user_tasks) struct Receive {
    handle: u64,
    address: u64,
    capacity: usize,
    attachment: ReceiveMode,
    deadline: Option<u64>,
}

// The deadline variant has a fixed 64-byte destination, leaving the third ABI
// argument for the absolute deadline. Check authority and the full destination
// before observing expiry; an available message or terminal error wins a tie.
pub(super) fn receive_call(
    session: &mut Session,
    slot: usize,
    call: (u64, u64, u64, ReceiveMode),
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
    if attachment == ReceiveMode::TryMemory {
        return Err(abi::WOULD_BLOCK);
    }
    if crate::user_tasks::time::timed_out(now, deadline, false) {
        return Err(abi::TIMED_OUT);
    }
    session.tasks[slot].receive = Some(wait);
    session.waits.register(
        slot,
        crate::user_tasks::execution::waits::Kind::Receive,
        deadline,
        now,
    );
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
    let size = if attachment != ReceiveMode::Bytes {
        abi::IPC_PACKET_BYTES
    } else {
        capacity
    };
    if !space.writable_range(address, size) {
        return Err(abi::BAD_ADDRESS);
    }
    let message = if attachment == ReceiveMode::Notification {
        session.ipc.receive_notification(slot, handle, capacity)?
    } else if matches!(attachment, ReceiveMode::Memory | ReceiveMode::TryMemory) {
        session.ipc.receive_memory(slot, handle, capacity)?
    } else {
        session
            .ipc
            .receive_cap(slot, handle, capacity, attachment == ReceiveMode::Endpoint)?
    };
    let Some(message) = message else {
        return Ok(None);
    };
    let capability = message
        .memory
        .map(|escrow| session.memory.model.publish_read(slot, escrow))
        .or(message.attachment)
        .or(message.notification);
    if attachment != ReceiveMode::Bytes {
        let mut packet = [0; abi::IPC_PACKET_BYTES];
        packet[..8].copy_from_slice(&(message.len as u64).to_le_bytes());
        packet[8..16].copy_from_slice(&capability.unwrap_or(0).to_le_bytes());
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
pub(in crate::user_tasks) fn wake(session: &mut Session, slot: usize, now: u64) {
    let wait = session.tasks[slot]
        .receive
        .expect("registered receive missing");
    let result = match receive(session, slot, wait) {
        Ok(None) if crate::user_tasks::time::timed_out(now, wait.deadline, false) => {
            Err(abi::TIMED_OUT)
        }
        Ok(None) => return,
        Ok(Some(length)) => Ok(length),
        Err(error) => Err(error),
    };
    session.tasks[slot].receive = None;
    session.waits.remove(slot);
    complete(session, slot, result);
    session.scheduler.unblock(slot);
    #[cfg(feature = "memory-lab")]
    if session.scheduler.states()[slot] == crate::scheduler::TaskState::Ready {
        session.timing.wait_ready(
            slot,
            crate::user_tasks::execution::waits::Kind::Receive,
            wait.deadline,
            now,
        );
    }
}
