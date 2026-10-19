//! Chrome sees only this matrix's admitted app instances. Local tickets survive
//! focus changes, never task replacement. Background has no workspace authority.
use super::*;
impl Children {
    pub(super) fn chrome(&self, request: &Request, world: &World) -> bool {
        self.desktop
            && request.client == Some(0)
            && self.slots[0]
                .as_ref()
                .is_some_and(|c| c.data.0.raw() == request.incarnation)
            && world.placement(0, request.incarnation).is_some()
    }
    pub(in crate::service) fn app_entry(
        &self,
        request: &Request,
        index: u64,
        world: &World,
    ) -> wire::Packet {
        if !self.chrome(request, world) || index >= 3 {
            return wire::error(abi::DENIED);
        }
        let slot = index as usize + 1;
        match self.running[slot] {
            Some((ticket, object)) => [
                0,
                ticket,
                object,
                u64::from(self.active == Some(slot)),
                0,
                0,
                0,
                0,
            ],
            None => [0; 8],
        }
    }
    pub(in crate::service) fn select_app(
        &mut self,
        request: &Request,
        ticket: u64,
        parent: &mut Parent,
        world: &mut World,
        server: &Server,
    ) -> Result<wire::Packet, Error> {
        if !self.chrome(request, world) {
            return Ok(wire::error(abi::DENIED));
        }
        let Some(slot) =
            (1..BACKGROUND).find(|s| self.running[*s].is_some_and(|(t, _)| t == ticket))
        else {
            return Ok(wire::error(abi::DENIED));
        };
        if parent.storing() || parent.upstream_busy() || self.loss.is_some() {
            return Ok(wire::error(abi::BUSY));
        }
        self.activate(slot, parent, world, server)?;
        Ok([0; 8])
    }
    pub(super) fn activate(
        &mut self,
        slot: usize,
        parent: &mut Parent,
        world: &mut World,
        server: &Server,
    ) -> Result<(), Error> {
        for sibling in 1..BACKGROUND {
            world
                .visible(sibling, sibling == slot)
                .map_err(|c| Error(c as i64))?;
            super::super::requests::replay(parent, world, sibling)?;
        }
        self.active = Some(slot);
        if let Some(incarnation) = server.incarnation(0) {
            self.management
                .grant(0, incarnation, slot)
                .map_err(|c| Error(c as i64))?;
        }
        world.notify(0);
        Ok(())
    }
}
