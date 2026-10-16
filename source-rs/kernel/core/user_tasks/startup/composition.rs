//! Validate resource bounds, then construct caller-bound launch and peer grants.
use super::{Config, Error, LaunchState, LinkState, next_epoch};
use crate::{ipc::Ipc, user_tasks::lifecycle::supervision::Supervisor};
use alloc::vec::Vec;
pub(super) struct Grants {
    pub(super) names: crate::names::Names,
    pub(super) clock: crate::user_tasks::time::Clock,
    pub(super) launches: Vec<LaunchState>,
    pub(super) links: Vec<LinkState>,
    pub(super) ipc: Ipc,
}
pub(super) fn validate(initial_count: usize, config: &Config<'_>) -> Result<usize, Error> {
    if initial_count == 0 || initial_count > crate::ipc::MAX_TASKS {
        return Err(Error::InvalidCount);
    }
    let count = initial_count
        .checked_add(config.supervision.len())
        .and_then(|n| n.checked_add(config.dynamic_tasks))
        .ok_or(Error::InvalidCount)?;
    if (config.dynamic_tasks != 0 || !config.executables.is_empty()) && initial_count != 1 {
        return Err(Error::InvalidCount);
    }
    if count > crate::ipc::MAX_TASKS
        || config.supervision.len() + config.executables.len()
            > cathedral_kernel_contracts::user::MAX_LAUNCHES
    {
        return Err(Error::InvalidCount);
    }
    if config.endpoints.len() + 2 * (count - initial_count + config.links.len())
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
    for (index, grant) in config.supervision.iter().enumerate() {
        if let Some(device) = grant.device
            && (!device.available()
                || config.supervision[..index]
                    .iter()
                    .any(|g| g.device == Some(device)))
        {
            return Err(Error::InvalidEndpoints);
        }
    }
    Ok(count)
}
pub(super) fn grants(
    initial_count: usize,
    count: usize,
    config: &Config<'_>,
) -> Result<Grants, Error> {
    let epoch = next_epoch().ok_or(Error::InvalidEndpoints)?;
    let mut names = crate::names::Names::default();
    let mut clock = crate::user_tasks::time::Clock::new(count, config.clock_readers)
        .map_err(|_| Error::InvalidEndpoints)?;
    for slot in 0..initial_count {
        if clock.allowed(slot) {
            clock.install(slot, names.mint(slot).map_err(|_| Error::InvalidEndpoints)?);
        }
    }
    let mut launches = Vec::new();
    launches
        .try_reserve_exact(count - initial_count)
        .map_err(|_| Error::OutOfHeap)?;
    for (index, launch) in config.supervision.iter().enumerate() {
        if (launch.keyboard && cathedral_arch::board::devices().keyboard.is_none())
            || (launch.disk && cathedral_arch::board::devices().disk.is_none())
            || launch.framebuffer.is_some_and(|fb| !fb.valid())
            || launch.owner >= initial_count
            || launch.peer >= initial_count
        {
            return Err(Error::InvalidEndpoints);
        }

        launches.push(LaunchState {
            images: [None; cathedral_kernel_contracts::user::MAX_LAUNCHES],
            template: Some(index),
            dynamic: false,
            frame_limit: launch.frame_limit,
            cpu_origin: 0,
            delegated: [0; cathedral_kernel_contracts::user::MAX_LAUNCHES],
            delegated_count: 0,
            initial: [0; cathedral_kernel_contracts::user::MAX_INITIAL_ENDPOINTS],
            initial_count: 0,
            model: Supervisor::new(
                names
                    .pair(launch.owner, launch.peer)
                    .map_err(|_| Error::InvalidEndpoints)?,
                launch.owner,
                launch.peer,
                initial_count + index,
            )
            .map_err(|_| Error::InvalidEndpoints)?,
            endpoint_base: config.endpoints.len() + 2 * index,
            framebuffer: launch.framebuffer,
            keyboard: launch.keyboard,
            disk: launch.disk,
            device: launch.device,
        });
    }
    for index in config.supervision.len()..count - initial_count {
        launches.push(LaunchState {
            images: [None; cathedral_kernel_contracts::user::MAX_LAUNCHES],
            template: None,
            dynamic: true,
            frame_limit: 0,
            cpu_origin: 0,
            delegated: [0; cathedral_kernel_contracts::user::MAX_LAUNCHES],
            delegated_count: 0,
            initial: [0; cathedral_kernel_contracts::user::MAX_INITIAL_ENDPOINTS],
            initial_count: 0,
            model: Supervisor::vacant(initial_count + index),
            endpoint_base: config.endpoints.len() + 2 * index,
            framebuffer: None,
            keyboard: false,
            disk: false,
            device: None,
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
            model: crate::ipc::link::Link::new(spec, count, initial_count)
                .map_err(|_| Error::InvalidEndpoints)?,
            endpoint_base: config.endpoints.len() + 2 * (count - initial_count + index),
        });
    }
    let mut defaults = [0; cathedral_kernel_contracts::user::MAX_TASKS];
    defaults[..initial_count + config.supervision.len()]
        .fill(cathedral_kernel_contracts::user::DEFAULT_CHANNELS);
    let limits = if config.channel_limits.is_empty() {
        &defaults[..count]
    } else {
        config.channel_limits
    };
    let mut ipc = Ipc::with_reservations(epoch, count, config.endpoints, limits)
        .map_err(|_| Error::InvalidEndpoints)?;
    for launch in &launches {
        ipc.close_task(launch.model.child);
    }
    Ok(Grants {
        names,
        clock,
        launches,
        links,
        ipc,
    })
}
