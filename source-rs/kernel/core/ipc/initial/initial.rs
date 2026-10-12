// Startup bundles: validate the whole set, then publish all rights before execution.
use super::channels::Transfer;
use super::*;
impl Ipc {
    /// A caller-local startup snapshot. Closing/moving a returned name cannot
    /// resurrect it by reading this snapshot again. No live-grant enumeration.
    pub fn initial(&self, caller: usize) -> Result<&[u64], u64> {
        if !self.live(caller) {
            return Err(abi::BAD_HANDLE);
        }
        let names = &self.initial[caller];
        let count = names
            .iter()
            .position(|&name| name == 0)
            .unwrap_or(names.len());
        Ok(&names[..count])
    }
    pub fn validate_initial(&self, owner: usize, child: usize, handles: &[u64]) -> Result<(), u64> {
        self.plan_initial(owner, child, handles).map(|_| ())
    }
    fn plan_initial(
        &self,
        owner: usize,
        child: usize,
        handles: &[u64],
    ) -> Result<[Option<Transfer>; abi::MAX_INITIAL_ENDPOINTS], u64> {
        if !self.live(owner) || child >= self.tasks || self.live(child) {
            return Err(abi::BAD_HANDLE);
        }
        if handles.len() > abi::MAX_INITIAL_ENDPOINTS {
            return Err(abi::INVALID_ARGUMENT);
        }
        let targets = self.grants.vacant(child, handles.len())?;
        self.grants.can_mint(child, handles.len() as u64)?;
        let mut moves: [Option<Transfer>; abi::MAX_INITIAL_ENDPOINTS] =
            [None; abi::MAX_INITIAL_ENDPOINTS];
        for (i, &ticket) in handles.iter().enumerate() {
            let (source, grant) = self.movable(owner, ticket)?;
            if moves[..i].iter().flatten().any(|m| m.source == source) {
                return Err(abi::INVALID_ARGUMENT);
            }
            moves[i] = Some(Transfer {
                source,
                target: targets[i],
                endpoint: grant.endpoint,
                right: grant.right,
            });
        }
        Ok(moves)
    }
    /// The target is a dormant reserved child. Failure has no side effects;
    /// success refreshes its boot connection and installs the complete bundle.
    pub fn prepare_child_with(
        &mut self,
        first: usize,
        peer: usize,
        child: usize,
        epoch: u64,
        owner: usize,
        handles: &[u64],
    ) -> Result<(), u64> {
        let moves = self.plan_initial(owner, child, handles)?;
        self.prepare_child(first, peer, child, epoch);
        for (i, transfer) in moves.into_iter().flatten().enumerate() {
            let slot = self.commit_transfer(owner, child, transfer);
            self.initial[child][i] = self.grants.publish(child, slot);
        }
        Ok(())
    }
}
