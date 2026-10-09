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
    session.ipc.close_task(slot);
    if let Some(model) = &mut session.supervisor {
        model.close(slot);
    }
    dispatch::wake_receivers(session);
}

pub(super) unsafe fn reap(session: &mut Session, source: &mut Frames<'_>) {
    if let Some(model) = &session.supervisor
        && model.cancel_child()
        && session.scheduler.states()[model.child] != TaskState::Exited
    {
        let slot = model.child;
        session.scheduler.cancel(slot);
        session.tasks[slot].report.exit = Some(Exit::Cancelled);
        session.completed += 1;
        session.tasks[slot].report.completion_order = session.completed;
        session.tasks[model.owner].report.cancelled += 1;
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
        if let Some(model) = &mut session.supervisor
            && model.child == slot
        {
            model.reaped(outcome(session.tasks[slot].report.exit.unwrap()));
            session.tasks[model.owner].report.reaped += 1;
        }
    }
    let owned: usize = session
        .tasks
        .iter()
        .filter(|task| task.space.is_some())
        .map(|task| task.report.frames)
        .sum();
    assert_eq!(source.frames.allocated(), session.frame_baseline + owned);
    taskcalls::wake(session);
}

pub(super) unsafe fn spawn(
    session: &mut Session,
    source: &mut Frames<'_>,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    launch: &Supervision<'_>,
) {
    let Some(model) = &session.supervisor else {
        return;
    };
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
                session
                    .ipc
                    .prepare_child(session.endpoint_base, peer, child, epoch);
                let ticket = session.supervisor.as_mut().unwrap().started(epoch);
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
        session.supervisor.as_mut().unwrap().failed();
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
