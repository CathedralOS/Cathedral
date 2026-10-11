//! Close grants, discard unaccepted moves, and release empty dynamic channels.
use super::*;
impl Ipc {
    pub fn close(&mut self, caller: usize, ticket: u64) -> Result<(), u64> {
        let (slot, _) = self.lookup(caller, ticket)?;
        self.close_grant(slot);
        Ok(())
    }
    fn close_grant(&mut self, slot: usize) {
        let Some(grant) = self.grants.entries[slot].take() else {
            return;
        };
        let endpoint = self.endpoints[grant.endpoint].as_mut().unwrap();
        match grant.right {
            Right::Send => endpoint.sender_alive = false,
            Right::Receive => {
                endpoint.receiver_alive = false;
                self.discard(grant.endpoint);
            }
            Right::Revoke => (),
        }
        self.reclaim(grant.endpoint);
    }
    pub(super) fn discard(&mut self, index: usize) {
        let queued = self.endpoints[index].as_mut().unwrap().message.take();
        if let Some(slot) = queued.and_then(|q| q.attachment) {
            // Queued receives cannot themselves be moved. However the endpoint
            // can receive a message after its receive right entered escrow; thus
            // teardown can cascade. Walk the one-attachment chain without recursion.
            self.discard_grant_tree(slot);
        }
    }
    fn discard_grant_tree(&mut self, first: usize) {
        let mut pending = Some(first);
        while let Some(slot) = pending.take() {
            let Some(grant) = self.grants.entries[slot].take() else {
                continue;
            };
            let endpoint = self.endpoints[grant.endpoint].as_mut().unwrap();
            match grant.right {
                Right::Send => endpoint.sender_alive = false,
                Right::Receive => {
                    endpoint.receiver_alive = false;
                    if let Some(next) = endpoint.message.take().and_then(|q| q.attachment) {
                        pending = Some(next);
                    }
                }
                Right::Revoke => unreachable!(),
            }
            self.reclaim(grant.endpoint);
        }
    }
    fn reclaim(&mut self, index: usize) {
        if index >= MAX_ENDPOINTS
            && self.endpoints[index].is_some_and(|e| !e.sender_alive && !e.receiver_alive)
        {
            self.endpoints[index] = None;
        }
    }
    pub fn close_task(&mut self, task: usize) {
        assert!(task < self.tasks);
        self.alive &= !(1 << task);
        for slot in 0..self.grants.entries.len() {
            if self.grants.entries[slot].is_some_and(|g| g.owner == task) {
                self.close_grant(slot);
            }
        }
        // Transferred endpoints survive creator death. Their current holders'
        // handle budgets and the global pool still bound their storage.
        for endpoint in self.endpoints[MAX_ENDPOINTS..].iter_mut().flatten() {
            if endpoint.creator == Some(task) {
                endpoint.creator = None;
            }
        }
    }
}
