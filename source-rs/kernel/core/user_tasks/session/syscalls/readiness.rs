//! Bounded endpoint readiness waits. It never consumes a queued message.
use super::{Session, complete};
use crate::{deadline, scheduler::Event};
use cathedral_contracts::user as abi;
#[derive(Clone, Copy)]
pub(in super::super) struct Wait {
    pub handles: [u64; abi::MAX_WAIT_HANDLES],
    pub count: usize,
    pub deadline: u64,
}
fn ready(session: &Session, slot: usize, wait: Wait, now: u64) -> Result<Option<u64>, u64> {
    if let Some(index) = session.ipc.ready_any(slot, &wait.handles[..wait.count])? {
        return Ok(Some(index));
    }
    if deadline::reached(now, wait.deadline) {
        Err(abi::TIMED_OUT)
    } else {
        Ok(None)
    }
}
pub(in super::super) fn syscall(
    session: &mut Session,
    slot: usize,
    wait: Wait,
    now: u64,
    event: &mut Event,
) -> Result<u64, u64> {
    session.clock.handle(slot)?;
    if !deadline::valid(now, wait.deadline) {
        return Err(abi::INVALID_ARGUMENT);
    }
    if let Some(index) = ready(session, slot, wait, now)? {
        return Ok(index);
    }
    session.tasks[slot].report.readiness_blocked += 1;
    session.tasks[slot].notify = Some(wait);
    *event = Event::Block;
    Ok(0)
}
pub(in super::super) fn wake(session: &mut Session) {
    for slot in 0..session.tasks.len() {
        let Some(wait) = session.tasks[slot].notify else {
            continue;
        };
        let result = match ready(session, slot, wait, cathedral_arch::ticks()) {
            Ok(None) => continue,
            Ok(Some(index)) => Ok(index),
            Err(error) => Err(error),
        };
        session.tasks[slot].notify = None;
        complete(session, slot, result);
        session.scheduler.unblock(slot);
    }
}

pub(in super::super) fn many(
    session: &mut Session,
    slot: usize,
    call: [u64; 3],
    now: u64,
    event: &mut Event,
) -> Result<u64, u64> {
    let [address, count, deadline] = call;
    if count == 0 || count > abi::MAX_WAIT_HANDLES as u64 {
        return Err(abi::INVALID_ARGUMENT);
    }
    let mut bytes = [0; abi::MAX_WAIT_HANDLES * 8];
    // SAFETY: Copy checked task-owned backing while running under the kernel root.
    if !unsafe {
        session.tasks[slot]
            .space
            .as_ref()
            .unwrap()
            .copy_from_user(address, &mut bytes[..count as usize * 8])
    } {
        return Err(abi::BAD_ADDRESS);
    }
    let handles =
        core::array::from_fn(|i| u64::from_le_bytes(bytes[i * 8..i * 8 + 8].try_into().unwrap()));
    syscall(
        session,
        slot,
        Wait {
            handles,
            count: count as usize,
            deadline,
        },
        now,
        event,
    )
}
