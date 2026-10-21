//! Withdraw one stopped task and enqueue its descendants before any waiter retry.
use super::{Session, Work, abi};

pub(super) fn task(session: &mut Session, slot: usize) {
    session.cancel_waiters &= !(1 << slot);
    session.waits.remove(slot);
    session.deferred.remove(Work::Memory, slot);
    session.deferred.insert(Work::Retire, slot);
    crate::user_tasks::syscalls::devices::leased::close(session, slot);
    for region in session.executables.close(slot).into_iter().flatten() {
        session.memory.unref_later(region);
    }
    session.clock.install(slot, 0);
    session.tasks[slot].memory_call = None;
    session.tasks[slot].receive = None;
    session.tasks[slot].notify = None;
    for link in &mut session.links {
        link.model.close(slot);
    }
    session.tasks[slot].wait = None;
    session.tasks[slot].keyboard_wait = false;
    session.tasks[slot].keyboard_deadline = None;
    if session.keyboard_owner == Some(slot) {
        crate::user_tasks::keyboard::reset(session);
    }
    session.ipc.close_task(slot);
    for (index, launch) in session.launches.iter_mut().enumerate() {
        launch.model.close(slot);
        if launch.model.owner == slot {
            launch.refusal = None;
            if launch.model.cancel_child() {
                session.deferred.insert(Work::Cancel, index);
            }
            session.deferred.remove(Work::Admission, index);
            session.deferred.insert(Work::Return, index);
            if launch.draft.is_some() {
                session.deferred.insert(Work::Abandon, index);
            }
            launch.images.fill(None);
            launch.initial = [0; abi::MAX_INITIAL_ENDPOINTS];
            launch.initial_count = 0;
            launch.delegated = [0; abi::MAX_LAUNCHES];
            launch.delegated_count = 0;
        }
    }
}
