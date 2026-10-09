//! Task syscalls only validate/enqueue work; spawn and reclamation use boot context.
use super::{Session, dispatch};
use crate::deadline;
use crate::scheduler::Event;
use cathedral_contracts::user as abi;

#[derive(Clone, Copy)]
pub(super) struct Wait {
    ticket: u64,
    address: u64,
    deadline: Option<u64>,
}

pub(super) fn dispatch(
    session: &mut Session,
    slot: usize,
    call: (u64, u64, u64, u64),
    event: &mut Event,
    now: u64,
) -> Result<u64, u64> {
    let (number, first, second, third) = call;
    if number == abi::CLOCK_HANDLE {
        return session.clock.handle(slot);
    }
    if number == abi::CLOCK_READ {
        session.clock.check(slot, first)?;
        if third != 8 {
            return Err(abi::INVALID_ARGUMENT);
        }
        // SAFETY: Kernel CR3, IRQs off; copy validates the complete writable range.
        if !unsafe {
            session.tasks[slot]
                .space
                .as_ref()
                .unwrap()
                .copy_to_user(second, &now.to_le_bytes())
        } {
            return Err(abi::BAD_ADDRESS);
        }
        return Ok(0);
    }
    let model = session.supervisor.as_mut().ok_or(abi::DENIED)?;
    match number {
        abi::TASK_LAUNCH => model.launch(slot),
        abi::TASK_PORT => model.port(slot),
        abi::TASK_SPAWN => {
            model.request(slot, first, second)?;
            *event = Event::Block;
            Ok(0) // Boot admission replaces this result before waking the owner.
        }
        abi::TASK_CONNECT => {
            model.connect(slot, first)?;
            let mut index = 0;
            while session.ipc.handle(slot, index).is_ok() {
                index += 1;
            }
            session.ipc.accept_child(session.endpoint_base, slot);
            Ok(index)
        }
        abi::TASK_CANCEL => {
            if model.request_cancel(slot, first)? {
                *event = Event::Block;
            }
            Ok(0) // Boot reclaims a live child before completing this call.
        }
        abi::TASK_WAIT | abi::TASK_WAIT_UNTIL => {
            model.wait(slot, first)?;
            if number == abi::TASK_WAIT_UNTIL {
                session.clock.handle(slot)?;
            }
            if number == abi::TASK_WAIT && third != abi::EXIT_BYTES as u64 {
                return Err(abi::INVALID_ARGUMENT);
            }
            if !session.tasks[slot]
                .space
                .as_ref()
                .unwrap()
                .writable_range(second, abi::EXIT_BYTES)
            {
                return Err(abi::BAD_ADDRESS);
            }
            let wait = Wait {
                ticket: first,
                address: second,
                deadline: if number == abi::TASK_WAIT_UNTIL {
                    if !deadline::valid(now, third) {
                        return Err(abi::INVALID_ARGUMENT);
                    }
                    Some(third)
                } else {
                    None
                },
            };
            if !finish_wait(session, slot, wait, now)? {
                session.tasks[slot].wait = Some(wait);
                session.tasks[slot].report.waits_blocked += 1;
                *event = Event::Block;
            }
            Ok(0)
        }
        _ => unreachable!(),
    }
}
fn finish_wait(session: &mut Session, slot: usize, wait: Wait, now: u64) -> Result<bool, u64> {
    let model = session.supervisor.as_mut().unwrap();
    let outcome = model.wait(slot, wait.ticket)?;
    if deadline::timed_out(now, wait.deadline, outcome.is_some()) {
        return Err(abi::TIMED_OUT);
    }
    let Some(bytes) = outcome else {
        return Ok(false);
    };
    // SAFETY: Kernel root, IRQs off; copy validates full writable range. Spaces
    // cannot change while waiting. Consume outcome only after successful copyout.
    if !unsafe {
        session.tasks[slot]
            .space
            .as_ref()
            .unwrap()
            .copy_to_user(wait.address, &bytes)
    } {
        return Err(abi::BAD_ADDRESS);
    }
    model.consume();
    Ok(true)
}
pub(super) fn wake(session: &mut Session, now: u64) {
    let Some(model) = &session.supervisor else {
        return;
    };
    let slot = model.owner;
    let Some(wait) = session.tasks[slot].wait else {
        return;
    };
    let result = match finish_wait(session, slot, wait, now) {
        Ok(false) => return,
        Ok(true) => Ok(0),
        Err(error) => Err(error),
    };
    session.tasks[slot].wait = None;
    dispatch::complete(session, slot, result);
    session.scheduler.unblock(slot);
}
