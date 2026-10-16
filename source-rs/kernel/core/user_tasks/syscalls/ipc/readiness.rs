//! Bounded IPC/device readiness waits. Observing readiness never consumes an event.
use crate::user_tasks::syscalls::{Session, complete};
use crate::{scheduler::Event, user_tasks::time as deadline};
use cathedral_kernel_contracts::user as abi;
#[derive(Clone, Copy)]
pub(in crate::user_tasks) struct Wait {
    pub handles: [u64; abi::MAX_WAIT_HANDLES],
    pub kinds: [u64; abi::MAX_WAIT_HANDLES],
    pub count: usize,
    pub deadline: u64,
}
fn ready(session: &Session, slot: usize, wait: Wait, now: u64) -> Result<Option<u64>, u64> {
    wait.select(now, |kind, handle| match kind {
        abi::WAIT_IPC => session.ipc.ready(slot, handle),
        abi::WAIT_DEVICE => session.devices.ready(slot, handle),
        _ => Err(abi::INVALID_ARGUMENT),
    })
}
impl Wait {
    fn select(
        self,
        now: u64,
        mut ready: impl FnMut(u64, u64) -> Result<bool, u64>,
    ) -> Result<Option<u64>, u64> {
        let mut first = None;
        for i in 0..self.count {
            // Do not return early: even a ready first entry cannot hide an invalid
            // later capability. No events are consumed on validation failure.
            if ready(self.kinds[i], self.handles[i])? && first.is_none() {
                first = Some(i as u64);
            }
        }
        if first.is_some() {
            Ok(first)
        } else if deadline::reached(now, self.deadline) {
            Err(abi::TIMED_OUT)
        } else {
            Ok(None)
        }
    }
}
pub(in crate::user_tasks) fn syscall(
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
pub(in crate::user_tasks) fn wake(session: &mut Session) {
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

pub(in crate::user_tasks) fn many(
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
            kinds: [abi::WAIT_IPC; abi::MAX_WAIT_HANDLES],
            count: count as usize,
            deadline,
        },
        now,
        event,
    )
}

pub(in crate::user_tasks) fn mixed(
    session: &mut Session,
    slot: usize,
    call: [u64; 3],
    now: u64,
    event: &mut Event,
) -> Result<u64, u64> {
    let [address, count, deadline] = call;
    if count > abi::MAX_WAIT_HANDLES as u64 {
        return Err(abi::INVALID_ARGUMENT);
    }
    let mut bytes = [0; abi::MAX_WAIT_HANDLES * 16];
    // An empty interest set has no user buffer and waits solely for its deadline.
    if count != 0
        // SAFETY: Copy checked task-owned backing under the kernel root.
        && !unsafe {
            session.tasks[slot]
                .space
                .as_ref()
                .unwrap()
                .copy_from_user(address, &mut bytes[..count as usize * 16])
        }
    {
        return Err(abi::BAD_ADDRESS);
    }
    let word = |offset| u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap());
    syscall(
        session,
        slot,
        Wait {
            handles: core::array::from_fn(|i| word(i * 16 + 8)),
            kinds: core::array::from_fn(|i| word(i * 16)),
            count: count as usize,
            deadline,
        },
        now,
        event,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn whole_set_validation_and_ready_before_deadline() {
        let mut wait = Wait {
            handles: [7; 8],
            kinds: [abi::WAIT_IPC; 8],
            count: 2,
            deadline: 10,
        };
        wait.kinds[1] = abi::WAIT_DEVICE;
        assert_eq!(
            wait.select(10, |kind, _| if kind == abi::WAIT_DEVICE {
                Err(abi::BAD_HANDLE)
            } else {
                Ok(true)
            }),
            Err(abi::BAD_HANDLE)
        );
        assert_eq!(wait.select(10, |_, _| Ok(true)), Ok(Some(0)));
        assert_eq!(wait.select(9, |_, _| Ok(false)), Ok(None));
        assert_eq!(wait.select(10, |_, _| Ok(false)), Err(abi::TIMED_OUT));
        wait.count = 0;
        assert_eq!(
            wait.select(10, |_, _| panic!("no interests")),
            Err(abi::TIMED_OUT)
        );
    }
}
