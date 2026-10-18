// Device authority and bounded raw-byte delivery. No key decoding or UI policy.
use crate::scheduler::Event;
use crate::user_tasks::syscalls::{Session, complete};
use cathedral_arch as arch;
use cathedral_kernel_contracts::user as abi;

fn owner(session: &Session) -> Option<usize> {
    session
        .launches
        .iter()
        .find(|launch| launch.keyboard)
        .map(|launch| launch.model.child)
}
pub(in crate::user_tasks) fn syscall(
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
        let device = arch::board::devices().keyboard.ok_or(abi::DENIED)?;
        // SAFETY: Exclusive controller grant; adapter validates command/data before device access.
        return unsafe { device.write(first, second) }
            .map(|()| 0)
            .map_err(crate::user_tasks::syscalls::devices::error);
    }
    let deadline = if number == abi::KEYBOARD_READ_UNTIL {
        session.clock.handle(slot)?;
        if !crate::user_tasks::time::valid(now, first) {
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
    if crate::user_tasks::time::timed_out(now, deadline, false) {
        return Err(abi::TIMED_OUT);
    }
    session.tasks[slot].keyboard_wait = true;
    session.tasks[slot].keyboard_deadline = deadline;
    session.tasks[slot].report.keyboard_reads_blocked += 1;
    *event = Event::Block;
    Ok(0)
}
pub(in crate::user_tasks) fn poll(session: &mut Session, now: u64) {
    let Some(slot) = owner(session) else {
        return;
    };
    let Some(device) = arch::board::devices().keyboard else {
        return;
    };
    for _ in 0..64 {
        // SAFETY: Session owns the PC controller and IRQ route, IRQs masked.
        let Some(event) = (unsafe { device.read() }) else {
            break;
        };
        if session.tasks[slot].space.is_none() {
            continue;
        }
        match event {
            arch::board::InputEvent::Lost => session.keyboard.lose(),
            arch::board::InputEvent::Byte(byte) => session.keyboard.push_tagged(byte),
        }
    }
    if session.tasks[slot].keyboard_wait {
        let result = match session.keyboard.pop() {
            Some(byte) => Ok(byte),
            None if crate::user_tasks::time::timed_out(
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
        complete(session, slot, result);
        session.scheduler.unblock(slot);
    }
}
pub(in crate::user_tasks) fn reset(session: &mut Session) {
    session.keyboard.reset();
    let Some(device) = arch::board::devices().keyboard else {
        return;
    };
    virtual_pointer::reset(&device);
    for _ in 0..64 {
        // SAFETY: Grant incarnation is retiring or not yet runnable; discard old bytes.
        if unsafe { device.read() }.is_none() {
            break;
        }
    }
}
