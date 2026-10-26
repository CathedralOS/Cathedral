//! Validate the caller and choose a locally authorized target before starting work.
use super::*;
impl Children {
    /// A child's request for a sibling. The other slot must be free and opened
    /// by the owner; the program is read with the requester's own scope, so a
    /// launcher can only launch what it can see.
    pub(in crate::service) fn launch(
        &mut self,
        request: Request,
        object: u64,
        withhold: u64,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        if self.launching.is_some() || self.loss.is_some() {
            return server.reply(&request, &wire::encode(wire::error(abi::BUSY)));
        }
        let requester = request.client.unwrap();
        let slot = if self.desktop {
            if !self.chrome(&request, world) {
                return server.reply(&request, &wire::encode(wire::error(abi::DENIED)));
            }
            let Some(slot) = (1..BACKGROUND).find(|s| self.slots[*s].is_none()) else {
                time::storage_sample("funding-local-slots-refused", &[3]);
                return server.reply(&request, &wire::encode(wire::error(abi::NO_MEMORY)));
            };
            slot
        } else if requester < 2 {
            1 - requester
        } else {
            return server.reply(&request, &wire::encode(wire::error(abi::DENIED)));
        };
        let refusal = if parent.storing() || self.slots[slot].is_some() || !server.available(slot) {
            Some(abi::BUSY)
        } else if !self.observations.available(requester) || !self.catalogue.has_ticket_capacity() {
            Some(abi::NO_MEMORY)
        } else if object == 0 || withhold & !cathedral_platform_contracts::imports::SUPPORTED != 0 {
            Some(abi::INVALID_ARGUMENT)
        } else {
            None
        };
        if let Some(code) = refusal {
            return server.reply(&request, &wire::encode(wire::error(code)));
        }
        let (Some(standing), Some((_, stamp))) = (
            self.standing[slot],
            parent.storage_binding(server, requester),
        ) else {
            return server.reply(&request, &wire::encode(wire::error(abi::DENIED)));
        };
        self.begin_launch(
            request,
            parent::LaunchRequest {
                slot,
                object,
                scope: object,
                argument: 0,
                channels: standing.channels,
                descendants: standing.descendants,
                cpu: standing.cpu,
                pages: Some(standing.pages),
                stamp,
                imports: self.allowed_imports[slot]
                    .unwrap_or(cathedral_platform_contracts::imports::ALL)
                    & !withhold,
            },
            parent,
            server,
        )
    }
}
