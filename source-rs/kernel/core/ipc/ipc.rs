// Bounded IPC entrance: authorize locally, stage transfers, then commit delivery.
// Boot composition and ordinary channel creation share transport and lifetime rules.
// Descend into grants for local names, channels for creation/escrow, lifecycle for teardown.
// Grants scopes task operations to shared boot records plus the owner's dynamic range.
use cathedral_kernel_contracts::user as abi;
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
    pub memory: Option<usize>,
    pub notification: Option<u64>,
}
#[derive(Clone, Copy)]
struct Queued {
    bytes: [u8; abi::MAX_MESSAGE],
    len: usize,
    // Reserved, inaccessible recipient-local grant. Published only on receive.
    attachment: Option<usize>,
    memory: Option<usize>,
    mutable_memory: bool,
    notification: Option<usize>,
}
#[derive(Clone, Copy)]
struct Endpoint {
    epoch: u64,
    spec: EndpointSpec,
    message: Option<Queued>,
    sender_alive: bool,
    receiver_alive: bool,
    revoked: bool,
    // Persistent boot/launch account, independent of the current process incarnation.
    account: Option<usize>,
}
pub struct Ipc {
    pub notifications: crate::notifications::Notifications,
    epoch: u64,
    retired_memory: [u64; cathedral_kernel_contracts::memory::MAX_READ_HANDLES.div_ceil(64)],
    retired_memory_pending: bool,
    tasks: usize,
    alive: u32,
    // Hints only: the wait owner revalidates the current local grant and payload.
    changed_receivers: u32,
    endpoints: alloc::vec::Vec<Option<Endpoint>>,
    grants: Grants,
    reservations: [usize; MAX_TASKS],
    // Reserved once at boot, including dormant children; never grows in a trap.
    initial: alloc::vec::Vec<[u64; abi::MAX_INITIAL_ENDPOINTS]>,
}
impl Ipc {
    /// Coalesced receiver hints for the task wait coordinator. Taking these does
    /// not consume messages or confer authority; it must recheck the live grant.
    /// See user_tasks/execution/waits/waits.rs for completion and deadline order.
    pub(crate) fn take_changed_receivers(&mut self) -> u32 {
        core::mem::take(&mut self.changed_receivers) & self.alive
    }
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
            memory: None,
            mutable_memory: false,
            notification: None,
        };
        message.bytes[..bytes.len()].copy_from_slice(bytes);
        self.endpoints[index].as_mut().unwrap().message = Some(message);
        self.changed_receivers |= 1 << receiver;
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
        self.receive_kind(caller, ticket, capacity, if accept { 1 } else { 0 })
    }
    pub fn receive_memory(
        &mut self,
        caller: usize,
        ticket: u64,
        capacity: usize,
    ) -> Result<Option<Message>, u64> {
        self.receive_kind(caller, ticket, capacity, 2)
    }
    pub fn receive_shared(
        &mut self,
        caller: usize,
        ticket: u64,
        capacity: usize,
    ) -> Result<Option<Message>, u64> {
        self.receive_kind(caller, ticket, capacity, 4)
    }
    fn receive_kind(
        &mut self,
        caller: usize,
        ticket: u64,
        capacity: usize,
        kind: u8,
    ) -> Result<Option<Message>, u64> {
        let index = self.check(caller, ticket, Right::Receive)?;
        let endpoint = self.endpoints[index].as_mut().unwrap();
        if let Some(message) = endpoint.message {
            if message.len > capacity
                || (message.attachment.is_some() && kind != 1)
                || (message.memory.is_some() && kind != if message.mutable_memory { 4 } else { 2 })
                || (message.notification.is_some() && kind != 3)
            {
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
                memory: message.memory,
                notification: message
                    .notification
                    .map(|slot| self.notifications.publish(caller, slot)),
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
        self.changed_receivers |= 1 << self.endpoints[index].as_ref().unwrap().spec.receiver;
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
