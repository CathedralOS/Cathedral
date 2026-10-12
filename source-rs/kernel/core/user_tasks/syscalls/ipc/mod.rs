//! IPC request entrance: delivery, capability transfer, peer links and readiness.
//! delivery owns checked buffers and blocked receives; readiness owns wait sets.
mod capabilities;
mod delivery;
pub(in crate::user_tasks) use delivery::Receive;
mod links;
pub(in crate::user_tasks) mod readiness;
use super::{Session, complete};
use crate::scheduler::Event;
use cathedral_contracts::user as abi;
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
            delivery::receive_call(
                session,
                slot,
                (first, second, abi::MAX_MESSAGE as u64, true),
                None,
                now,
                event,
            )
        }
        abi::IPC_HANDLE => session.ipc.handle(slot, first),
        abi::IPC_SEND => delivery::send(session, slot, first, second, third),
        abi::IPC_RECEIVE | abi::IPC_RECEIVE_UNTIL => {
            let deadline = (number == abi::IPC_RECEIVE_UNTIL).then_some(third);
            let capacity = if deadline.is_some() {
                abi::MAX_MESSAGE as u64
            } else {
                third
            };
            delivery::receive_call(
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

/// Complete readiness waits before retrying blocked copied-message receivers.
pub(in crate::user_tasks) fn wake_receivers(session: &mut Session) {
    readiness::wake(session);
    delivery::wake_receivers(session);
}
