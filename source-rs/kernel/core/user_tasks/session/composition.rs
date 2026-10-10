//! Validate resource bounds, then construct caller-bound launch and peer grants.
use super::{Config, Error, LaunchState, LinkState, next_epoch};
use crate::{ipc::Ipc, supervision::Supervisor};
use alloc::vec::Vec;
pub(super) struct Grants {
    pub(super) clock: crate::deadline::Clock,
    pub(super) launches: Vec<LaunchState>,
    pub(super) links: Vec<LinkState>,
    pub(super) ipc: Ipc,
}
pub(super) fn validate(initial_count: usize, config: &Config<'_>) -> Result<usize, Error> {
    if initial_count == 0 || initial_count > 8 {
        return Err(Error::InvalidCount);
    }
    let count = initial_count + config.supervision.len();
    if count > crate::ipc::MAX_TASKS
        || config.supervision.len() > cathedral_contracts::user::MAX_LAUNCHES
    {
        return Err(Error::InvalidCount);
    }
    if config.endpoints.len() + 2 * (config.supervision.len() + config.links.len())
        > crate::ipc::MAX_ENDPOINTS
        || config
            .supervision
            .iter()
            .filter(|grant| grant.framebuffer.is_some())
            .count()
            > 1
        || config
            .supervision
            .iter()
            .filter(|grant| grant.keyboard)
            .count()
            > 1
    {
        return Err(Error::InvalidEndpoints);
    }
    if config.supervision.iter().filter(|grant| grant.disk).count() > 1 {
        return Err(Error::InvalidEndpoints);
    }
    Ok(count)
}
pub(super) fn grants(
    initial_count: usize,
    count: usize,
    config: &Config<'_>,
) -> Result<Grants, Error> {
    let epoch = next_epoch().ok_or(Error::InvalidEndpoints)?;
    let clock = crate::deadline::Clock::new(epoch, count, config.clock_readers)
        .map_err(|_| Error::InvalidEndpoints)?;
    let mut launches = Vec::new();
    launches
        .try_reserve_exact(config.supervision.len())
        .map_err(|_| Error::OutOfHeap)?;
    for (index, launch) in config.supervision.iter().enumerate() {
        if launch.framebuffer.is_some_and(|fb| !fb.valid())
            || launch.owner >= initial_count
            || launch.peer >= initial_count
        {
            return Err(Error::InvalidEndpoints);
        }
        let grant_epoch = next_epoch().ok_or(Error::InvalidEndpoints)?;
        launches.push(LaunchState {
            model: Supervisor::new(
                grant_epoch,
                launch.owner,
                launch.peer,
                initial_count + index,
            )
            .map_err(|_| Error::InvalidEndpoints)?,
            endpoint_base: config.endpoints.len() + 2 * index,
            framebuffer: launch.framebuffer,
            keyboard: launch.keyboard,
            disk: launch.disk,
        });
    }
    // Static grants may refer only to initial principals, never the reusable child slot.
    if config.endpoints.iter().any(|spec| {
        spec.sender >= initial_count
            || spec.receiver >= initial_count
            || spec.revoker.is_some_and(|slot| slot >= initial_count)
    }) {
        return Err(Error::InvalidEndpoints);
    }
    let mut links = Vec::new();
    links
        .try_reserve_exact(config.links.len())
        .map_err(|_| Error::OutOfHeap)?;
    for (index, &spec) in config.links.iter().enumerate() {
        // Links connect reserved children only in this bounded launch graph.
        if spec.client < initial_count || spec.service < initial_count {
            return Err(Error::InvalidEndpoints);
        }
        links.push(LinkState {
            model: crate::link::Link::new(
                next_epoch().ok_or(Error::InvalidEndpoints)?,
                spec,
                count,
                initial_count,
            )
            .map_err(|_| Error::InvalidEndpoints)?,
            endpoint_base: config.endpoints.len() + 2 * (config.supervision.len() + index),
        });
    }
    let mut ipc = Ipc::new(epoch, count, config.endpoints).map_err(|_| Error::InvalidEndpoints)?;
    for launch in &launches {
        ipc.close_task(launch.model.child);
    }
    Ok(Grants {
        clock,
        launches,
        links,
        ipc,
    })
}
