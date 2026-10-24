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
        Ok(wire::error(self.activate(slot, parent, world, server)?))
    }
    pub(super) fn activate(
        &mut self,
        slot: usize,
        parent: &mut Parent,
        world: &mut World,
        server: &Server,
    ) -> Result<u64, Error> {
        if self.active == Some(slot) {
            return Ok(0);
        }
        for sibling in 1..BACKGROUND {
            world
                .visible(sibling, sibling == slot)
                .map_err(|c| Error(c as i64))?;
        }
        let status = parent.switch_scene(world, self.active, slot)?;
        if status != 0 {
            for sibling in 1..BACKGROUND {
                world
                    .visible(sibling, self.active == Some(sibling))
                    .map_err(|c| Error(c as i64))?;
            }
            return Ok(status);
        }
        // This desktop's app-switch policy deactivates the former app's routes.
        // Visibility itself is neutral: other matrices may keep hidden control.
        if let Some(previous) = self.active {
            world
                .deactivate_input(previous)
                .map_err(|c| Error(c as i64))?;
        }
        self.active = Some(slot);
        if let Some(incarnation) = server.incarnation(0) {
            self.management
                .grant(0, incarnation, slot)
                .map_err(|c| Error(c as i64))?;
        }
        world.notify(0);
        Ok(0)
    }
}

impl Children {
    pub(super) fn place_surface_range(
        &self,
        world: &mut World,
        slot: usize,
        count: usize,
    ) -> Result<(), u64> {
        for base in 1..world.backed_surfaces().saturating_sub(7) {
            if world.surface_range(slot, base, count).is_ok() {
                return Ok(());
            }
        }
        Err(abi::NO_MEMORY)
    }
}
