// Backed channel reservations and caller-local accounting. Accounts outlive tasks.
// Endpoint moves change custody, never sponsorship. Final endpoint closure frees
// the charge; restarting the creator cannot mint another storage allowance.
use super::*;
pub(super) fn reservations(tasks: usize, limits: &[usize]) -> Result<[usize; MAX_TASKS], u64> {
    if tasks > MAX_TASKS || (!limits.is_empty() && limits.len() != tasks) {
        return Err(abi::INVALID_ARGUMENT);
    }
    let mut result = [0; MAX_TASKS];
    for (task, limit) in result.iter_mut().enumerate().take(tasks) {
        *limit = if limits.is_empty() {
            abi::DEFAULT_CHANNELS
        } else {
            limits[task]
        };
        if *limit > abi::MAX_CREATED_CHANNELS {
            return Err(abi::INVALID_ARGUMENT);
        }
    }
    if result.iter().sum::<usize>() > abi::MAX_DYNAMIC_CHANNELS {
        return Err(abi::INVALID_ARGUMENT);
    }
    Ok(result)
}
impl Ipc {
    pub(crate) fn charged_channels(&self, task: usize) -> usize {
        self.notifications.charged(task)
            + self.endpoints[MAX_ENDPOINTS..]
                .iter()
                .flatten()
                .filter(|e| e.account == Some(task))
                .count()
    }
    pub(crate) fn channel_limit(&self, task: usize) -> usize {
        self.reservations[task]
    }
    pub(crate) fn install_limit(&mut self, task: usize, limit: usize) {
        self.reservations[task] = limit;
    }
    /// No task selector or global free count. Hidden incoming grants still consume
    /// the caller's handle budget, so the snapshot includes them.
    pub fn budget(&self, caller: usize) -> Result<[u64; 4], u64> {
        if !self.live(caller) {
            return Err(abi::BAD_HANDLE);
        }
        let handles = self.grants.entries[Grants::dynamic_slots(caller)]
            .iter()
            .flatten()
            .filter(|g| g.owner == caller && g.endpoint >= MAX_ENDPOINTS)
            .count();
        Ok([
            self.reservations[caller] as u64,
            self.charged_channels(caller) as u64,
            abi::MAX_DYNAMIC_HANDLES as u64,
            handles as u64,
        ])
    }
}
