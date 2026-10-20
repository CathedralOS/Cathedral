// Redistribute page/IPC reservations along an authorized owner-child edge.
// Capacity is conserved; live or retained backing prevents withdrawing its funding.
use crate::{ipc::Ipc, user_tasks::memory::regions::Regions};
use cathedral_kernel_contracts::memory;
use cathedral_kernel_contracts::user as abi;

/// Validate every family against the same serialized snapshot, then publish all.
/// Existing mappings/charges are untouched; growth spends only unused owner funds.
pub fn grow(
    ipc: &mut Ipc,
    regions: &mut Regions,
    funding: &mut physical::Funding,
    pool: &mut crate::user_tasks::admission::pool::Pool,
    owner: usize,
    child: usize,
    delta: [usize; 5],
) -> Result<(), u64> {
    if owner >= abi::MAX_TASKS || child >= abi::MAX_TASKS || owner == child {
        return Err(abi::INVALID_ARGUMENT);
    }
    let old = snapshot(ipc, regions, child);
    let mut limits = [0; 3];
    for i in 0..3 {
        limits[i] = old[i].checked_add(delta[i]).ok_or(abi::INVALID_ARGUMENT)?;
    }
    let frames = funding.snapshot(child)[0]
        .checked_add(delta[4])
        .ok_or(abi::INVALID_ARGUMENT)?;
    funding.check_assign(owner, child, frames)?;
    let slots = pool
        .available(child)
        .checked_add(delta[3])
        .ok_or(abi::INVALID_ARGUMENT)?;
    let mut staged_pool = pool.clone();
    staged_pool.assign(owner, child, slots).map_err(|error| {
        if error == abi::NO_MEMORY {
            abi::NO_PROCESS_CAPACITY
        } else {
            error
        }
    })?;
    assign(ipc, regions, owner, child, limits)?;
    funding
        .assign(owner, child, frames)
        .expect("validated growth");
    *pool = staged_pool;
    Ok(())
}

pub fn snapshot(ipc: &Ipc, regions: &Regions, account: usize) -> [usize; 6] {
    let [private, shared] = regions.limits(account);
    let [pages, readers] = regions.charged(account);
    [
        private,
        shared,
        ipc.channel_limit(account),
        pages,
        readers,
        ipc.charged_channels(account),
    ]
}
/// The syscall resolves the child through caller-local launch authority first.
/// Loans carry the reserved account; subdivision never mints capacity at a new depth.
pub fn assign(
    ipc: &mut Ipc,
    regions: &mut Regions,
    owner: usize,
    child: usize,
    requested: [usize; 3],
) -> Result<(), u64> {
    if owner >= abi::MAX_TASKS || child >= abi::MAX_TASKS || owner == child {
        return Err(abi::INVALID_ARGUMENT);
    }
    let parent = snapshot(ipc, regions, owner);
    let previous = snapshot(ipc, regions, child);
    let mut remaining = [0; 3];
    for i in 0..3 {
        let maximum = match i {
            0 => memory::MAX_PRIVATE_PAGES,
            1 => memory::MAX_READER_PAGES,
            _ => abi::MAX_CREATED_CHANNELS,
        };
        remaining[i] = parent[i]
            .checked_add(previous[i])
            .and_then(|total| total.checked_sub(requested[i]))
            .ok_or(abi::NO_MEMORY)?;
        if requested[i] > maximum || remaining[i] > maximum {
            return Err(abi::INVALID_ARGUMENT);
        }
        if remaining[i] < parent[i + 3] || requested[i] < previous[i + 3] {
            return Err(abi::BUSY);
        }
    }
    // Every family was checked before committing any limit. No allocation/failure.
    regions.install_limits(owner, [remaining[0], remaining[1]]);
    regions.install_limits(child, [requested[0], requested[1]]);
    ipc.install_limit(owner, remaining[2]);
    ipc.install_limit(child, requested[2]);
    Ok(())
}
