// IPC request entrance: delivery, capability transfer, peer links and readiness.
// delivery owns checked buffers and blocked receives; readiness owns wait sets.
use super::{Session, complete};
use crate::scheduler::Event;
use cathedral_kernel_contracts::user as abi;
use delivery::ReceiveMode;
pub(in crate::user_tasks) fn dispatch(
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
        abi::IPC_SEND_CAP | abi::IPC_SEND_MEMORY => capabilities::send_cap(
            session,
            slot,
            first,
            second,
            third,
            number == abi::IPC_SEND_MEMORY,
        ),
        abi::IPC_RECEIVE_CAP | abi::IPC_RECEIVE_MEMORY | abi::IPC_TRY_RECEIVE_MEMORY => {
            if third != 0 {
                return Err(abi::INVALID_ARGUMENT);
            }
            delivery::receive_call(
                session,
                slot,
                (
                    first,
                    second,
                    abi::MAX_MESSAGE as u64,
                    if number == abi::IPC_RECEIVE_CAP {
                        ReceiveMode::Endpoint
                    } else if number == abi::IPC_TRY_RECEIVE_MEMORY {
                        ReceiveMode::TryMemory
                    } else {
                        ReceiveMode::Memory
                    },
                ),
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
                (first, second, capacity, ReceiveMode::Bytes),
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
                handles: core::array::from_fn(|i| match i {
                    0 => first,
                    1 => second,
                    _ => 0,
                }),
                kinds: [abi::WAIT_IPC; abi::MAX_WAIT_HANDLES],
                count: 2,
                deadline: third,
            },
            now,
            event,
        ),
        abi::IPC_WAIT_MANY => readiness::many(session, slot, [first, second, third], now, event),
        abi::EVENT_WAIT => readiness::mixed(session, slot, [first, second, third], now, event),
        abi::IPC_REVOKE => session.ipc.revoke(slot, first).map(|()| 0),
        _ => Err(abi::UNKNOWN),
    }
}
