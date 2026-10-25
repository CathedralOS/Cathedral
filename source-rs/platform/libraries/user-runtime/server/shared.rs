//! Explicitly selected client migration. Endpoint identity remains authoritative;
//! transport request IDs only correlate replies within that exact incarnation.
use super::*;
use crate::{
    connection::{Accept, Peer},
    notification::Notification,
    time,
};
use cathedral_platform_contracts::connection as wire;
enum State {
    Armed,
    Opening(Accept),
    Ready(Peer),
}
pub(super) struct Shared {
    pub client: usize,
    state: State,
    last: u64,
    pending: u64,
}
impl Server {
    /// First migration supports one explicitly installed stream per server.
    /// Other streams retain their existing transport. No service name or PID lookup.
    pub fn enable_shared(&mut self, client: usize) -> Result<(), Error> {
        if self.shared.is_some()
            || self.count + 2 > abi::MAX_WAIT_HANDLES
            || client >= self.count
            || self.clients[client].is_none()
            || self.links[client].is_some()
        {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        self.shared = Some(Shared {
            client,
            state: State::Armed,
            last: 0,
            pending: 0,
        });
        Ok(())
    }
    pub(super) fn shared_active(&self, client: usize) -> bool {
        self.shared
            .as_ref()
            .is_some_and(|s| s.client == client && !matches!(s.state, State::Armed))
    }
    pub(super) fn shared_open(
        &mut self,
        client: usize,
        pair: (Handle, Handle),
        bytes: &[u8],
    ) -> bool {
        if !self
            .shared
            .as_ref()
            .is_some_and(|s| s.client == client && matches!(s.state, State::Armed))
            || bytes != wire::OPEN
        {
            return false;
        }
        match time::after(300).and_then(|d| Accept::start((pair.1, pair.0), d)) {
            Ok(accept) => self.shared.as_mut().unwrap().state = State::Opening(accept),
            Err(_) => self.disconnect(client),
        }
        true
    }
    pub(super) fn shared_poll(
        &mut self,
        client: usize,
        pair: (Handle, Handle),
    ) -> Result<Option<Event>, Error> {
        let shared = self.shared.as_mut().unwrap();
        assert_eq!(shared.client, client);
        match &mut shared.state {
            State::Armed => unreachable!(),
            State::Opening(accept) => {
                if let Some(peer) = accept.step()? {
                    shared.state = State::Ready(peer);
                }
                Ok(None)
            }
            State::Ready(peer) => {
                if shared.pending != 0 {
                    peer.check()?;
                    return Ok(None);
                }
                let Some(message) = peer.receive()? else {
                    return Ok(None);
                };
                let (id, bytes, len, object) =
                    (message.id, message.bytes, message.len, message.object);
                if object.is_some() && !self.memory {
                    return Err(Error(abi::DENIED as i64));
                }
                if shared.last.checked_add(1) != Some(id) {
                    return Err(Error(abi::IO_ERROR as i64));
                }
                shared.last = id;
                shared.pending = id;
                let request = Request {
                    bytes,
                    len,
                    control: false,
                    client: Some(client),
                    incarnation: pair.0.raw(),
                    reply: pair.1,
                    shared_id: id,
                };
                Ok(Some(match object {
                    Some(object) => Event::Memory(request, object),
                    None => Event::Request(request),
                }))
            }
        }
    }
    pub(super) fn shared_reply(
        &mut self,
        request: &Request,
        bytes: &[u8],
        object: Option<&crate::memory::Object>,
    ) -> Result<(), Error> {
        if bytes.len() > 64 {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        let Some(client) = request.client else {
            return Err(Error(abi::DENIED as i64));
        };
        if self.incarnation(client) != Some(request.incarnation) {
            return Ok(());
        }
        let Some(shared) = self
            .shared
            .as_mut()
            .filter(|s| s.client == client && s.pending == request.shared_id)
        else {
            return Ok(());
        };
        let State::Ready(peer) = &mut shared.state else {
            return Ok(());
        };
        match peer.send(request.shared_id, bytes, object) {
            Ok(()) => shared.pending = 0,
            // A refused grant published nothing. The provider still owns its
            // object and may send a plain refusal on this exact request.
            Err(failure) if object.is_some() && !failure.exposed => return Err(failure.error),
            Err(failure) => {
                self.disconnect(client);
                if object.is_some() {
                    return Err(failure.error);
                }
            }
        }
        Ok(())
    }
    pub(super) fn shared_wake(&self) -> Option<(usize, Notification)> {
        let shared = self.shared.as_ref()?;
        match &shared.state {
            State::Ready(peer) if shared.pending == 0 => Some((shared.client, peer.wake())),
            _ => None,
        }
    }
    pub(super) fn shared_deadline(&self) -> Option<u64> {
        match &self.shared.as_ref()?.state {
            State::Opening(accept) => Some(accept.deadline),
            State::Ready(peer) => peer.deadline(),
            State::Armed => None,
        }
    }
    pub(super) fn expire_shared(&mut self) -> Result<(), Error> {
        if let Some(deadline) = self.shared_deadline()
            && time::reached(time::now()?, deadline)
        {
            let client = self.shared.as_ref().unwrap().client;
            self.disconnect(client);
        }
        Ok(())
    }
}
