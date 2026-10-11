//! Lend an unused child slot, then return it only after its subtree is reclaimed.
use super::*;

impl Supervisor {
    pub fn can_lend(&self, caller: usize, ticket: u64, recipient: usize) -> Result<(), u64> {
        if self.launch(caller)? != ticket {
            return Err(abi::BAD_HANDLE);
        }
        // Separate peer consent cannot be silently transferred. The increasing
        // slot order is a bounded lab DAG, not an application-visible task ID.
        if self.peer != caller || !self.peer_alive || recipient <= caller || recipient >= self.child
        {
            return Err(abi::DENIED);
        }
        if self.state != Child::Empty {
            return Err(abi::BUSY);
        }
        Ok(())
    }

    /// Caller has validated the complete bundle and reserved both unique epochs.
    pub fn lend(&mut self, recipient: usize, epoch: u64, return_epoch: u64) {
        assert!(
            self.can_lend(self.owner, self.launch(self.owner).unwrap(), recipient)
                .is_ok()
        );
        assert!(epoch > self.latest_epoch && return_epoch > epoch && return_epoch <= MAX_EPOCH);
        self.sponsors[self.owner] = return_epoch;
        self.owner = recipient;
        self.peer = recipient;
        self.epoch = epoch;
        self.latest_epoch = return_epoch;
        self.connected = false;
    }

    /// Called after cancellation and physical reaping, before waking waiters.
    /// Skip dead intermediate lenders. Neither old grants nor outcomes revive.
    pub fn reclaim(&mut self) {
        if self.owner_alive || matches!(self.state, Child::Live(_) | Child::Starting(_)) {
            return;
        }
        let Some(owner) = (0..self.owner)
            .rev()
            .find(|&owner| self.sponsors[owner] != 0)
        else {
            return;
        };
        self.epoch = core::mem::take(&mut self.sponsors[owner]);
        self.owner = owner;
        self.peer = owner;
        self.owner_alive = true;
        self.peer_alive = true;
        self.connected = false;
        self.cancellation = false;
        self.state = Child::Empty;
    }
}
