//! While publication or storage is pending, retain one dependent event
//! per stream. Backpressure keeps memory bounded and preserves each stream's FIFO.
use super::*;
use cathedral_user_runtime::server::Request;

pub(super) struct Deferred {
    events: [Option<Event>; 4],
}
impl Deferred {
    pub fn blocks(event: &Event, parent: &Parent) -> bool {
        if parent.upstream_busy() {
            return true;
        }
        if !parent.storing() {
            return false;
        }
        match event {
            Event::Request(r) => {
                if r.client == Some(parent.storing_client()) {
                    return true;
                }
                match wire::decode(&r.bytes[..r.len]) {
                    Some(w) if r.control => {
                        let hop = (w[0] >> wire::OP_BITS) & 3;
                        !(matches!(w[0], wire::HEALTH | wire::READ_PIXEL)
                            || matches!(hop, 1 | 2) && hop as usize != parent.storing_client() + 1)
                    }
                    Some(w) => {
                        w[0] == wire::STORAGE_INFO
                            || (wire::STORAGE_BASE..wire::STORAGE_BASE + 256)
                                .contains(&(w[0] & 0xffff))
                    }
                    None => false,
                }
            }
            Event::Memory(r, _) => r.client == Some(parent.storing_client()),
            Event::Disconnected { .. } => true,
        }
    }
    pub const fn empty() -> Self {
        Self {
            events: [const { None }; 4],
        }
    }
    pub fn take(&mut self, server: &mut Server) -> Option<Event> {
        for i in 0..self.events.len() {
            if let Some(event) = self.events[i].take() {
                server.pause(self.mask());
                return Some(event);
            }
        }
        None
    }
    pub fn hold(
        &mut self,
        event: Event,
        server: &mut Server,
        world: &mut World,
        blocked_clients: u8,
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
    fn mask(&self) -> u8 {
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
        if let Some(slot @ 0..=1) = request.client
            && matches!(
                words[0],
                wire::VIEW | wire::POLL | wire::NEXT | wire::SURFACE_INFO
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
