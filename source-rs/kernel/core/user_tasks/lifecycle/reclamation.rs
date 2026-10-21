//! Retire backing incrementally; publish outcomes and return loans only when the
//! finite cleanup wave is empty. Admission is held while this wave is pending.
use super::{Exit, Frames, Session, abi, admission, arch, close, syscalls};
use crate::scheduler::TaskState;
use crate::user_tasks::execution::deferred::Work;
pub(super) const FRAME_BATCH: usize = 8;

pub(super) fn withdraw(session: &mut Session) {
    while let Some(index) = session.deferred.first(Work::Cancel) {
        let model = &session.launches[index].model;
        let (slot, owner) = (model.child, model.owner);
        if model.cancellation_pending() {
            session.cancel_waiters |= 1 << owner;
        }
        if !model.cancel_child() || session.scheduler.states()[slot] == TaskState::Exited {
            continue;
        }
        match session.scheduler.states()[slot] {
            TaskState::Ready => session.tasks[owner].report.cancelled_ready += 1,
            TaskState::Blocked => session.tasks[owner].report.cancelled_blocked += 1,
            _ => {}
        }
        session.scheduler.cancel(slot);
        session.tasks[slot].report.exit = Some(Exit::Cancelled);
        session.completed += 1;
        session.tasks[slot].report.completion_order = session.completed;
        session.tasks[owner].report.cancelled += 1;
        close(session, slot);
    }
}

pub(super) unsafe fn retire_step(session: &mut Session, source: &mut Frames<'_>, slot: usize) {
    assert_eq!(session.scheduler.states()[slot], TaskState::Exited);
    session.scheduler.account_cpu(arch::cpu_time_us());
    let mut cursor = core::mem::take(&mut session.tasks[slot].closing);
    // SAFETY: No dead task can resume or acquire new aliases while closing.
    let aliases_gone =
        unsafe { super::super::memory::close_step(session, slot, source, &mut cursor) };
    session.tasks[slot].closing = cursor;
    let mut done = false;
    if aliases_gone {
        let space = session.tasks[slot].space.as_mut().unwrap();
        let before = space.frame_count();
        // SAFETY: All borrowed aliases retired; root is permanently inactive.
        done = unsafe { space.release_pages(source, FRAME_BATCH) };
        session
            .memory
            .funding
            .refund(slot, before - space.frame_count());
        if done {
            session.tasks[slot].space = None;
        }
    }
    session
        .deferred
        .insert(if done { Work::Collect } else { Work::Retire }, slot);
    session.scheduler.charge_cpu(slot, arch::cpu_time_us());
}

pub(super) unsafe fn abandon_step(session: &mut Session, source: &mut Frames<'_>, index: usize) {
    let owner = session.launches[index].model.owner;
    session.scheduler.account_cpu(arch::cpu_time_us());
    // SAFETY: Dead owner; draft was never installed or exposed to a user context.
    if !unsafe { admission::abort_step(session, source, index, FRAME_BATCH) } {
        session.deferred.insert(Work::Abandon, index);
    }
    session.scheduler.charge_cpu(owner, arch::cpu_time_us());
}

pub(super) fn collect(session: &mut Session) {
    assert!(!session.deferred.has(Work::Retire) && !session.deferred.has(Work::Abandon));
    // Publish all completed task outcomes before returning their withdrawn loans.
    while let Some(slot) = session.deferred.first(Work::Collect) {
        session.scheduler.account_cpu(arch::cpu_time_us());
        session.scheduler.reap(slot);
        let mut ancestor = session.scheduler.parent(slot).unwrap_or(0);
        while ancestor != 0
            && (session.tasks[ancestor].space.is_none()
                || session.scheduler.states()[ancestor] == TaskState::Exited)
        {
            ancestor = session.scheduler.parent(ancestor).unwrap_or(0);
        }
        session.pool.reclaim(slot, ancestor);
        if let Some(index) = session.child_launch[slot] {
            let launch = &mut session.launches[index];
            launch
                .model
                .reaped(outcome(session.tasks[slot].report.exit.unwrap()));
            session.tasks[launch.model.owner].report.reaped += 1;
            session.waits.child_changed(index);
        }
        session.scheduler.charge_cpu(slot, arch::cpu_time_us());
    }
    while let Some(index) = session.deferred.first(Work::Return) {
        let launch = &mut session.launches[index];
        let previous_owner = launch.model.owner;
        launch.model.reclaim();
        if launch.model.launch(launch.model.owner).is_err()
            && let Some(admission::pool::Code::Object { region, .. }) = launch.code
        {
            // No surviving lender can use this launch. Its source reference
            // must not pin backing after the unpublished draft was withdrawn.
            session.memory.unref_later(region);
            launch.code = None;
        }
        if launch.model.owner != previous_owner {
            session
                .pool
                .reattach(launch.model.child, launch.model.owner);
            session
                .scheduler
                .set_parent(launch.model.child, launch.model.owner);
        }
    }
    let requesters = core::mem::take(&mut session.cancel_waiters);
    crate::user_tasks::execution::waits::wake(session, arch::ticks());
    for owner in 0..session.tasks.len() {
        if requesters & (1 << owner) != 0 && session.tasks[owner].space.is_some() {
            syscalls::complete(session, owner, Ok(0));
            session.scheduler.unblock(owner);
        }
    }
}

pub(super) fn conserved(session: &Session, source: &Frames<'_>) {
    let owned: usize = session
        .tasks
        .iter()
        .filter(|task| task.space.is_some())
        .map(|task| task.space.as_ref().unwrap().frame_count())
        .sum::<usize>()
        + session
            .launches
            .iter()
            .filter_map(|launch| launch.draft.as_ref())
            .map(admission::Draft::frames)
            .sum::<usize>();
    assert_eq!(
        source.allocated(),
        session.frame_baseline + owned + session.memory.frames()
    );
    assert_eq!(
        session.memory.funding.charged(),
        owned + session.memory.frames() + session.memory.reserved_unbacked()
    );
}

fn outcome(exit: Exit) -> [u8; abi::EXIT_BYTES] {
    let words = match exit {
        Exit::Returned(status) => [abi::EXIT_RETURNED, status, 0, 0, 0],
        Exit::Fault(fault) => [
            abi::EXIT_FAULT,
            fault.vector,
            fault.error,
            fault.address,
            fault.instruction,
        ],
        Exit::Cancelled => [abi::EXIT_CANCELLED, 0, 0, 0, 0],
    };
    let mut bytes = [0; abi::EXIT_BYTES];
    for (word, output) in words.into_iter().zip(bytes.chunks_exact_mut(8)) {
        output.copy_from_slice(&word.to_le_bytes());
    }
    bytes
}
