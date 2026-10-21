// Wait completion entrance: gather changed event owners and expired deadlines,
// then revalidate readiness, copied receives, and child outcomes in that order.
// A hint never authorizes a completion: syscall owners retain authority checks,
// generation validation, checked copyout, and ready/terminal-before-timeout rules.
use crate::user_tasks::execution::timing::{self, Phase};
use crate::user_tasks::{Session, syscalls};

pub(super) const COMPLETION_BATCH: usize = 8;

pub(in crate::user_tasks) fn wake(session: &mut Session, now: u64) {
    let start = timing::start();
    let ipc = session.ipc.take_changed_receivers();
    let devices = session.devices.take_changed();
    let [receivers, readiness, children] = session.waits.batch(now, ipc, devices);
    for slot in slots(readiness) {
        syscalls::ipc::readiness::wake(session, slot, now);
    }
    for slot in slots(receivers) {
        syscalls::ipc::delivery::wake(session, slot, now);
    }
    for slot in slots(children) {
        syscalls::tasks::wake(session, slot, now);
    }
    session.timing.record(Phase::WaitCompletion, start);
}

fn slots(mut pending: u32) -> impl Iterator<Item = usize> {
    core::iter::from_fn(move || {
        if pending == 0 {
            return None;
        }
        let slot = pending.trailing_zeros() as usize;
        pending &= pending - 1;
        Some(slot)
    })
}
