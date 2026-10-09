//! Boot-context admission/reaping and grant teardown. No restart policy here.
use super::{
    Error, Exit, Frames, Session, Supervision, admission, dispatch, next_epoch, taskcalls,
};
use crate::{scheduler::TaskState, users::Program};
use cathedral_arch as arch;
use cathedral_contracts::user as abi;

pub(super) fn close(session: &mut Session, slot: usize) {
    session.tasks[slot].receive = None;
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
    }
    dispatch::wake_receivers(session);
}

pub(super) unsafe fn reap(session: &mut Session, source: &mut Frames<'_>) {
    let mut requesters = 0u8;
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
        // SAFETY: No context can resume this task. Kernel root, IRQs off.
        unsafe {
            admission::retire(&mut session.tasks[slot], source);
        }
        session.scheduler.reap(slot);
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
    assert_eq!(source.frames.allocated(), session.frame_baseline + owned);
    taskcalls::wake(session, arch::ticks());
    for owner in 0..session.tasks.len() {
        if requesters & (1 << owner) != 0 {
            dispatch::complete(session, owner, Ok(0));
            session.scheduler.unblock(owner);
        }
    }
}

pub(super) unsafe fn spawn(
    session: &mut Session,
    source: &mut Frames<'_>,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    launch: &Supervision<'_>,
    index: usize,
) {
    let model = &session.launches[index].model;
    let Some(argument) = model.pending() else {
        return;
    };
    let (owner, child, peer) = (model.owner, model.child, model.peer);
    let baseline = source.frames.allocated();
    let result = if let Some(epoch) = next_epoch() {
        let program = Program {
            executable: launch.program.executable,
            arguments: [launch.program.arguments[0], argument],
        };
        source.remaining = launch.frame_limit;
        // SAFETY: Boot context, sole CPU/IRQs off. Child slot is vacant; peer roots
        // stay live. All partial admission is rolled back before returning errors.
        match unsafe { admission::admit(session, layout, image, &program, source, child) } {
            Ok(()) => {
                session.ipc.prepare_child(
                    session.launches[index].endpoint_base,
                    peer,
                    child,
                    epoch,
                );
                if launch.keyboard {
                    super::keyboard::reset(session);
                }
                let ticket = session.launches[index].model.started(epoch);
                session.tasks[owner].report.spawned += 1;
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
        assert_eq!(source.frames.allocated(), baseline);
        session.launches[index].model.failed();
    }
    dispatch::complete(session, owner, result);
    session.scheduler.unblock(owner);
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
