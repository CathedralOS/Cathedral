//! A bounded two-endpoint readiness wait. It never consumes a queued message.
use super::{Session, complete};
use crate::{deadline, scheduler::Event};
use cathedral_contracts::user as abi;
#[derive(Clone, Copy)]
pub(in super::super) struct Wait {
    pub handles: [u64; 2],
    pub deadline: u64,
}
fn ready(session: &Session, slot: usize, wait: Wait, now: u64) -> Result<Option<u64>, u64> {
    for (index, handle) in wait.handles.into_iter().enumerate() {
        if session.ipc.ready(slot, handle)? {
            return Ok(Some(index as u64));
        }
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
    // Validate both grants even when the first is already ready.
    for handle in wait.handles {
        session
            .ipc
            .check(slot, handle, crate::ipc::Right::Receive)?;
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
