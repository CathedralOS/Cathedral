// Boot-context admission/reaping and grant teardown. No restart policy here.
use super::{Exit, Frames, Session, admission, syscalls};
use crate::scheduler::TaskState;
use crate::user_tasks::execution::deferred::Work;
use cathedral_arch as arch;
use cathedral_kernel_contracts::user as abi;

pub(super) fn close(session: &mut Session, slot: usize) {
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
            launch.images.fill(None);
            launch.initial = [0; abi::MAX_INITIAL_ENDPOINTS];
            launch.initial_count = 0;
            launch.delegated = [0; abi::MAX_LAUNCHES];
            launch.delegated_count = 0;
        }
    }
    super::execution::waits::wake(session, arch::ticks());
}

pub(super) unsafe fn reap(session: &mut Session, source: &mut Frames<'_>) {
    if !session.deferred.has(Work::Cancel)
        && !session.deferred.has(Work::Retire)
        && !session.deferred.has(Work::Return)
    {
        return;
    }
    let mut requesters = 0u32;
    while let Some(index) = session.deferred.first(Work::Cancel) {
        let model = &session.launches[index].model;
        let (slot, owner) = (model.child, model.owner);
        if model.cancellation_pending() {
            requesters |= 1 << owner;
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
    while let Some(slot) = session.deferred.first(Work::Retire) {
        assert_eq!(session.scheduler.states()[slot], TaskState::Exited);
        session.scheduler.account_cpu(arch::cpu_time_us());
        // SAFETY: No context can resume this task. Kernel root, IRQs off.
        unsafe {
            super::memory::close(session, slot, source);
            admission::retire(session, slot, source);
        }
        session.scheduler.reap(slot);
        session.scheduler.charge_cpu(slot, arch::cpu_time_us());
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
    }
    // Increasing owner/child slots made cancellation cascade in the first pass.
    // Return empty budgets only after every descendant address space is gone.
    while let Some(index) = session.deferred.first(Work::Return) {
        let owner = session.launches[index].model.owner;
        session.scheduler.account_cpu(arch::cpu_time_us());
        // SAFETY: Owner died before commit; no draft context or rights were published.
        unsafe {
            admission::abort(session, source, index);
        }
        session.scheduler.charge_cpu(owner, arch::cpu_time_us());
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
    let owned: usize = session
        .tasks
        .iter()
        .filter(|task| task.space.is_some())
        .map(|task| task.report.frames)
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
    super::execution::waits::wake(session, arch::ticks());
    for owner in 0..session.tasks.len() {
        if requesters & (1 << owner) != 0 && session.tasks[owner].space.is_some() {
            syscalls::complete(session, owner, Ok(0));
            session.scheduler.unblock(owner);
        }
    }
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
