// Boot-context admission/reaping and grant teardown. No restart policy here.
use super::{Exit, Frames, Session, admission, syscalls};
use crate::user_tasks::execution::deferred::Work;
use crate::user_tasks::execution::timing::{self, Phase as TimedPhase};
use cathedral_arch as arch;
use cathedral_kernel_contracts::user as abi;

pub(super) fn close(session: &mut Session, slot: usize) {
    session.cancel_waiters &= !(1 << slot);
    session.waits.remove(slot);
    session.deferred.remove(Work::Memory, slot);
    session.deferred.insert(Work::Retire, slot);
    super::syscalls::devices::leased::close(session, slot);
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
        super::keyboard::reset(session);
    }
    session.ipc.close_task(slot);
    for (index, launch) in session.launches.iter_mut().enumerate() {
        launch.model.close(slot);
        if launch.model.owner == slot {
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
    super::execution::waits::wake(session, arch::ticks());
}

/// Advance physical cleanup once after withdrawing all newly canceled authority.
/// # Safety
/// Sole boot CPU, kernel root, IRQs off; no dead context can resume.
pub(super) unsafe fn reap(session: &mut Session, source: &mut Frames<'_>) {
    if !session.deferred.reclaiming() {
        return;
    }
    let start = timing::start();
    reclamation::withdraw(session);
    session.timing.record(TimedPhase::Withdraw, start);
    let start = timing::start();
    // SAFETY: Withdrawal has stopped every affected descendant before any yield.
    unsafe {
        if let Some(slot) = session.deferred.next(Work::Retire) {
            reclamation::retire_step(session, source, slot);
            session.timing.record(TimedPhase::Retire, start);
        } else if let Some(index) = session.deferred.next(Work::Abandon) {
            reclamation::abandon_step(session, source, index);
            session.timing.record(TimedPhase::Abandon, start);
        } else {
            reclamation::collect(session);
            session.timing.record(TimedPhase::Collect, start);
        }
    }
    let start = timing::start();
    reclamation::conserved(session, source);
    session.timing.record(TimedPhase::Conservation, start);
}
