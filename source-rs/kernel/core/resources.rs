//! Redistribute page/IPC reservations along an authorized owner-child edge.
//! Capacity is conserved; live or retained backing prevents withdrawing its funding.
pub mod physical;
use crate::{ipc::Ipc, regions::Regions};
use cathedral_contracts::{memory, user as abi};

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
        let maximum = if i == 2 {
            abi::MAX_CREATED_CHANNELS
        } else {
            memory::MAX_PAGES
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

#[cfg(test)]
mod tests;
