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

    /// Caller has validated the complete bundle and reserved fresh recipient and return names.
    pub fn lend(&mut self, recipient: usize, names: [u64; 2], returned: [u64; 2]) {
        assert!(
            self.can_lend(self.owner, self.launch(self.owner).unwrap(), recipient)
                .is_ok()
        );
        self.sponsors[self.owner] = returned;
        self.owner = recipient;
        self.peer = recipient;
        self.names = names;
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
            .find(|&owner| self.sponsors[owner] != [0; 2])
        else {
            return;
        };
        self.names = core::mem::take(&mut self.sponsors[owner]);
        self.owner = owner;
        self.peer = owner;
        self.owner_alive = true;
        self.peer_alive = true;
        self.connected = false;
        self.cancellation = false;
        self.state = Child::Empty;
    }
}
