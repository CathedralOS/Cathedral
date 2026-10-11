//! Bounded IPC entrance: authorize locally, stage transfers, then commit delivery.
//! Boot composition and ordinary channel creation share transport and lifetime rules.
//! Descend into grants for local names, channels for creation/escrow, lifecycle for teardown.
mod bootstrap;
mod channels;
mod grants;
mod initial;
mod lifecycle;
pub use abi::{MAX_ENDPOINTS, MAX_TASKS};
use cathedral_contracts::user as abi;
use grants::{Grant, Grants};
pub const MAX_EPOCH: u64 = (1 << 47) - 1;

#[derive(Clone, Copy, Debug)]
pub struct EndpointSpec {
    pub sender: usize,
    pub receiver: usize,
    pub revoker: Option<usize>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Right {
    Send,
    Receive,
    Revoke,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Message {
    pub bytes: [u8; abi::MAX_MESSAGE],
    pub len: usize,
    pub attachment: Option<u64>,
}
#[derive(Clone, Copy)]
struct Queued {
    bytes: [u8; abi::MAX_MESSAGE],
    len: usize,
    // Reserved, inaccessible recipient-local grant. Published only on receive.
    attachment: Option<usize>,
}
#[derive(Clone, Copy)]
struct Endpoint {
    epoch: u64,
    spec: EndpointSpec,
    message: Option<Queued>,
    sender_alive: bool,
    receiver_alive: bool,
    revoked: bool,
    creator: Option<usize>,
}
pub struct Ipc {
    epoch: u64,
    tasks: usize,
    alive: u8,
    endpoints: alloc::vec::Vec<Option<Endpoint>>,
    grants: Grants,
    initial: [[u64; abi::MAX_INITIAL_ENDPOINTS]; MAX_TASKS],
}
impl Ipc {
    pub fn check(&self, caller: usize, ticket: u64, needed: Right) -> Result<usize, u64> {
        let (_, grant) = self.lookup(caller, ticket)?;
        if grant.right != needed {
            return Err(abi::DENIED);
        }
        if self.endpoints[grant.endpoint].as_ref().unwrap().revoked {
            return Err(abi::REVOKED);
        }
        Ok(grant.endpoint)
    }
    fn lookup(&self, caller: usize, ticket: u64) -> Result<(usize, Grant), u64> {
        if !self.live(caller) {
            return Err(abi::BAD_HANDLE);
        }
        self.grants.lookup(caller, ticket)
    }
    pub fn send(&mut self, caller: usize, ticket: u64, bytes: &[u8]) -> Result<(), u64> {
        self.send_cap(caller, ticket, bytes, None)
    }
    /// All checks and recipient reservation precede the move and queue commit.
    /// A failed send leaves both the endpoint and payload with the caller.
    pub fn send_cap(
        &mut self,
        caller: usize,
        ticket: u64,
        bytes: &[u8],
        attachment: Option<u64>,
    ) -> Result<(), u64> {
        let index = self.check(caller, ticket, Right::Send)?;
        if bytes.len() > abi::MAX_MESSAGE {
            return Err(abi::INVALID_ARGUMENT);
        }
        let endpoint = self.endpoints[index].as_ref().unwrap();
        if !endpoint.receiver_alive {
            return Err(abi::PEER_CLOSED);
        }
        if endpoint.message.is_some() {
            return Err(abi::WOULD_BLOCK);
        }
        let receiver = endpoint.spec.receiver;
        let transfer = attachment
            .map(|handle| self.prepare_transfer(caller, index, receiver, handle))
            .transpose()?;
        let reserved = transfer.map(|transfer| self.commit_transfer(caller, receiver, transfer));
        let mut message = Queued {
            bytes: [0; abi::MAX_MESSAGE],
            len: bytes.len(),
            attachment: reserved,
        };
        message.bytes[..bytes.len()].copy_from_slice(bytes);
        self.endpoints[index].as_mut().unwrap().message = Some(message);
        Ok(())
    }
    pub fn receive(
        &mut self,
        caller: usize,
        ticket: u64,
        capacity: usize,
    ) -> Result<Option<Message>, u64> {
        self.receive_cap(caller, ticket, capacity, false)
    }
    /// Too-small byte buffers or absent attachment acceptance preserve the message.
    pub fn receive_cap(
        &mut self,
        caller: usize,
        ticket: u64,
        capacity: usize,
        accept: bool,
    ) -> Result<Option<Message>, u64> {
        let index = self.check(caller, ticket, Right::Receive)?;
        let endpoint = self.endpoints[index].as_mut().unwrap();
        if let Some(message) = endpoint.message {
            if message.len > capacity || (message.attachment.is_some() && !accept) {
                return Err(abi::TOO_SMALL);
            }
            endpoint.message = None;
            let attachment = message
                .attachment
                .map(|slot| self.grants.publish(caller, slot));
            return Ok(Some(Message {
                bytes: message.bytes,
                len: message.len,
                attachment,
            }));
        }
        if !endpoint.sender_alive {
            return Err(abi::PEER_CLOSED);
        }
        Ok(None)
    }
    pub fn revoke(&mut self, caller: usize, ticket: u64) -> Result<(), u64> {
        let index = self.check(caller, ticket, Right::Revoke)?;
        self.endpoints[index].as_mut().unwrap().revoked = true;
        self.discard(index);
        Ok(())
    }
    pub fn ready(&self, caller: usize, ticket: u64) -> Result<bool, u64> {
        let index = self.check(caller, ticket, Right::Receive)?;
        let endpoint = self.endpoints[index].as_ref().unwrap();
        Ok(endpoint.message.is_some() || !endpoint.sender_alive)
    }
    /// Validate the entire receive set before reporting its first ready entry.
    pub fn ready_any(&self, caller: usize, handles: &[u64]) -> Result<Option<u64>, u64> {
        if handles.is_empty() || handles.len() > abi::MAX_WAIT_HANDLES {
            return Err(abi::INVALID_ARGUMENT);
        }
        for &handle in handles {
            self.check(caller, handle, Right::Receive)?;
        }
        for (index, &handle) in handles.iter().enumerate() {
            if self.ready(caller, handle)? {
                return Ok(Some(index as u64));
            }
        }
        Ok(None)
    }
    fn live(&self, task: usize) -> bool {
        task < self.tasks && self.alive & (1 << task) != 0
    }
}
#[cfg(test)]
mod tests;
