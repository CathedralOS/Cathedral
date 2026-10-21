// Allocate every arena before publishing a task or entering a trap callback.
use super::{Config, Error, LaunchState, LinkState, Report, Session, admission, next_epoch};
use alloc::{boxed::Box, vec::Vec};
pub(super) fn prepare(
    initial_count: usize,
    config: &Config<'_>,
    baseline: usize,
    output: fn(&[u8]) -> bool,
) -> Result<(Box<Session>, Vec<Report>), Error> {
    let count = composition::validate(initial_count, config)?;
    let grants = composition::grants(initial_count, count, config)?;
    let mut session = arena::allocate(count, config, grants, baseline, output)?;
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
