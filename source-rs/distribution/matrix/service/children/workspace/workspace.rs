// Authenticate chrome before querying Omega's catalogue of local app instances.
// source/distribution/matrix/workspace/main.omg owns tickets and selection.
// SELECT_APP enters selection.rs; custody.rs checks its exact tasks and caller.
// activate below remains the synchronous cleanup helper; commit_activation is
// the shared local handoff after either visual path succeeds.
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
        if !self.chrome(request, world) {
            return wire::error(abi::DENIED);
        }
        self.catalogue
            .entry(index)
            .unwrap_or_else(|| wire::error(abi::DENIED))
    }
    pub(super) fn activate(
        &mut self,
        slot: usize,
        parent: &mut Parent,
        world: &mut World,
        server: &Server,
    ) -> Result<u64, Error> {
        if self.catalogue.active() == Some(slot) {
            return Ok(0);
        }
        for sibling in 1..BACKGROUND {
            world
                .visible(sibling, sibling == slot)
                .map_err(|c| Error(c as i64))?;
        }
        let status = parent.switch_scene(world, self.catalogue.active(), slot)?;
        if status != 0 {
            for sibling in 1..BACKGROUND {
                world
                    .visible(sibling, self.catalogue.active() == Some(sibling))
                    .map_err(|c| Error(c as i64))?;
            }
            return Ok(status);
        }
        self.commit_activation(slot, world, server)?;
        Ok(0)
    }
    pub(super) fn commit_activation(
        &mut self,
        slot: usize,
        world: &mut World,
        server: &Server,
    ) -> Result<(), Error> {
        for sibling in 1..BACKGROUND {
            world
                .visible(sibling, sibling == slot)
                .map_err(|c| Error(c as i64))?;
        }
        // This desktop's app-switch policy deactivates the former app's routes.
        // Visibility itself is neutral: other matrices may keep hidden control.
        if let Some(previous) = self.catalogue.active() {
            world
                .deactivate_input(previous)
                .map_err(|c| Error(c as i64))?;
        }
        assert!(
            self.catalogue.activate(slot),
            "activation requires a published instance"
        );
        if let Some(incarnation) = server.incarnation(0) {
            self.management
                .grant(0, incarnation, slot)
                .map_err(|c| Error(c as i64))?;
        }
        world.notify(0);
        Ok(())
    }
}

impl Children {
    pub(super) fn place_surface_range(
        &self,
        world: &mut World,
        slot: usize,
        count: usize,
    ) -> Result<(), u64> {
        for base in 1..world
            .backed_surfaces()
            .saturating_sub(wire::config::LAUNCHER_SURFACES)
        {
            if world.surface_range(slot, base, count).is_ok() {
                return Ok(());
            }
        }
        Err(abi::NO_MEMORY)
    }
}
