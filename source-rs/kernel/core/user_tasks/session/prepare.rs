//! Allocate every arena before publishing a task or entering a trap callback.
use super::{Config, Error, Report, Session, admission, composition};
use crate::scheduler::Scheduler;
use alloc::vec::Vec;
use cathedral_arch::Context;
pub(super) fn session(
    initial_count: usize,
    count: usize,
    config: &Config<'_>,
    baseline: usize,
    output: fn(&[u8]) -> bool,
) -> Result<(Session, Vec<Report>), Error> {
    let grants = composition::grants(initial_count, count, config)?;
    let mut scheduler = Scheduler::new(count);
    if !scheduler.configure_cpu(config.cpu_limits) {
        return Err(Error::InvalidCpuBudget);
    }
    for launch in &grants.launches {
        scheduler.set_parent(launch.model.child, launch.model.owner);
    }
    let memory =
        super::memory::Memory::new(count, config.dynamic_tasks, config.memory).map_err(|code| {
            if code == cathedral_contracts::user::NO_MEMORY {
                Error::OutOfHeap
            } else {
                Error::InvalidEndpoints
            }
        })?;
    let mut session = Session {
        pool: crate::admission_pool::Pool::new(count - config.dynamic_tasks, count),
        executables: crate::admission_pool::Executables::new(count)
            .map_err(|_| Error::OutOfHeap)?,
        names: grants.names,
        memory,
        tasks: Vec::new(),
        scheduler,
        boot: Context::default(),
        output,
        completed: 0,
        ipc: grants.ipc,
        launches: grants.launches,
        links: grants.links,
        keyboard: crate::byte_queue::ByteQueue::new(),
        frame_baseline: baseline,
        clock: grants.clock,
    };
    for index in 0..config.executables.len() {
        let name = session.names.mint(0).map_err(|_| Error::OutOfHeap)?;
        session
            .executables
            .install(0, config.supervision.len() + index, name)
            .map_err(|_| Error::OutOfHeap)?;
    }
    session
        .tasks
        .try_reserve_exact(count)
        .map_err(|_| Error::OutOfHeap)?;
    let mut reports = Vec::new();
    reports
        .try_reserve_exact(count)
        .map_err(|_| Error::OutOfHeap)?;
    for _ in 0..count {
        admission::reserve_slot(&mut session);
    }
    Ok((session, reports))
}
