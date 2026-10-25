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
    /// Server dispatch index: control is zero, client n is n+1.
    pub channel: usize,
    state: State,
    last: u64,
    pending: u64,
}
impl Server {
    /// One explicitly selected shared stream per server; no global lookup.
    pub fn enable_shared(&mut self, client: usize) -> Result<(), Error> {
        if client >= self.count || self.clients[client].is_none() || self.links[client].is_some() {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        self.arm_shared(client + 1)
    }
    /// Upgrade the owner's supplied control stream, preserving control authority.
    pub fn enable_shared_control(&mut self) -> Result<(), Error> {
        self.arm_shared(0)
    }
    fn arm_shared(&mut self, channel: usize) -> Result<(), Error> {
        if self.shared.is_some() || self.count + 2 > abi::MAX_WAIT_HANDLES {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        self.shared = Some(Shared {
            channel,
            state: State::Armed,
            last: 0,
            pending: 0,
        });
        Ok(())
    }
    fn retire_shared(&mut self, channel: usize) -> Result<(), Error> {
        if channel == 0 {
            self.shared = None;
            Err(Error(abi::PEER_CLOSED as i64))
        } else {
            self.disconnect(channel - 1);
            Ok(())
        }
    }
    pub(super) fn shared_active(&self, channel: usize) -> bool {
        self.shared
            .as_ref()
            .is_some_and(|s| s.channel == channel && !matches!(s.state, State::Armed))
    }
    pub(super) fn shared_open(
        &mut self,
        channel: usize,
        pair: (Handle, Handle),
        bytes: &[u8],
    ) -> Result<bool, Error> {
        if !self
            .shared
            .as_ref()
            .is_some_and(|s| s.channel == channel && matches!(s.state, State::Armed))
            || bytes != wire::OPEN
        {
            return Ok(false);
        }
        match time::after(300).and_then(|d| Accept::start((pair.1, pair.0), d)) {
            Ok(accept) => self.shared.as_mut().unwrap().state = State::Opening(accept),
            Err(_) => self.retire_shared(channel)?,
        }
        Ok(true)
    }
    pub(super) fn shared_poll(
        &mut self,
        channel: usize,
        pair: (Handle, Handle),
    ) -> Result<Option<Event>, Error> {
        let shared = self.shared.as_mut().unwrap();
        assert_eq!(shared.channel, channel);
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
                    control: channel == 0,
                    client: channel.checked_sub(1),
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
        let channel = if request.control {
            0
        } else {
            request.client.ok_or(Error(abi::DENIED as i64))? + 1
        };
        let current = if channel == 0 {
            Some(self.control.0.raw())
        } else {
            self.incarnation(channel - 1)
        };
        if current != Some(request.incarnation) {
            return Ok(());
        }
        let Some(shared) = self
            .shared
            .as_mut()
            .filter(|s| s.channel == channel && s.pending == request.shared_id)
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
                self.retire_shared(channel)?;
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
            State::Ready(peer) if shared.pending == 0 => Some((shared.channel, peer.wake())),
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
            let channel = self.shared.as_ref().unwrap().channel;
            self.retire_shared(channel)?;
        }
        Ok(())
    }
}
