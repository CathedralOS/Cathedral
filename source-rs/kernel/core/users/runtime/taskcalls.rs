//! Task syscalls only validate/enqueue work; spawn and reclamation use boot context.
use super::{Session, dispatch};
use crate::scheduler::Event;
use cathedral_contracts::user as abi;

#[derive(Clone, Copy)]
pub(super) struct Wait {
    ticket: u64,
    address: u64,
}

pub(super) fn dispatch(
    session: &mut Session,
    slot: usize,
    number: u64,
    first: u64,
    second: u64,
    third: u64,
    event: &mut Event,
) -> Result<u64, u64> {
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
        abi::TASK_WAIT => {
            model.wait(slot, first)?;
            if third != abi::EXIT_BYTES as u64 {
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
            };
            if !finish_wait(session, slot, wait)? {
                session.tasks[slot].wait = Some(wait);
                session.tasks[slot].report.waits_blocked += 1;
                *event = Event::Block;
            }
            Ok(0)
        }
        _ => unreachable!(),
    }
}
fn finish_wait(session: &mut Session, slot: usize, wait: Wait) -> Result<bool, u64> {
    let model = session.supervisor.as_mut().unwrap();
    let Some(bytes) = model.wait(slot, wait.ticket)? else {
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
pub(super) fn wake(session: &mut Session) {
    let Some(model) = &session.supervisor else {
        return;
    };
    let slot = model.owner;
    let Some(wait) = session.tasks[slot].wait else {
        return;
    };
    let result = match finish_wait(session, slot, wait) {
        Ok(false) => return,
        Ok(true) => Ok(0),
        Err(error) => Err(error),
    };
    session.tasks[slot].wait = None;
    dispatch::complete(session, slot, result);
    session.scheduler.unblock(slot);
}
