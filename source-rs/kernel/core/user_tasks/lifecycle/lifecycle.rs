// Boot-context admission/reaping and grant teardown. No restart policy here.
use super::{Exit, Frames, Session, admission, syscalls};
use crate::user_tasks::execution::deferred::Work;
use crate::user_tasks::execution::timing::{self, Phase as TimedPhase};
use cathedral_arch as arch;
use cathedral_kernel_contracts::user as abi;

pub(super) fn close(session: &mut Session, slot: usize) {
    let start = timing::start();
    withdrawal::task(session, slot);
    super::execution::waits::wake(session, arch::ticks());
    session.timing.record(TimedPhase::Close, start);
}

/// Advance physical cleanup once after withdrawing all newly canceled authority.
/// # Safety
/// Sole boot CPU, kernel root, IRQs off; no dead context can resume.
pub(super) unsafe fn reap(session: &mut Session, source: &mut Frames<'_>) {
    if !session.deferred.reclaiming() {
        return;
    }
    if session.deferred.has(Work::Cancel) {
        let start = timing::start();
        reclamation::withdraw(session);
        session.timing.record(TimedPhase::Withdraw, start);
    }
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
