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
    let mut session = Session {
        tasks: Vec::new(),
        scheduler: Scheduler::new(count),
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
