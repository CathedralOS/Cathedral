//! Selection authority belongs to exact tasks and the authenticated chrome link.
use super::*;
use selection::Pending;

impl Children {
    pub(super) fn selection_refusal(
        &mut self,
        p: &Pending,
        world: &World,
        server: &Server,
    ) -> Result<Option<u64>, Error> {
        if !self.chrome(&p.request, world)
            || server.incarnation(0) != Some(p.request.incarnation)
            || self.slots[0].as_ref().map(|c| c.task.raw()) != Some(p.caller_task)
            || self.slots[p.slot].as_ref().map(|c| c.task.raw()) != Some(p.task)
            || self.active != p.previous.map(|(slot, _)| slot)
        {
            return Ok(Some(abi::DENIED));
        }
        if self.completed(0)? || self.completed(p.slot)? {
            return Ok(Some(abi::PEER_CLOSED));
        }
        if let Some((slot, task)) = p.previous
            && (self.slots[slot].as_ref().map(|c| c.task.raw()) != Some(task)
                || self.completed(slot)?)
        {
            return Ok(Some(abi::PEER_CLOSED));
        }
        Ok(None)
    }

    pub(in crate::service) fn cancel_selection_event(
        &mut self,
        event: &Event,
        parent: &mut Parent,
    ) {
        let Some(p) = &mut self.selecting else {
            return;
        };
        let relevant =
            |slot: usize| slot == 0 || slot == p.slot || p.previous.is_some_and(|(s, _)| s == slot);
        let cancelled = match event {
            Event::Request(r) if r.control => wire::decode(&r.bytes[..r.len]).is_some_and(|w| {
                matches!(w[0], wire::BIND | wire::CONFIGURE)
                    || relevant(w[1] as usize)
                        && matches!(
                            w[0],
                            wire::STOP | wire::PLACE | wire::REVOKE | wire::SURFACE_LIMIT
                        )
            }),
            Event::Disconnected {
                client,
                incarnation,
            } => {
                relevant(*client)
                    && self.slots[*client]
                        .as_ref()
                        .is_some_and(|c| c.data.0.raw() == *incarnation)
            }
            _ => false,
        };
        if cancelled && p.policy.cancel(abi::DENIED) {
            parent.cancel_scene(p.policy.status());
            time::storage_sample("selection-cancelled", &[p.ticket.raw()]);
        }
    }
}
