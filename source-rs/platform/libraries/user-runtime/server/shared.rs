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
/// Service-owned storage for one shared connection. Install once per selected
/// dispatch slot; disconnect/reconnect clears custody and correlation in place.
pub struct SharedSlot {
    state: State,
    last: u64,
    pending: u64,
}
impl SharedSlot {
    pub const fn empty() -> Self {
        Self {
            state: State::Armed,
            last: 0,
            pending: 0,
        }
    }
    fn reset(&mut self) {
        *self = Self::empty();
    }
    fn deadline(&self) -> Option<u64> {
        match &self.state {
            State::Opening(accept) => Some(accept.deadline),
            State::Ready(peer) => peer.deadline(),
            State::Armed => None,
        }
    }
}
impl Server {
    /// Select an explicit client or owner-supplied link. Its slot keeps the same
    /// transport policy across reconnection, but no old custody or request IDs.
    pub fn enable_shared(
        &mut self,
        client: usize,
        slot: &'static mut SharedSlot,
    ) -> Result<(), Error> {
        if client >= self.count || (self.clients[client].is_none() && self.links[client].is_none())
        {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        self.arm_shared(client + 1, slot)
    }
    pub fn enable_shared_control(&mut self, slot: &'static mut SharedSlot) -> Result<(), Error> {
        self.arm_shared(0, slot)
    }
    fn arm_shared(&mut self, channel: usize, slot: &'static mut SharedSlot) -> Result<(), Error> {
        let reserved = self.shared.iter().filter(|s| s.is_some()).count();
        // Every selected stream may need its own notification beside the full
        // lifetime endpoint set. Refuse before taking the caller's storage.
        if self.shared[channel].is_some() || self.count + 2 + reserved > abi::MAX_WAIT_HANDLES {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        slot.reset();
        self.shared[channel] = Some(slot);
        Ok(())
    }
    pub(super) fn reset_shared(&mut self, channel: usize) {
        if let Some(slot) = self.shared[channel].as_deref_mut() {
            slot.reset();
        }
    }
    fn retire_shared(&mut self, channel: usize) -> Result<(), Error> {
        if channel == 0 {
            self.reset_shared(0);
            Err(Error(abi::PEER_CLOSED as i64))
        } else {
            self.disconnect(channel - 1);
            Ok(())
        }
    }
    pub(super) fn shared_active(&self, channel: usize) -> bool {
        self.shared[channel]
            .as_ref()
            .is_some_and(|s| !matches!(s.state, State::Armed))
    }
    pub(super) fn shared_open(
        &mut self,
        channel: usize,
        pair: (Handle, Handle),
        bytes: &[u8],
    ) -> Result<bool, Error> {
        if !self.shared[channel]
            .as_ref()
            .is_some_and(|s| matches!(s.state, State::Armed))
            || bytes != wire::OPEN
        {
            return Ok(false);
        }
        match time::after(300).and_then(|d| Accept::start((pair.1, pair.0), d)) {
            Ok(accept) => self.shared[channel].as_mut().unwrap().state = State::Opening(accept),
            Err(_) => self.retire_shared(channel)?,
        }
        Ok(true)
    }
    pub(super) fn shared_poll(
        &mut self,
        channel: usize,
        pair: (Handle, Handle),
    ) -> Result<Option<Event>, Error> {
        let shared = self.shared[channel].as_mut().unwrap();
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
        let Some(shared) = self.shared[channel]
            .as_mut()
            .filter(|s| s.pending == request.shared_id)
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
    pub(super) fn shared_wake(&self, channel: usize) -> Option<Notification> {
        let shared = self.shared[channel].as_ref()?;
        match &shared.state {
            State::Ready(peer) if shared.pending == 0 => Some(peer.wake()),
            _ => None,
        }
    }
    pub(super) fn shared_deadline(&self) -> Option<u64> {
        self.shared
            .iter()
            .flatten()
            .filter_map(|s| s.deadline())
            .reduce(|a, b| if time::reached(a, b) { b } else { a })
    }
    pub(super) fn expire_shared(&mut self) -> Result<(), Error> {
        let Some(earliest) = self.shared_deadline() else {
            return Ok(());
        };
        let now = time::now()?;
        if time::reached(now, earliest) {
            for channel in 0..=self.count {
                if self.shared[channel]
                    .as_ref()
                    .and_then(|s| s.deadline())
                    .is_some_and(|d| time::reached(now, d))
                {
                    self.retire_shared(channel)?;
                }
            }
        }
        Ok(())
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        for slot in self.shared.iter_mut().flatten() {
            slot.reset();
        }
    }
}
