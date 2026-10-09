//! Device authority and bounded raw-byte delivery. No key decoding or UI policy.
use super::{Session, dispatch};
use crate::scheduler::Event;
use cathedral_arch as arch;
use cathedral_contracts::user as abi;

fn owner(session: &Session) -> Option<usize> {
    session
        .launches
        .iter()
        .find(|launch| launch.keyboard)
        .map(|launch| launch.model.child)
}
pub(super) fn syscall(
    session: &mut Session,
    slot: usize,
    call: (u64, u64, u64),
    event: &mut Event,
    now: u64,
) -> Result<u64, u64> {
    let (number, first, second) = call;
    if owner(session) != Some(slot) {
        return Err(abi::DENIED);
    }
    if number == abi::KEYBOARD_WRITE {
        if first > 1
            || second > 255
            || (first == 1 && !matches!(second, 0x20 | 0x60 | 0xa7 | 0xad | 0xae))
        {
            return Err(abi::INVALID_ARGUMENT);
        }
        // SAFETY: Caller has the exclusive boot-issued controller grant; only
        // data/mode/interface operations are permitted, never reset/A20 commands.
        return unsafe { arch::keyboard_write(first == 1, second as u8) }
            .then_some(0)
            .ok_or(abi::WOULD_BLOCK);
    }
    let deadline = if number == abi::KEYBOARD_READ_UNTIL {
        session.clock.handle(slot)?;
        if !crate::deadline::valid(now, first) {
            return Err(abi::INVALID_ARGUMENT);
        }
        Some(first)
    } else {
        None
    };
    if deadline.is_none() && first > 1 {
        return Err(abi::INVALID_ARGUMENT);
    }
    poll(session, now);
    if let Some(byte) = session.keyboard.pop() {
        return Ok(byte);
    }
    if deadline.is_none() && first == 0 {
        return Err(abi::WOULD_BLOCK);
    }
    if crate::deadline::timed_out(now, deadline, false) {
        return Err(abi::TIMED_OUT);
    }
    session.tasks[slot].keyboard_wait = true;
    session.tasks[slot].keyboard_deadline = deadline;
    session.tasks[slot].report.keyboard_reads_blocked += 1;
    *event = Event::Block;
    Ok(0)
}
pub(super) fn poll(session: &mut Session, now: u64) {
    let Some(slot) = owner(session) else {
        return;
    };
    for _ in 0..64 {
        // SAFETY: Session owns the PC controller and IRQ route, IRQs masked.
        let Some((status, byte)) = (unsafe { arch::keyboard_read() }) else {
            break;
        };
        if session.tasks[slot].space.is_none() || status & 0x20 != 0 {
            continue;
        }
        if status & 0xc0 != 0 {
            session.keyboard.lose();
        } else {
            session.keyboard.push(byte);
        }
    }
    if session.tasks[slot].keyboard_wait {
        let result = match session.keyboard.pop() {
            Some(byte) => Ok(byte),
            None if crate::deadline::timed_out(
                now,
                session.tasks[slot].keyboard_deadline,
                false,
            ) =>
            {
                Err(abi::TIMED_OUT)
            }
            None => return,
        };
        session.tasks[slot].keyboard_wait = false;
        session.tasks[slot].keyboard_deadline = None;
        dispatch::complete(session, slot, result);
        session.scheduler.unblock(slot);
    }
}
pub(super) fn reset(session: &mut Session) {
    session.keyboard.reset();
    for _ in 0..64 {
        // SAFETY: Grant incarnation is retiring or not yet runnable; discard old bytes.
        if unsafe { arch::keyboard_read() }.is_none() {
            break;
        }
    }
}
