//! Validate rights and complete records before effects; then register or finish
//! one notification wait. Copied IPC is used only for transitional grant handoff.
use super::{Session, abi, capabilities, complete, delivery};
use crate::{
    scheduler::Event,
    user_tasks::{execution::waits::Kind, time},
};

pub(in crate::user_tasks) fn dispatch(
    session: &mut Session,
    slot: usize,
    [number, a, b, c]: [u64; 4],
    now: u64,
    event: &mut Event,
) -> Result<u64, u64> {
    if c != 0 && number != abi::NOTIFY_WAIT {
        return Err(abi::INVALID_ARGUMENT);
    }
    match number {
        abi::NOTIFY_CREATE | abi::NOTIFY_BUDGET => {
            if b != 0 {
                return Err(abi::INVALID_ARGUMENT);
            }
            let size = if number == abi::NOTIFY_CREATE { 24 } else { 32 };
            let space = session.tasks[slot].space.as_ref().unwrap();
            if !space.writable_range(a, size) {
                return Err(abi::BAD_ADDRESS);
            }
            let words = if number == abi::NOTIFY_CREATE {
                let [wait, signal, control] = session.ipc.create_notification(slot)?;
                [wait, signal, control, 0]
            } else {
                session.ipc.notification_budget(slot)
            };
            let mut bytes = [0; 32];
            for (word, out) in words.iter().zip(bytes.chunks_exact_mut(8)) {
                out.copy_from_slice(&word.to_le_bytes());
            }
            // SAFETY: Entire destination checked before creating any grants; kernel serialized.
            assert!(unsafe { space.copy_to_user(a, &bytes[..size]) });
            Ok(0)
        }
        abi::NOTIFY_DROP => {
            if b != 0 {
                return Err(abi::INVALID_ARGUMENT);
            }
            session.ipc.notifications.drop_handle(slot, a).map(|()| 0)
        }
        abi::NOTIFY_SIGNAL => {
            if b > 1 {
                return Err(abi::INVALID_ARGUMENT);
            }
            session
                .ipc
                .notifications
                .signal(slot, a, b == 1)
                .map(|()| 0)
        }
        abi::NOTIFY_CONTROL => session.ipc.notifications.control(slot, a, b).map(|()| 0),
        abi::NOTIFY_WAIT | abi::NOTIFY_SIGNAL_WAIT => {
            let (signal, handle, until, timed) = if number == abi::NOTIFY_WAIT {
                (None, a, b, c)
            } else {
                if b != 0 {
                    return Err(abi::INVALID_ARGUMENT);
                }
                let mut bytes = [0; 32];
                // SAFETY: Checked caller backing; no pointer survives this syscall.
                if !unsafe {
                    session.tasks[slot]
                        .space
                        .as_ref()
                        .unwrap()
                        .copy_from_user(a, &mut bytes)
                } {
                    return Err(abi::BAD_ADDRESS);
                }
                let words: [u64; 4] = core::array::from_fn(|i| {
                    u64::from_le_bytes(bytes[i * 8..i * 8 + 8].try_into().unwrap())
                });
                (Some(words[0]), words[1], words[2], words[3])
            };
            if timed > 1 || timed == 0 && until != 0 {
                return Err(abi::INVALID_ARGUMENT);
            }
            let deadline = (timed == 1).then_some(until);
            if let Some(d) = deadline {
                session.clock.handle(slot)?;
                if !time::valid(now, d) {
                    return Err(abi::INVALID_ARGUMENT);
                }
            }
            let outcome = if let Some(signal) = signal {
                session
                    .ipc
                    .notifications
                    .signal_and_wait(slot, signal, handle, deadline, now)?
            } else {
                session
                    .ipc
                    .notifications
                    .wait(slot, handle, deadline, now)?
            };
            if let Some(outcome) = outcome {
                return Ok(outcome);
            }
            session
                .waits
                .register(slot, Kind::Notification, deadline, now);
            *event = Event::Block;
            Ok(0)
        }
        abi::NOTIFY_SEND => capabilities::send_cap(session, slot, a, b, c, 3),
        abi::NOTIFY_RECEIVE | abi::NOTIFY_TRY_RECEIVE => delivery::receive_call(
            session,
            slot,
            (
                a,
                b,
                abi::MAX_MESSAGE as u64,
                if number == abi::NOTIFY_TRY_RECEIVE {
                    delivery::ReceiveMode::TryNotification
                } else {
                    delivery::ReceiveMode::Notification
                },
            ),
            None,
            now,
            event,
        ),
        _ => Err(abi::UNKNOWN),
    }
}
pub(in crate::user_tasks) fn wake(session: &mut Session, slot: usize, now: u64) {
    #[cfg(feature = "memory-lab")]
    let deadline = session.ipc.notifications.deadline(slot);
    let Some(result) = session.ipc.notifications.complete(slot, now) else {
        return;
    };
    session.waits.remove(slot);
    complete(session, slot, Ok(result));
    session.scheduler.unblock(slot);
    #[cfg(feature = "memory-lab")]
    if session.scheduler.states()[slot] == crate::scheduler::TaskState::Ready {
        session
            .timing
            .wait_ready(slot, Kind::Notification, deadline, now);
    }
}
