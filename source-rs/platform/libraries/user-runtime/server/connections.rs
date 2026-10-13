//! Admit explicit/local-link clients and retire incarnations before reconnecting.
use super::{Event, MAX_CLIENTS, Server};
use crate::{Error, abi, ipc::Handle, link::Link};

impl Server {
    pub fn set_control(&mut self, pair: (Handle, Handle)) {
        self.control = pair;
    }
    pub fn open() -> Result<Self, Error> {
        Self::open_clients(1)
    }
    pub fn open_clients(count: usize) -> Result<Self, Error> {
        Self::open_clients_from(0, count)
    }
    /// Reserve earlier links for an upstream provider; serve only this local range.
    pub fn open_clients_from(first: u64, count: usize) -> Result<Self, Error> {
        if count == 0 || count > MAX_CLIENTS {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        let mut links = [None; MAX_CLIENTS];
        for (index, slot) in links.iter_mut().enumerate().take(count) {
            *slot = match Link::at(
                first
                    .checked_add(index as u64)
                    .ok_or(Error(abi::INVALID_ARGUMENT as i64))?,
            ) {
                Ok(link) => Some(link),
                // No links is DENIED; an index beyond existing links is BAD_HANDLE.
                // Both represent an absent optional connection in this fixed budget.
                Err(Error(code))
                    if code == abi::DENIED as i64 || code == abi::BAD_HANDLE as i64 =>
                {
                    None
                }
                Err(error) => return Err(error),
            };
        }
        Ok(Self {
            control: (Handle::bootstrap(0)?, Handle::bootstrap(1)?),
            links,
            clients: [None; MAX_CLIENTS],
            count,
            cursor: 0,
            memory: false,
            closed: [None; MAX_CLIENTS],
        })
    }
    /// Whether a local slot can accept an explicitly supplied connection.
    pub fn available(&self, index: usize) -> bool {
        index < self.count && self.links[index].is_none() && self.clients[index].is_none()
    }
    /// Provider-local connection identity for explicit authority binding.
    pub fn incarnation(&self, index: usize) -> Option<u64> {
        self.clients.get(index).and_then(|p| *p).map(|p| p.0.raw())
    }
    /// Install an explicitly supplied client pair; no registry or global lookup.
    /// Caller owns placement/state invalidation before replacing a client.
    pub fn install(&mut self, index: usize, pair: (Handle, Handle)) -> Result<(), Error> {
        if !self.available(index) {
            return Err(Error(abi::BUSY as i64));
        }
        self.closed[index] = None;
        self.clients[index] = Some(pair);
        Ok(())
    }
    /// Detach a dynamic client. The caller closes its endpoints and invalidates state.
    pub fn remove(&mut self, index: usize) {
        assert!(index < self.count && self.links[index].is_none());
        self.clients[index] = None;
        self.closed[index] = None;
    }
    pub(super) fn disconnect(&mut self, client: usize) {
        if let Some(pair) = self.clients[client].take() {
            self.closed[client] = Some(pair.0.raw());
        }
    }
    pub(super) fn connect(&mut self) -> Result<(), Error> {
        for index in 0..self.count {
            if self.clients[index].is_none()
                && let Some(link) = self.links[index]
            {
                match link.connect() {
                    Ok(pair) => self.clients[index] = Some(pair),
                    Err(Error(code))
                        if code == abi::PEER_CLOSED as i64 || code == abi::BUSY as i64 => {}
                    Err(error) => return Err(error),
                }
            }
        }
        Ok(())
    }
    pub(super) fn take_disconnection(&mut self) -> Option<Event> {
        for client in 0..self.count {
            if let Some(incarnation) = self.closed[client].take() {
                return Some(Event::Disconnected {
                    client,
                    incarnation,
                });
            }
        }
        None
    }
}
