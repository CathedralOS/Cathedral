// Task syscalls only validate/enqueue work; spawn and reclamation use boot context.
use super::{Session, complete};
use crate::scheduler::Event;
use crate::user_tasks::time as deadline;
use cathedral_contracts::user as abi;

#[derive(Clone, Copy)]
pub(in super::super) struct Wait {
    launch: usize,
    ticket: u64,
    address: u64,
    deadline: Option<u64>,
}

pub(in super::super) fn dispatch(
    session: &mut Session,
    slot: usize,
    call: (u64, u64, u64, u64),
    event: &mut Event,
    now: u64,
) -> Result<u64, u64> {
    let (number, first, second, third) = call;
    if number == abi::TASK_ENVIRONMENT {
        return launch::environment(session, slot, first, second, third);
    }
    if number == abi::YIELD {
        *event = Event::Yield;
        session.tasks[slot].report.yields += 1;
        return Ok(0);
    }
    if number == abi::EXIT {
        session.tasks[slot].report.exit = Some(crate::user_tasks::Exit::Returned(first));
        *event = Event::Exit;
        return Ok(0);
    }
    if number == abi::TASK_LAUNCH || number == abi::TASK_PORT {
        let grants: fn(
            &crate::user_tasks::lifecycle::supervision::Supervisor,
            usize,
        ) -> Result<u64, u64> = if number == abi::TASK_LAUNCH {
            crate::user_tasks::lifecycle::supervision::Supervisor::launch
        } else {
            crate::user_tasks::lifecycle::supervision::Supervisor::port
        };
        let mut tickets = session
            .launches
            .iter()
            .filter_map(|launch| grants(&launch.model, slot).ok());
        let first_ticket = tickets.next().ok_or(abi::DENIED)?;
        return if first == 0 {
            Ok(first_ticket)
        } else {
            tickets
                .nth(usize::try_from(first - 1).map_err(|_| abi::BAD_HANDLE)?)
                .ok_or(abi::BAD_HANDLE)
        };
    }
    let index = session
        .launches
        .iter()
        .position(|launch| match number {
            abi::TASK_SPAWN | abi::TASK_SPAWN_WITH | abi::TASK_SPAWN_TREE => {
                launch.model.launch(slot) == Ok(first)
            }
            abi::TASK_CONNECT => launch.model.port(slot) == Ok(first),
            _ => launch.model.wait(slot, first).is_ok(),
        })
        .ok_or(abi::BAD_HANDLE)?;
    if matches!(
        number,
        abi::TASK_SPAWN | abi::TASK_SPAWN_WITH | abi::TASK_SPAWN_TREE
    ) {
        return launch::request(session, slot, index, call, event);
    }
    let launch = &mut session.launches[index];
    let model = &mut launch.model;
    match number {
        abi::TASK_CONNECT => {
            model.connect(slot, first)?;
            Ok(session.ipc.accept_child(launch.endpoint_base, slot))
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
                launch: index,
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
    let model = &mut session.launches[wait.launch].model;
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
pub(in super::super) fn wake(session: &mut Session, now: u64) {
    for slot in 0..session.tasks.len() {
        let Some(wait) = session.tasks[slot].wait else {
            continue;
        };
        let result = match finish_wait(session, slot, wait, now) {
            Ok(false) => continue,
            Ok(true) => Ok(0),
            Err(error) => Err(error),
        };
        session.tasks[slot].wait = None;
        complete(session, slot, result);
        session.scheduler.unblock(slot);
    }
}
