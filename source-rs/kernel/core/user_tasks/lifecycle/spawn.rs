// Admission entrance: validate current authority, advance private construction,
// then revalidate/reserve grants and publish atomically. No partial child runs.
use super::{Frames, Session, abi, delegation, grants};
use crate::user_tasks::admission::pool::Code;
use crate::user_tasks::execution::deferred::Work;
use crate::user_tasks::{Error, Program, admission, next_epoch};
use cathedral_arch as arch;

pub(in crate::user_tasks) unsafe fn spawn(
    session: &mut Session,
    source: &mut Frames<'_>,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    executable: &Program<'_>,
    index: usize,
) {
    let Some(argument) = session.launches[index].model.pending() else {
        return;
    };
    let owner = session.launches[index].model.owner;
    session.scheduler.account_cpu(arch::cpu_time_us());
    let result = if let Some(error) = session.launches[index].refusal {
        // SAFETY: A refused draft was never runnable; all its charged frames
        // remain in the private ledger. No authority/source is returned mid-step.
        if unsafe { admission::abort_step(session, source, index, super::reclamation::FRAME_BATCH) }
        {
            session.launches[index].refusal = None;
            Err(error)
        } else {
            Ok(None)
        }
    } else {
        // SAFETY: Boot stack, exclusive inactive roots, pinned executable and frame custody.
        match unsafe { progress(session, source, layout, image, executable, argument, index) } {
            Err(error) if session.launches[index].draft.is_some() => {
                // Finish this visit at the construction/validation boundary.
                // Rollback uses later turns; the first error remains definitive.
                session.launches[index].refusal = Some(error);
                Ok(None)
            }
            result => result,
        }
    };
    match result {
        Ok(None) => session.deferred.insert(Work::Admission, index),
        finished => {
            let result = finished.map(|ticket| ticket.unwrap());
            if result.is_err() {
                assert!(session.launches[index].draft.is_none());
                session.launches[index].model.failed();
            }
            session.launches[index].initial.fill(0);
            session.launches[index].initial_count = 0;
            session.launches[index].delegated.fill(0);
            session.launches[index].delegated_count = 0;
            super::syscalls::complete(session, owner, result);
            session.scheduler.unblock(owner);
        }
    }
    session.scheduler.charge_cpu(owner, arch::cpu_time_us());
}

unsafe fn progress(
    session: &mut Session,
    source: &mut Frames<'_>,
    layout: &arch::BootLayout,
    image: arch::ImageRange,
    executable: &Program<'_>,
    argument: u64,
    index: usize,
) -> Result<Option<u64>, u64> {
    let launch = &session.launches[index];
    let (owner, child, peer) = (launch.model.owner, launch.model.child, launch.model.peer);
    launch.model.admission_ready()?;
    if launch.code == Some(Code::Admitted) {
        return Err(abi::BAD_EXECUTABLE);
    }
    let initial = launch.initial;
    let handles = &initial[..launch.initial_count];
    session.ipc.validate_initial(owner, child, handles)?;
    // SAFETY: Source remains held by this Starting launch across all steps. A
    // draft root stays outside task/context storage until final publication.
    if !unsafe { admission::advance(session, layout, image, executable, source, index) }
        .map_err(error)?
    {
        return Ok(None);
    }

    // Every fallible authority operation is performed now, after the last yield.
    // Nothing may interrupt between these checks and the infallible commit below.
    let delegated = session.launches[index].delegated;
    let loans = delegation::Plan::prepare(
        session,
        owner,
        child,
        &delegated[..session.launches[index].delegated_count],
    )?;
    let names = grants::Plan::prepare(session, owner, child)?;
    let epoch = next_epoch()
        .filter(|epoch| *epoch <= crate::ipc::MAX_EPOCH - session.links.len() as u64)
        .ok_or(abi::NO_MEMORY)?;
    admission::publish(session, index, [executable.arguments[0], argument]);
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
        .expect("validated bundle changed during commit");
    let ticket = names.child;
    if let Some(device) = session.launches[index].device {
        // SAFETY: Child cannot execute until this commit returns; exclusive device custody.
        unsafe {
            device.reset();
        }
    }
    names.commit(session, child);
    loans.commit(session, child);
    if session.launches[index].keyboard {
        super::super::keyboard::reset(session);
    }
    let ticket = session.launches[index].model.started(ticket);
    session.tasks[owner].report.spawned += 1;
    if let Some(Code::Object { region, .. }) = session.launches[index].code {
        session.memory.unref_later(region);
        session.launches[index].code = Some(Code::Admitted);
    }
    Ok(Some(ticket))
}

fn error(error: Error) -> u64 {
    match error {
        Error::Executable(_) => abi::BAD_EXECUTABLE,
        Error::Memory(arch::MemoryError::OutOfFrames) | Error::OutOfHeap => abi::NO_MEMORY,
        _ => abi::INVALID_ARGUMENT,
    }
}
