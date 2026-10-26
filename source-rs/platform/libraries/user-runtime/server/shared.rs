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
    limit: usize,
    capacity: usize,
    pending: [u64; wire::MAX_IN_FLIGHT],
    outgoing: [Option<Reply>; wire::MAX_IN_FLIGHT],
}
enum ReplyGrant {
    Endpoint(Handle),
    Notification(Notification),
}
struct Reply {
    id: u64,
    bytes: [u8; 64],
    len: usize,
}

impl SharedSlot {
    pub const fn empty() -> Self {
        Self {
            state: State::Armed,
            last: 0,
            limit: 1,
            capacity: 1,
            pending: [0; wire::MAX_IN_FLIGHT],
            outgoing: [const { None }; wire::MAX_IN_FLIGHT],
        }
    }
    // Keep complete custody teardown out of repeated setup/removal call sites.
    #[inline(never)]
    fn reset(&mut self) {
        let capacity = self.capacity;
        *self = Self::empty();
        self.capacity = capacity;
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
    /// Select a local dispatch slot before or after installing its endpoints.
    /// An empty selected slot grants no authority. Its transport policy survives
    /// client replacement, but no old custody or request IDs survive.
    pub fn enable_shared(
        &mut self,
        client: usize,
        slot: &'static mut SharedSlot,
    ) -> Result<(), Error> {
        if client >= self.count {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        self.arm_shared(client + 1, slot)
    }
    /// Explicitly admit independently outstanding requests. The service must
    /// own each delivered Request until replying, pausing or removing its slot.
    /// Ordinary enable_shared slots reject this setup rather than expanding
    /// their application's deferred-work allowance on a caller's demand.
    pub fn enable_pipeline(
        &mut self,
        client: usize,
        slot: &'static mut SharedSlot,
    ) -> Result<(), Error> {
        self.enable_shared(client, slot)?;
        self.shared[client + 1].as_mut().unwrap().capacity = wire::MAX_IN_FLIGHT;
        Ok(())
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
        slot.capacity = 1;
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
            self.control_failed(Error(abi::PEER_CLOSED as i64))
        } else {
            self.disconnect(channel - 1);
            Ok(())
        }
    }
    pub(super) fn shared_ready(&self, channel: usize) -> bool {
        self.shared[channel]
            .as_ref()
            .is_some_and(|s| matches!(s.state, State::Ready(_)))
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
            || (bytes != wire::OPEN && bytes != wire::OPEN_PIPELINE)
        {
            return Ok(false);
        }
        let pipelined = bytes == wire::OPEN_PIPELINE;
        if pipelined && self.shared[channel].as_ref().unwrap().capacity == 1 {
            self.retire_shared(channel)?;
            return Ok(true);
        }
        self.shared[channel].as_mut().unwrap().limit =
            if pipelined { wire::MAX_IN_FLIGHT } else { 1 };
        match time::after(300).and_then(|d| Accept::start((pair.1, pair.0), d, pipelined)) {
            Ok(accept) => self.shared[channel].as_mut().unwrap().state = State::Opening(accept),
            Err(error) => {
                time::storage_sample("shared-open-refused", &[channel as u64, error.0 as u64]);
                if let Ok(b) = crate::resources::budget() {
                    time::storage_sample(
                        "shared-open-budget",
                        &[
                            b.limits.private_pages,
                            b.used.private_pages,
                            b.limits.reader_pages,
                            b.used.reader_pages,
                            b.limits.channels,
                            b.used.channels,
                        ],
                    );
                }
                if let Ok(b) = crate::notification::budget() {
                    time::storage_sample("shared-open-notifications", &b);
                }
                if let Ok(b) = crate::resources::physical_budget() {
                    time::storage_sample("shared-open-frames", &[b.reserved, b.charged]);
                }
                self.retire_shared(channel)?;
            }
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
                // Drain queued plain replies before accepting more work. A
                // peer acknowledgement wakes this loop without an app reply.
                for slot in &mut shared.outgoing {
                    let Some(reply) = slot else {
                        continue;
                    };
                    match peer.send(reply.id, &reply.bytes[..reply.len], None) {
                        Ok(()) => *slot = None,
                        Err(failure)
                            if !failure.exposed && failure.error.0 == abi::WOULD_BLOCK as i64 =>
                        {
                            break;
                        }
                        Err(failure) => return Err(failure.error),
                    }
                }
                if shared.pending.iter().filter(|id| **id != 0).count()
                    + shared.outgoing.iter().filter(|slot| slot.is_some()).count()
                    >= shared.limit
                {
                    peer.check()?;
                    return Ok(None);
                }
                let Some(pending) = shared.pending.iter_mut().find(|id| **id == 0) else {
                    peer.check()?;
                    return Ok(None);
                };
                let Some(message) = peer.receive()? else {
                    return Ok(None);
                };
                // This server dispatch API accepts bytes and immutable images.
                // Endpoint/notification custody is an explicit client-reply operation.
                if message.endpoint.is_some() || message.notification.is_some() {
                    return Err(Error(abi::DENIED as i64));
                }
                let (id, bytes, len, object) =
                    (message.id, message.bytes, message.len, message.object);
                if object.is_some() && !self.memory {
                    return Err(Error(abi::DENIED as i64));
                }
                if shared.last.checked_add(1) != Some(id) {
                    return Err(Error(abi::IO_ERROR as i64));
                }
                shared.last = id;
                *pending = id;
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
            .filter(|s| s.pending.contains(&request.shared_id))
        else {
            return Ok(());
        };
        let State::Ready(peer) = &mut shared.state else {
            return Ok(());
        };
        match peer.send(request.shared_id, bytes, object) {
            Ok(()) => {
                *shared
                    .pending
                    .iter_mut()
                    .find(|id| **id == request.shared_id)
                    .unwrap() = 0;
            }
            Err(failure)
                if object.is_none()
                    && !failure.exposed
                    && failure.error.0 == abi::WOULD_BLOCK as i64 =>
            {
                let Some(slot) = shared.outgoing.iter_mut().find(|slot| slot.is_none()) else {
                    // Bounded dispatch must stop before this is possible.
                    return Err(Error(abi::BUSY as i64));
                };
                let mut payload = [0; 64];
                payload[..bytes.len()].copy_from_slice(bytes);
                *slot = Some(Reply {
                    id: request.shared_id,
                    bytes: payload,
                    len: bytes.len(),
                });
                *shared
                    .pending
                    .iter_mut()
                    .find(|id| **id == request.shared_id)
                    .unwrap() = 0;
            }
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
    /// Err means the endpoint never left the caller. Once grant publication
    /// commits, even a later transport failure consumes custody and returns Ok.
    pub(super) fn shared_endpoint(
        &mut self,
        request: &Request,
        bytes: &[u8],
        endpoint: Handle,
    ) -> Result<(), Error> {
        self.shared_grant(request, bytes, ReplyGrant::Endpoint(endpoint))
    }
    pub(super) fn shared_notification(
        &mut self,
        request: &Request,
        bytes: &[u8],
        notification: Notification,
    ) -> Result<(), Error> {
        self.shared_grant(request, bytes, ReplyGrant::Notification(notification))
    }
    fn shared_grant(
        &mut self,
        request: &Request,
        bytes: &[u8],
        grant: ReplyGrant,
    ) -> Result<(), Error> {
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
            return Err(Error(abi::PEER_CLOSED as i64));
        }
        let shared = self.shared[channel]
            .as_mut()
            .filter(|s| s.pending.contains(&request.shared_id))
            .ok_or(Error(abi::PEER_CLOSED as i64))?;
        let State::Ready(peer) = &mut shared.state else {
            return Err(Error(abi::PEER_CLOSED as i64));
        };
        let result = match grant {
            ReplyGrant::Endpoint(endpoint) => {
                peer.send_endpoint(request.shared_id, bytes, endpoint)
            }
            ReplyGrant::Notification(notification) => {
                peer.send_notification(request.shared_id, bytes, notification)
            }
        };
        match result {
            Ok(()) => {
                *shared
                    .pending
                    .iter_mut()
                    .find(|id| **id == request.shared_id)
                    .unwrap() = 0;
                Ok(())
            }
            Err(failure) if !failure.exposed => Err(failure.error),
            Err(_) => {
                // Closing the connection releases queued escrow. A receiver may
                // already have accepted it; transport failure cannot undo that.
                let _ = self.retire_shared(channel);
                Ok(())
            }
        }
    }
    pub(super) fn shared_wake(&self, channel: usize) -> Option<Notification> {
        let shared = self.shared[channel].as_ref()?;
        match &shared.state {
            State::Ready(peer)
                if shared.pending.iter().filter(|id| **id != 0).count() < shared.limit
                    || shared.outgoing.iter().any(Option::is_some) =>
            {
                Some(peer.wake())
            }
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
