//! Dynamic channel quotas and one-endpoint move transactions.
use super::*;
#[derive(Clone, Copy)]
pub(super) struct Transfer {
    pub(super) source: usize,
    pub(super) target: usize,
    pub(super) endpoint: usize,
    pub(super) right: Right,
}
impl Ipc {
    /// A single-direction channel, initially with both ends owned by the creator.
    pub fn create(&mut self, caller: usize) -> Result<[u64; 2], u64> {
        if !self.live(caller) {
            return Err(abi::BAD_HANDLE);
        }
        if self.charged_channels(caller) >= self.reservations[caller] {
            return Err(abi::NO_MEMORY);
        }
        let slots = self.grants.vacant(caller, 2)?;
        self.grants.can_mint(caller, 2)?;
        let index = (MAX_ENDPOINTS..self.endpoints.len())
            .find(|&i| self.endpoints[i].is_none())
            .expect("backed channel reservation missing storage");
        self.endpoints[index] = Some(Endpoint {
            epoch: 0,
            spec: EndpointSpec {
                sender: caller,
                receiver: caller,
                revoker: None,
            },
            message: None,
            sender_alive: true,
            receiver_alive: true,
            revoked: false,
            account: Some(caller),
        });
        Ok([
            self.grants
                .install(caller, slots[0], index, Right::Send, true),
            self.grants
                .install(caller, slots[1], index, Right::Receive, true),
        ])
    }
    pub(super) fn prepare_transfer(
        &self,
        caller: usize,
        transport: usize,
        receiver: usize,
        ticket: u64,
    ) -> Result<Transfer, u64> {
        let (source, grant) = self.movable(caller, ticket)?;
        if grant.endpoint == transport {
            return Err(abi::DENIED);
        }
        let target = self.grants.vacant(receiver, 1)?[0];
        self.grants.can_mint(receiver, 1)?;
        Ok(Transfer {
            source,
            target,
            endpoint: grant.endpoint,
            right: grant.right,
        })
    }
    pub(super) fn movable(&self, caller: usize, ticket: u64) -> Result<(usize, Grant), u64> {
        let (source, grant) = self.lookup(caller, ticket)?;
        // Boot/lifecycle grants remain pinned to their admitted principals.
        if grant.endpoint < MAX_ENDPOINTS {
            return Err(abi::DENIED);
        }
        // Keep the same receive restriction for IPC moves and initial bundles.
        // Queued receivers cannot be hidden inside escrow or a launch transaction.
        if grant.right == Right::Receive
            && self.endpoints[grant.endpoint]
                .as_ref()
                .unwrap()
                .message
                .is_some()
        {
            return Err(abi::BUSY);
        }
        Ok((source, grant))
    }
    pub(super) fn commit_transfer(
        &mut self,
        caller: usize,
        receiver: usize,
        transfer: Transfer,
    ) -> usize {
        let Transfer {
            source,
            target,
            endpoint,
            right,
        } = transfer;
        assert_eq!(self.grants.entries[source].unwrap().owner, caller);
        self.grants.entries[source] = None;
        self.grants
            .install(receiver, target, endpoint, right, false);
        let endpoint = self.endpoints[endpoint].as_mut().unwrap();
        match right {
            Right::Send => endpoint.spec.sender = receiver,
            Right::Receive => endpoint.spec.receiver = receiver,
            Right::Revoke => unreachable!(),
        }
        target
    }
}
