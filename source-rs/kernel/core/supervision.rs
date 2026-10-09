//! Bounded launch authority and child lifecycle, independent of address spaces.
//! One boot-issued launch grant, one consenting peer port, one child at a time.
use crate::ipc::{MAX_EPOCH, MAX_TASKS};
use cathedral_contracts::user as abi;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Child {
    Empty,
    Starting(u64),
    Live(u64),
    Reaped(u64, [u8; abi::EXIT_BYTES]),
}
pub struct Supervisor {
    pub owner: usize,
    pub peer: usize,
    pub child: usize,
    epoch: u64,
    latest_epoch: u64,
    state: Child,
    connected: bool,
    owner_alive: bool,
    peer_alive: bool,
    cancellation: bool,
}
impl Supervisor {
    pub fn new(epoch: u64, owner: usize, peer: usize, child: usize) -> Result<Self, u64> {
        if !(1..=MAX_EPOCH).contains(&epoch)
            || child >= MAX_TASKS
            || owner >= child
            || peer >= child
            || owner == peer
        {
            return Err(abi::INVALID_ARGUMENT);
        }
        Ok(Self {
            owner,
            peer,
            child,
            epoch,
            latest_epoch: epoch,
            state: Child::Empty,
            connected: false,
            owner_alive: true,
            peer_alive: true,
            cancellation: false,
        })
    }
    fn ticket(epoch: u64, owner: usize, kind: u64) -> u64 {
        (epoch << 16) | ((owner as u64) << 8) | kind
    }
    pub fn launch(&self, caller: usize) -> Result<u64, u64> {
        if caller != self.owner || !self.owner_alive {
            return Err(abi::DENIED);
        }
        Ok(Self::ticket(self.epoch, caller, 128))
    }
    pub fn port(&self, caller: usize) -> Result<u64, u64> {
        if caller != self.peer || !self.peer_alive {
            return Err(abi::DENIED);
        }
        Ok(Self::ticket(self.epoch, caller, 130))
    }
    pub fn request(&mut self, caller: usize, grant: u64, argument: u64) -> Result<(), u64> {
        if self.launch(caller)? != grant {
            return Err(abi::BAD_HANDLE);
        }
        if !self.peer_alive {
            return Err(abi::PEER_CLOSED);
        }
        if self.state != Child::Empty {
            return Err(abi::BUSY);
        }
        self.state = Child::Starting(argument);
        Ok(())
    }
    pub fn pending(&self) -> Option<u64> {
        if let Child::Starting(argument) = self.state {
            Some(argument)
        } else {
            None
        }
    }
    pub fn started(&mut self, epoch: u64) -> u64 {
        assert!(self.pending().is_some() && epoch > self.latest_epoch && epoch <= MAX_EPOCH);
        self.latest_epoch = epoch;
        let ticket = Self::ticket(epoch, self.owner, 129);
        self.state = Child::Live(ticket);
        self.connected = false;
        ticket
    }
    pub fn failed(&mut self) {
        assert!(self.pending().is_some());
        self.state = Child::Empty;
    }
    /// A completed wait consumes the outcome. Failed copyout must not call this.
    pub fn consume(&mut self) {
        assert!(matches!(self.state, Child::Reaped(..)));
        self.state = Child::Empty;
    }
    pub fn wait(&self, caller: usize, ticket: u64) -> Result<Option<[u8; abi::EXIT_BYTES]>, u64> {
        if caller != self.owner || !self.owner_alive {
            return Err(abi::DENIED);
        }
        match self.state {
            Child::Live(current) if current == ticket => Ok(None),
            Child::Reaped(current, outcome) if current == ticket => Ok(Some(outcome)),
            _ => Err(abi::BAD_HANDLE),
        }
    }
    pub fn reaped(&mut self, outcome: [u8; abi::EXIT_BYTES]) {
        let Child::Live(ticket) = self.state else {
            panic!("reaping absent child")
        };
        self.state = Child::Reaped(ticket, outcome);
        self.cancellation = false;
    }
    pub fn connect(&mut self, caller: usize, port: u64) -> Result<(), u64> {
        if self.port(caller)? != port {
            return Err(abi::BAD_HANDLE);
        }
        if !matches!(self.state, Child::Live(_)) {
            return Err(abi::PEER_CLOSED);
        }
        if self.connected {
            return Err(abi::BUSY);
        }
        self.connected = true;
        Ok(())
    }
    pub fn close(&mut self, task: usize) {
        if task == self.owner {
            self.owner_alive = false;
        }
        if task == self.peer {
            self.peer_alive = false;
        }
    }
    pub fn cancel_child(&self) -> bool {
        (!self.owner_alive || self.cancellation) && matches!(self.state, Child::Live(_))
    }
    /// Cancel only this owner's exact incarnation. A reaped outcome is preserved.
    pub fn request_cancel(&mut self, caller: usize, ticket: u64) -> Result<bool, u64> {
        if self.wait(caller, ticket)?.is_some() {
            return Ok(false);
        }
        self.cancellation = true;
        Ok(true)
    }
    pub fn cancellation_pending(&self) -> bool {
        self.cancellation
    }
}

#[cfg(test)]
mod tests;
