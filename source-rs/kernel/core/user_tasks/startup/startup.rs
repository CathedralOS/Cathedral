// Allocate every arena before publishing a task or entering a trap callback.
use super::{Config, Error, LaunchState, LinkState, Report, Session, admission, next_epoch};
use crate::scheduler::Scheduler;
use alloc::vec::Vec;
use cathedral_arch::Context;
pub(super) fn prepare(
    initial_count: usize,
    config: &Config<'_>,
    baseline: usize,
    output: fn(&[u8]) -> bool,
) -> Result<(Session, Vec<Report>), Error> {
    let count = composition::validate(initial_count, config)?;
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
            if code == cathedral_kernel_contracts::user::NO_MEMORY {
                Error::OutOfHeap
            } else {
                Error::InvalidEndpoints
            }
        })?;
    let mut session = Session {
        deferred: super::execution::deferred::Pending::default(),
        child_launch: core::array::from_fn(|slot| {
            grants
                .launches
                .iter()
                .position(|launch| launch.model.child == slot)
        }),
        waits: super::execution::waits::Registry::new(count),
        device_routes: core::array::from_fn(|slot| {
            grants
                .launches
                .iter()
                .find(|launch| launch.model.child == slot)
                .and_then(|launch| launch.device)
        }),
        keyboard_owner: grants
            .launches
            .iter()
            .find(|launch| launch.keyboard)
            .map(|launch| launch.model.child),
        devices: super::devices::Devices::default(),
        pool: crate::user_tasks::admission::pool::Pool::new(count - config.dynamic_tasks, count),
        executables: crate::user_tasks::admission::pool::Executables::new(count)
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
        keyboard: crate::user_tasks::syscalls::devices::keyboard::queue::ByteQueue::new(),
        frame_baseline: baseline,
        clock: grants.clock,
    };
    for index in 0..config.executables.len() {
        let name = session.names.mint(0).map_err(|_| Error::OutOfHeap)?;
        session
            .executables
            .install(
                0,
                crate::user_tasks::admission::pool::Code::Template(
                    config.supervision.len() + index,
                ),
                name,
            )
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
