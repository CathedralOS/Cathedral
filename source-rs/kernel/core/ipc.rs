//! Bounded copied-message lab transport. Allocates its arena once at admission.
//! No allocation in message operations, names or delegation.
//! Tickets are bound to a caller and a non-reused session epoch, not bearer tokens.
use cathedral_contracts::user as abi;

pub use abi::{MAX_ENDPOINTS, MAX_TASKS};
pub const MAX_EPOCH: u64 = (1 << 47) - 1;

#[derive(Clone, Copy, Debug)]
pub struct EndpointSpec {
    pub sender: usize,
    pub receiver: usize,
    pub revoker: Option<usize>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Right {
    Send,
    Receive,
    Revoke,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Message {
    pub bytes: [u8; abi::MAX_MESSAGE],
    pub len: usize,
}
#[derive(Clone, Copy)]
struct Endpoint {
    epoch: u64,
    hidden: u8,
    spec: EndpointSpec,
    message: Option<Message>,
    sender_alive: bool,
    receiver_alive: bool,
    revoked: bool,
}
pub struct Ipc {
    epoch: u64,
    tasks: usize,
    alive: u8,
    endpoints: alloc::vec::Vec<Option<Endpoint>>,
}
impl Ipc {
    pub fn new(epoch: u64, tasks: usize, specs: &[EndpointSpec]) -> Result<Self, u64> {
        if !(1..=MAX_EPOCH).contains(&epoch) || tasks > MAX_TASKS || specs.len() > MAX_ENDPOINTS {
            return Err(abi::INVALID_ARGUMENT);
        }
        let mut endpoints = alloc::vec::Vec::new();
        endpoints
            .try_reserve_exact(MAX_ENDPOINTS)
            .map_err(|_| abi::NO_MEMORY)?;
        endpoints.resize(MAX_ENDPOINTS, None);
        let mut ipc = Self {
            epoch,
            tasks,
            alive: ((1u16 << tasks) - 1) as u8,
            endpoints,
        };
        for (index, &spec) in specs.iter().enumerate() {
            if spec.sender >= tasks
                || spec.receiver >= tasks
                || spec.sender == spec.receiver
                || spec.revoker.is_some_and(|slot| slot >= tasks)
            {
                return Err(abi::INVALID_ARGUMENT);
            }
            ipc.endpoints[index] = Some(Endpoint {
                epoch,
                hidden: 0,
                spec,
                message: None,
                sender_alive: true,
                receiver_alive: true,
                revoked: false,
            });
        }
        Ok(ipc)
    }

    fn owner(endpoint: &Endpoint, right: Right) -> Option<usize> {
        match right {
            Right::Send => Some(endpoint.spec.sender),
            Right::Receive => Some(endpoint.spec.receiver),
            Right::Revoke => endpoint.spec.revoker,
        }
    }
    fn ticket(epoch: u64, owner: usize, slot: usize) -> u64 {
        (epoch << 16) | ((owner as u64) << 8) | (slot as u64 + 1)
    }
    /// Enumerate only the calling task's preinstalled grants, in endpoint/right order.
    pub fn handle(&self, caller: usize, index: u64) -> Result<u64, u64> {
        if !self.live(caller) {
            return Err(abi::BAD_HANDLE);
        }
        let mut found = 0;
        for (endpoint_index, endpoint) in self.endpoints.iter().enumerate() {
            let Some(endpoint) = endpoint else { continue };
            if endpoint.hidden & (1 << caller) != 0 {
                continue;
            }
            for (right_index, right) in [Right::Send, Right::Receive, Right::Revoke]
                .into_iter()
                .enumerate()
            {
                if Self::owner(endpoint, right) == Some(caller) {
                    if found == index {
                        return Ok(Self::ticket(
                            endpoint.epoch,
                            caller,
                            endpoint_index * 3 + right_index,
                        ));
                    }
                    found += 1;
                }
            }
        }
        Err(abi::BAD_HANDLE)
    }
    pub fn check(&self, caller: usize, ticket: u64, needed: Right) -> Result<usize, u64> {
        let slot = (ticket & 255).checked_sub(1).ok_or(abi::BAD_HANDLE)? as usize;
        if !self.live(caller) || slot >= MAX_ENDPOINTS * 3 {
            return Err(abi::BAD_HANDLE);
        }
        let endpoint = self.endpoints[slot / 3].as_ref().ok_or(abi::BAD_HANDLE)?;
        if endpoint.hidden & (1 << caller) != 0
            || ticket != Self::ticket(endpoint.epoch, caller, slot)
        {
            return Err(abi::BAD_HANDLE);
        }
        let right = [Right::Send, Right::Receive, Right::Revoke][slot % 3];
        if Self::owner(endpoint, right) != Some(caller) {
            return Err(abi::BAD_HANDLE);
        }
        if right != needed {
            return Err(abi::DENIED);
        }
        if endpoint.revoked {
            return Err(abi::REVOKED);
        }
        Ok(slot / 3)
    }
    pub fn send(&mut self, caller: usize, ticket: u64, bytes: &[u8]) -> Result<(), u64> {
        let index = self.check(caller, ticket, Right::Send)?;
        if bytes.len() > abi::MAX_MESSAGE {
            return Err(abi::INVALID_ARGUMENT);
        }
        let endpoint = self.endpoints[index].as_mut().unwrap();
        if !endpoint.receiver_alive {
            return Err(abi::PEER_CLOSED);
        }
        if endpoint.message.is_some() {
            return Err(abi::WOULD_BLOCK);
        }
        let mut message = Message {
            bytes: [0; abi::MAX_MESSAGE],
            len: bytes.len(),
        };
        message.bytes[..bytes.len()].copy_from_slice(bytes);
        endpoint.message = Some(message);
        Ok(())
    }
    /// None means park. Too-small buffers leave the queued message intact.
    pub fn receive(
        &mut self,
        caller: usize,
        ticket: u64,
        capacity: usize,
    ) -> Result<Option<Message>, u64> {
        let index = self.check(caller, ticket, Right::Receive)?;
        let endpoint = self.endpoints[index].as_mut().unwrap();
        if let Some(message) = endpoint.message {
            if message.len > capacity {
                return Err(abi::TOO_SMALL);
            }
            return Ok(endpoint.message.take());
        }
        if !endpoint.sender_alive {
            return Err(abi::PEER_CLOSED);
        }
        Ok(None)
    }
    /// Experimental cancellation policy: queued messages and pending receives are revoked.
    pub fn revoke(&mut self, caller: usize, ticket: u64) -> Result<(), u64> {
        let index = self.check(caller, ticket, Right::Revoke)?;
        let endpoint = self.endpoints[index].as_mut().unwrap();
        endpoint.revoked = true;
        endpoint.message = None;
        Ok(())
    }
    pub fn ready(&self, caller: usize, ticket: u64) -> Result<bool, u64> {
        let index = self.check(caller, ticket, Right::Receive)?;
        let endpoint = self.endpoints[index].as_ref().unwrap();
        Ok(endpoint.message.is_some() || !endpoint.sender_alive)
    }
    pub fn close_task(&mut self, task: usize) {
        assert!(task < self.tasks);
        self.alive &= !(1 << task);
        for endpoint in self.endpoints.iter_mut().flatten() {
            if endpoint.spec.sender == task {
                endpoint.sender_alive = false;
            }
            if endpoint.spec.receiver == task {
                endpoint.receiver_alive = false;
                endpoint.message = None;
            }
        }
    }
    fn live(&self, task: usize) -> bool {
        task < self.tasks && self.alive & (1 << task) != 0
    }
}

// Dynamic launch replaces only two explicitly reserved endpoint slots. Ordinary
// boot endpoints keep their generations and continue serving the live peers.
impl Ipc {
    pub fn prepare_child(&mut self, first: usize, peer: usize, child: usize, epoch: u64) {
        assert!(first + 1 < MAX_ENDPOINTS && self.live(peer) && !self.live(child));
        assert!(child < self.tasks && child != peer && epoch > self.epoch && epoch <= MAX_EPOCH);
        self.alive |= 1 << child;
        self.prepare_pair(first, peer, child, epoch);
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
        for (index, sender, receiver) in [(first, peer, child), (first + 1, child, peer)] {
            self.endpoints[index] = Some(Endpoint {
                epoch,
                hidden: (1 << peer) | (1 << child),
                spec: EndpointSpec {
                    sender,
                    receiver,
                    revoker: None,
                },
                message: None,
                sender_alive: true,
                receiver_alive: true,
                revoked: false,
            });
        }
    }
    pub fn accept_child(&mut self, first: usize, peer: usize) -> u64 {
        assert!(self.live(peer));
        for index in first..first + 2 {
            self.endpoints[index].as_mut().unwrap().hidden &= !(1 << peer);
        }
        let ticket = Self::ticket(
            self.endpoints[first].as_ref().unwrap().epoch,
            peer,
            first * 3 + usize::from(self.endpoints[first].as_ref().unwrap().spec.receiver == peer),
        );
        (0..(MAX_ENDPOINTS * 3) as u64)
            .find(|index| self.handle(peer, *index) == Ok(ticket))
            .unwrap()
    }
}

#[cfg(test)]
mod tests;
