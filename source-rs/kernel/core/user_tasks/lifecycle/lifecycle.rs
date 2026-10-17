// Boot-context admission/reaping and grant teardown. No restart policy here.
use super::{Error, Exit, Frames, Session, admission, next_epoch, syscalls, taskcalls};
use crate::{scheduler::TaskState, user_tasks::Program, user_tasks::admission::pool::Code};
use cathedral_arch as arch;
use cathedral_kernel_contracts::user as abi;

pub(super) fn close(session: &mut Session, slot: usize) {
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
    if session
        .launches
        .iter()
        .any(|launch| launch.model.child == slot && launch.keyboard)
    {
        super::keyboard::reset(session);
    }
    session.ipc.close_task(slot);
    for launch in &mut session.launches {
        launch.model.close(slot);
        if launch.model.owner == slot {
            launch.images.fill(None);
            launch.initial = [0; abi::MAX_INITIAL_ENDPOINTS];
            launch.initial_count = 0;
            launch.delegated = [0; abi::MAX_LAUNCHES];
            launch.delegated_count = 0;
        }
    }
    syscalls::wake_receivers(session);
}

pub(super) unsafe fn reap(session: &mut Session, source: &mut Frames<'_>) {
    let mut requesters = 0u16;
    for index in 0..session.launches.len() {
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
    for slot in 0..session.tasks.len() {
        if session.scheduler.states()[slot] != TaskState::Exited {
            continue;
        }
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
        if let Some(launch) = session
            .launches
            .iter_mut()
            .find(|launch| launch.model.child == slot)
        {
            launch
                .model
                .reaped(outcome(session.tasks[slot].report.exit.unwrap()));
            session.tasks[launch.model.owner].report.reaped += 1;
        }
    }
    let owned: usize = session
        .tasks
        .iter()
        .filter(|task| task.space.is_some())
        .map(|task| task.report.frames)
        .sum();
    assert_eq!(
        source.allocated(),
        session.frame_baseline + owned + session.memory.frames()
    );
    assert_eq!(
        session.memory.funding.charged(),
        owned + session.memory.frames()
    );
    // Increasing owner/child slots made cancellation cascade in the first pass.
    // Return empty budgets only after every descendant address space is gone.
    for launch in &mut session.launches {
        let previous_owner = launch.model.owner;
        launch.model.reclaim();
        if launch.model.owner != previous_owner {
            session
                .pool
                .reattach(launch.model.child, launch.model.owner);
            session
                .scheduler
                .set_parent(launch.model.child, launch.model.owner);
        }
    }
    taskcalls::wake(session, arch::ticks());
    for owner in 0..session.tasks.len() {
        if requesters & (1 << owner) != 0 && session.tasks[owner].space.is_some() {
            syscalls::complete(session, owner, Ok(0));
            session.scheduler.unblock(owner);
        }
    }
}

pub(super) unsafe fn spawn(
    session: &mut Session,
    source: &mut Frames<'_>,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    executable: &Program<'_>,
    index: usize,
) {
    let model = &session.launches[index].model;
    let Some(argument) = model.pending() else {
        return;
    };
    let (owner, child, peer) = (model.owner, model.child, model.peer);
    if session.launches[index].code == Some(Code::Admitted) {
        // Object code was consumed by an earlier admission; its frames may be gone.
        session.launches[index].model.failed();
        syscalls::complete(session, owner, Err(abi::BAD_EXECUTABLE));
        session.scheduler.unblock(owner);
        return;
    }
    session.scheduler.account_cpu(arch::cpu_time_us());
    let initial = core::mem::take(&mut session.launches[index].initial);
    let initial_count = core::mem::take(&mut session.launches[index].initial_count);
    let delegated = core::mem::take(&mut session.launches[index].delegated);
    let delegated_count = core::mem::take(&mut session.launches[index].delegated_count);
    let handles = &initial[..initial_count];
    let baseline = source.allocated();
    // A blocked parent retains its rights through every fallible admission step.
    let plan = session
        .ipc
        .validate_initial(owner, child, handles)
        .and_then(|()| {
            let loans =
                delegation::Plan::prepare(session, owner, child, &delegated[..delegated_count])?;
            Ok((loans, grants::Plan::prepare(session, owner, child)?))
        });
    let result = if let Err(error) = plan {
        Err(error)
    } else if let Some(epoch) =
        next_epoch().filter(|epoch| *epoch <= crate::ipc::MAX_EPOCH - session.links.len() as u64)
    {
        let program = Program {
            executable: executable.executable,
            arguments: [executable.arguments[0], argument],
        };
        source.remaining = session.launches[index].frame_limit;
        // SAFETY: Boot context, sole CPU/IRQs off. Child slot is vacant; peer roots
        // stay live. All partial admission is rolled back before returning errors.
        match unsafe { admission::admit(session, layout, image, &program, source, child) } {
            Ok(()) => {
                // No user/trap can run between validation, address-space admission,
                // and this infallible commit (sole boot CPU, interrupts disabled).
                // Although admit marks Ready, execution resumes only after spawn returns.
                session
                    .ipc
                    .prepare_child_with(
                        session.launches[index].endpoint_base,
                        peer,
                        child,
                        epoch,
                        owner,
                        handles,
                    )
                    .expect("validated initial bundle changed during admission");
                let (loans, names) = plan.unwrap();
                let ticket = names.child;
                if let Some(device) = session.launches[index].device {
                    // SAFETY: Child admitted but not runnable; boot owns exclusive device custody.
                    unsafe {
                        device.reset();
                    }
                }
                names.commit(session, child);
                loans.commit(session, child);
                if session.launches[index].keyboard {
                    super::keyboard::reset(session);
                }
                let ticket = session.launches[index].model.started(ticket);
                session.tasks[owner].report.spawned += 1;
                if let Some(Code::Object { region, .. }) = session.launches[index].code {
                    // The image is copied; nothing keeps the object's frames needed.
                    session.memory.unref_later(region);
                    session.launches[index].code = Some(Code::Admitted);
                }
                Ok(ticket)
            }
            Err(error) => Err(match error {
                Error::Executable(_) => abi::BAD_EXECUTABLE,
                Error::Memory(arch::MemoryError::OutOfFrames) | Error::OutOfHeap => abi::NO_MEMORY,
                _ => abi::INVALID_ARGUMENT,
            }),
        }
    } else {
        Err(abi::NO_MEMORY)
    };
    if result.is_err() {
        assert_eq!(source.allocated(), baseline);
        session.launches[index].model.failed();
    }
    syscalls::complete(session, owner, result);
    session.scheduler.unblock(owner);
    session.scheduler.charge_cpu(owner, arch::cpu_time_us());
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
