//! Syscall policy and wait completion. Always kernel CR3, sole CPU, IRQs masked.
use super::{Exit, Session};
use crate::{ipc::Right, scheduler::Event, users::syscall as abi};

#[derive(Clone, Copy)]
pub(super) struct Receive {
    handle: u64,
    address: u64,
    capacity: usize,
    deadline: Option<u64>,
}

pub(super) fn syscall(session: &mut Session, slot: usize, now: u64) -> Event {
    let (number, first, second, third) = session.tasks[slot].context.syscall();
    let mut event = Event::Yield;
    let result = match number {
        abi::WRITE => write(session, slot, first, second),
        abi::YIELD => {
            session.tasks[slot].report.yields += 1;
            Ok(0)
        }
        abi::EXIT => {
            session.tasks[slot].report.exit = Some(Exit::Returned(first));
            event = Event::Exit;
            Ok(0)
        }
        abi::IPC_HANDLE => session.ipc.handle(slot, first),
        abi::IPC_SEND => send(session, slot, first, second, third),
        abi::IPC_RECEIVE | abi::IPC_RECEIVE_UNTIL => {
            let deadline = if number == abi::IPC_RECEIVE_UNTIL {
                Some(third)
            } else {
                None
            };
            let capacity = if deadline.is_some() {
                abi::MAX_MESSAGE as u64
            } else {
                third
            };
            receive_call(
                session,
                slot,
                (first, second, capacity),
                deadline,
                now,
                &mut event,
            )
        }
        abi::IPC_REVOKE => session.ipc.revoke(slot, first).map(|()| 0),
        abi::KEYBOARD_READ | abi::KEYBOARD_WRITE | abi::KEYBOARD_READ_UNTIL => {
            super::keyboard::syscall(session, slot, (number, first, second), &mut event, now)
        }
        abi::DISPLAY_INFO => display_info(session, slot, first, second),
        abi::TASK_LAUNCH..=abi::CLOCK_HANDLE => super::taskcalls::dispatch(
            session,
            slot,
            (number, first, second, third),
            &mut event,
            now,
        ),
        _ => Err(abi::UNKNOWN),
    };
    complete(session, slot, result);
    wake_receivers(session);
    event
}

// The deadline variant has a fixed 64-byte destination, leaving the third ABI
// argument for the absolute deadline. Check authority and the full destination
// before observing expiry; an available message or terminal error wins a tie.
fn receive_call(
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

fn display_info(session: &Session, slot: usize, address: u64, length: u64) -> Result<u64, u64> {
    let fb = session
        .launches
        .iter()
        .find(|launch| launch.model.child == slot)
        .and_then(|launch| launch.framebuffer)
        .ok_or(abi::DENIED)?;
    if length != cathedral_contracts::display::INFO_BYTES as u64 {
        return Err(abi::INVALID_ARGUMENT);
    }
    // SAFETY: Kernel root and IRQs off; only owned writable RAM is a copy target.
    if !unsafe {
        session.tasks[slot]
            .space
            .as_ref()
            .unwrap()
            .copy_to_user(address, &fb.user_info())
    } {
        return Err(abi::BAD_ADDRESS);
    }
    Ok(0)
}

pub(super) fn complete(session: &mut Session, slot: usize, result: Result<u64, u64>) {
    let task = &mut session.tasks[slot];
    if result.is_err() {
        task.report.rejected += 1;
    }
    if result == Err(abi::TIMED_OUT) {
        task.report.wait_timeouts += 1;
    }
    task.context
        .set_result(result.unwrap_or_else(|error| error));
}
fn write(session: &mut Session, slot: usize, address: u64, length: u64) -> Result<u64, u64> {
    if length > abi::MAX_WRITE as u64 {
        return Err(abi::INVALID_ARGUMENT);
    }
    let mut buffer = [0; abi::MAX_WRITE];
    // SAFETY: Kernel root, IRQs off; full source range checked against owned pages.
    if !unsafe {
        session.tasks[slot]
            .space
            .as_ref()
            .unwrap()
            .copy_from_user(address, &mut buffer[..length as usize])
    } {
        return Err(abi::BAD_ADDRESS);
    }
    if !(session.output)(&buffer[..length as usize]) {
        return Err(abi::IO_ERROR);
    }
    session.tasks[slot].report.writes += 1;
    Ok(length)
}
fn send(
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
pub(super) fn wake_receivers(session: &mut Session) {
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
