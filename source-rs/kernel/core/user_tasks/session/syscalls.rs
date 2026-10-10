//! Syscall entrance: route by authority/resource family, then complete and wake.
//! Platform APIs reach this transport through platform/libraries/user-runtime/;
//! display/input/storage request semantics remain in their service executables.
mod diagnostics;
mod disk;
mod display;
mod ipc;
pub(super) mod keyboard;
mod links;
mod memory;
pub(super) mod readiness;
pub(super) mod tasks;
use super::{Exit, Session};
use crate::{scheduler::Event, user_tasks::syscall as abi};
pub(super) use ipc::{Receive, wake_receivers};
pub(super) fn syscall(session: &mut Session, slot: usize, now: u64) -> Event {
    let (number, first, second, third) = session.tasks[slot].context.syscall();
    let mut event = Event::Yield;
    let result = match number {
        abi::WRITE => diagnostics::write(session, slot, first, second),
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
        abi::IPC_SEND => ipc::send(session, slot, first, second, third),
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
            ipc::receive_call(
                session,
                slot,
                (first, second, capacity),
                deadline,
                now,
                &mut event,
            )
        }
        abi::LINK_HANDLE | abi::LINK_CONNECT => links::syscall(session, slot, number, first),
        abi::IPC_WAIT_TWO => readiness::syscall(
            session,
            slot,
            readiness::Wait {
                handles: [first, second],
                deadline: third,
            },
            now,
            &mut event,
        ),
        abi::IPC_REVOKE => session.ipc.revoke(slot, first).map(|()| 0),
        abi::KEYBOARD_READ | abi::KEYBOARD_WRITE | abi::KEYBOARD_READ_UNTIL => {
            keyboard::syscall(session, slot, (number, first, second), &mut event, now)
        }
        abi::DISK_REGISTER_READ..=abi::DISK_TRANSFER => {
            disk::syscall(session, slot, (number, first, second, third))
        }
        abi::DISPLAY_INFO => display::information(session, slot, first, second),
        abi::TASK_LAUNCH..=abi::CLOCK_HANDLE => tasks::dispatch(
            session,
            slot,
            (number, first, second, third),
            &mut event,
            now,
        ),
        abi::MEMORY_ALLOCATE..=abi::MEMORY_PAGES => {
            memory::syscall(session, slot, [number, first, second, third], &mut event)
        }
        _ => Err(abi::UNKNOWN),
    };
    complete(session, slot, result);
    ipc::wake_receivers(session);
    event
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
