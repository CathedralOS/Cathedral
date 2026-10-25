//! While publication or storage is pending, retain one dependent event
//! per stream. Backpressure keeps memory bounded and preserves each stream's FIFO.
use super::*;
use cathedral_user_runtime::server::Request;

pub(super) struct Deferred {
    events: [Option<Event>; STREAMS],
}
impl Deferred {
    /// Retire old incarnation events before installing a replacement connection.
    pub fn prune(&mut self, server: &Server) {
        for event in &mut self.events {
            let stale = match event {
                Some(Event::Request(r) | Event::Memory(r, _)) => r
                    .client
                    .is_some_and(|slot| server.incarnation(slot) != Some(r.incarnation)),
                Some(Event::Disconnected {
                    client,
                    incarnation,
                }) => server
                    .incarnation(*client)
                    .is_some_and(|current| current != *incarnation),
                None => false,
            };
            if stale {
                *event = None;
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.events.iter().all(Option::is_none)
    }
    pub fn has_ready(&self, parent: &Parent) -> bool {
        self.events
            .iter()
            .flatten()
            .any(|event| !Self::blocks(event, parent))
    }
    pub fn blocks(event: &Event, parent: &Parent) -> bool {
        if let Some(slot) = parent.admitting {
            let client = match event {
                Event::Request(r) | Event::Memory(r, _) => r.client,
                Event::Disconnected { client, .. } => Some(*client),
            };
            if client == Some(slot) || client == Some(parent.storage_client(slot)) {
                return true;
            }
        }
        if parent.separate_storage
            && let Event::Request(r) | Event::Memory(r, _) = event
            && r.client.is_some_and(|client| parent.storage_stream(client))
        {
            return parent.storing();
        }
        if parent.upstream_busy() {
            return true;
        }
        if !parent.storing() {
            return false;
        }
        match event {
            Event::Request(r) => {
                if r.client.is_some() && r.client == parent.storing_client() {
                    return true;
                }
                match wire::decode(&r.bytes[..r.len]) {
                    Some(w) if r.control => {
                        let hop = (w[0] >> wire::OP_BITS) & 7;
                        !(matches!(
                            w[0],
                            wire::HEALTH
                                | wire::LOSS
                                | wire::READ_PIXEL
                                | wire::STORAGE_STATUS
                                | wire::SAVE_RECOVERY_POLL
                                | wire::STORAGE_BINDING
                        ) || (1..=CHILDREN as u64).contains(&hop)
                            && (parent.separate_storage
                                || parent
                                    .storing_client()
                                    .is_none_or(|client| hop as usize != client + 1)))
                    }
                    Some(w) => {
                        matches!(w[0], wire::STORAGE_INFO | wire::LAUNCH)
                            || (wire::STORAGE_BASE..wire::STORAGE_BASE + 256)
                                .contains(&(w[0] & 0xffff))
                    }
                    None => false,
                }
            }
            Event::Memory(r, _) => r.client.is_some() && r.client == parent.storing_client(),
            Event::Disconnected { .. } => true,
        }
    }
    pub const fn empty() -> Self {
        Self {
            events: [const { None }; STREAMS],
        }
    }
    pub fn take(&mut self, parent: &Parent) -> Option<Event> {
        for i in 0..self.events.len() {
            if self.events[i]
                .as_ref()
                .is_some_and(|event| !Self::blocks(event, parent))
            {
                return self.events[i].take();
            }
        }
        None
    }
    pub fn pause(&self, server: &mut Server, parent: &Parent) {
        // One catalog request at a time; unread requests remain in their bounded
        // queues. Exclude all child storage receivers while their shared upstream
        // exchange is active; interaction and owner replies remain serviceable.
        let storage = if parent.separate_storage && parent.storing() {
            ((1u32 << CHILDREN) - 1) << (STORAGE_BASE + 1)
        } else {
            0
        };
        server.pause(self.mask() | storage);
    }
    pub fn hold(
        &mut self,
        event: Event,
        server: &mut Server,
        world: &mut World,
        blocked_clients: u32,
    ) -> Result<(), Error> {
        if let Event::Request(request) = &event
            && request
                .client
                .is_none_or(|slot| blocked_clients & (1 << slot) == 0)
            && self.local(request, server, world)?
        {
            return Ok(());
        }
        let index = match &event {
            Event::Request(r) | Event::Memory(r, _) => r.client.map_or(0, |c| c + 1),
            Event::Disconnected { client, .. } => client + 1,
        };
        // Paused streams are neither read nor reconnected until this event retires.
        assert!(self.events[index].is_none());
        self.events[index] = Some(event);
        server.pause(self.mask());
        Ok(())
    }
    fn mask(&self) -> u32 {
        let mut mask = 0;
        for i in 0..self.events.len() {
            if self.events[i].is_some() {
                mask |= 1 << i;
            }
        }
        mask
    }
    fn local(
        &self,
        request: &Request,
        server: &mut Server,
        world: &mut World,
    ) -> Result<bool, Error> {
        let Some(words) = wire::decode(&request.bytes[..request.len]) else {
            return Ok(false);
        };
        if request.control && words == [wire::HEALTH, 0, 0, 0, 0, 0, 0, 0] {
            // Liveness is local. LOSS not handled by the read-only polling path
            // stays queued here until publication/admission can safely finish.
            server.reply(request, &wire::encode([0; 8]))?;
            return Ok(true);
        }
        if let Some(slot) = request.client
            && slot < CHILDREN
            && matches!(
                words[0],
                wire::VIEW | wire::POLL | wire::POLL_TAGGED | wire::NEXT | wire::SURFACE_INFO
            )
        {
            let response = world.request(slot, request.incarnation, words);
            server.reply(request, &wire::encode(response))?;
            return Ok(true);
        }
        // No authority-changing command runs against an in-flight geometry snapshot.
        Ok(false)
    }
}

/// Local diagnostic context only; handle names can repeat in other matrices.
#[cfg(feature = "trace-storage")]
pub(super) fn trace_loss(event: &Event, parent: &Parent) {
    if let Event::Request(r) = event
        && r.control
        && wire::decode(&r.bytes[..r.len]) == Some([wire::LOSS, 0, 0, 0, 0, 0, 0, 0])
    {
        cathedral_user_runtime::time::storage_sample(
            "matrix-loss-deferred",
            &[
                r.incarnation,
                cathedral_user_runtime::time::now().unwrap_or(0),
                u64::from(parent.publishing()),
                u64::from(parent.storing()),
            ],
        );
    }
}
