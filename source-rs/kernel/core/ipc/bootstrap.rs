//! Boot-only graph admission; reserved endpoint slots cannot be consumed by apps.
use super::*;
impl Ipc {
    pub fn new(epoch: u64, tasks: usize, specs: &[EndpointSpec]) -> Result<Self, u64> {
        Self::with_reservations(epoch, tasks, specs, &[])
    }
    pub fn with_reservations(
        epoch: u64,
        tasks: usize,
        specs: &[EndpointSpec],
        limits: &[usize],
    ) -> Result<Self, u64> {
        if !(1..=MAX_EPOCH).contains(&epoch) || tasks > MAX_TASKS || specs.len() > MAX_ENDPOINTS {
            return Err(abi::INVALID_ARGUMENT);
        }
        let reservations = budget::reservations(tasks, limits)?;
        let mut endpoints = alloc::vec::Vec::new();
        endpoints
            .try_reserve_exact(MAX_ENDPOINTS + abi::MAX_DYNAMIC_CHANNELS)
            .map_err(|_| abi::NO_MEMORY)?;
        endpoints.resize(MAX_ENDPOINTS + abi::MAX_DYNAMIC_CHANNELS, None);
        let mut initial = alloc::vec::Vec::new();
        initial
            .try_reserve_exact(tasks)
            .map_err(|_| abi::NO_MEMORY)?;
        initial.resize(tasks, [0; abi::MAX_INITIAL_ENDPOINTS]);
        let mut ipc = Self {
            epoch,
            retired_memory: [0; cathedral_kernel_contracts::memory::MAX_READ_HANDLES.div_ceil(64)],
            tasks,
            reservations,
            alive: (1u32 << tasks) - 1,
            changed_receivers: 0,
            retired_memory_pending: false,
            endpoints,
            grants: Grants::new(tasks)?,
            initial,
        };
        for (index, &spec) in specs.iter().enumerate() {
            if spec.sender >= tasks
                || spec.receiver >= tasks
                || spec.sender == spec.receiver
                || spec.revoker.is_some_and(|slot| slot >= tasks)
            {
                return Err(abi::INVALID_ARGUMENT);
            }
            ipc.install_boot(index, spec, epoch, true);
        }
        Ok(ipc)
    }
    fn install_boot(&mut self, index: usize, spec: EndpointSpec, epoch: u64, visible: bool) {
        if let Some(old) = self.endpoints[index] {
            self.changed_receivers |= 1 << old.spec.receiver;
            self.discard(index);
        }
        for grant in &mut self.grants.entries {
            if grant.is_some_and(|g| g.endpoint == index) {
                *grant = None;
            }
        }
        self.endpoints[index] = Some(Endpoint {
            epoch,
            spec,
            message: None,
            sender_alive: true,
            receiver_alive: true,
            revoked: false,
            account: None,
        });
        for (owner, right) in [
            (Some(spec.sender), Right::Send),
            (Some(spec.receiver), Right::Receive),
            (spec.revoker, Right::Revoke),
        ] {
            if let Some(owner) = owner {
                self.grants.install_boot(owner, index, right, visible);
            }
        }
    }
    /// Enumerates installed boot rights only. Created/transferred rights are
    /// returned directly by create/receive, never ambient discovery.
    pub fn handle(&self, caller: usize, index: u64) -> Result<u64, u64> {
        if !self.live(caller) {
            return Err(abi::BAD_HANDLE);
        }
        self.grants.handle(caller, index)
    }
    pub fn prepare_child(&mut self, first: usize, peer: usize, child: usize, epoch: u64) {
        assert!(first + 1 < MAX_ENDPOINTS && self.live(peer) && !self.live(child));
        assert!(child < self.tasks && child != peer && epoch > self.epoch && epoch <= MAX_EPOCH);
        // A collected dynamic slot may return under a different owner. Its
        // reserved control pair belongs to the child slot, not the old parent.
        // Require the old child's side to be closed, then invalidate all old
        // grants and escrow while installing the newly authorized owner pair.
        for index in first..first + 2 {
            if let Some(old) = self.endpoints[index] {
                assert!(old.account.is_none() && old.epoch < epoch);
                assert!(
                    (old.spec.sender == child && !old.sender_alive)
                        || (old.spec.receiver == child && !old.receiver_alive)
                );
            }
        }
        self.alive |= 1 << child;
        self.changed_receivers &= !(1 << child);
        self.install_pair(first, peer, child, epoch);
        self.accept_child(first, child);
    }
    pub fn prepare_pair(&mut self, first: usize, peer: usize, child: usize, epoch: u64) {
        assert!(first + 1 < MAX_ENDPOINTS && self.live(peer) && self.live(child));
        assert!(child != peer && epoch > self.epoch && epoch <= MAX_EPOCH);
        for index in first..first + 2 {
            if let Some(old) = self.endpoints[index] {
                assert!(old.epoch < epoch && (!old.sender_alive || !old.receiver_alive));
                assert!(
                    (old.spec.sender == child && old.spec.receiver == peer)
                        || (old.spec.receiver == child && old.spec.sender == peer)
                );
            }
        }
        self.install_pair(first, peer, child, epoch);
    }
    fn install_pair(&mut self, first: usize, peer: usize, child: usize, epoch: u64) {
        for (index, sender, receiver) in [(first, peer, child), (first + 1, child, peer)] {
            self.install_boot(
                index,
                EndpointSpec {
                    sender,
                    receiver,
                    revoker: None,
                },
                epoch,
                false,
            );
        }
    }
    pub fn accept_child(&mut self, first: usize, peer: usize) -> u64 {
        assert!(self.live(peer));
        let entries = &mut self.grants.entries;
        for grant in entries.iter_mut().flatten() {
            if grant.owner == peer && (first..first + 2).contains(&grant.endpoint) {
                grant.visible = true;
            }
        }
        entries
            .iter()
            .flatten()
            .filter(|g| g.owner == peer && g.visible)
            .position(|g| g.endpoint == first)
            .unwrap() as u64
    }
}
