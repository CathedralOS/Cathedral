//! Scene completion keeps the launch ticket, task and previous selection in custody.
use super::*;
impl Children {
    pub(in crate::service) fn progress_launch_scene(
        &mut self,
        parent: &mut Parent,
        world: &mut World,
        server: &Server,
    ) -> Result<(), Error> {
        if !parent.scene_active() || self.launching.is_none() {
            return Ok(());
        }
        let mut l = self
            .launching
            .take()
            .expect("scene belongs to retained launch");
        if parent.scene_commit_ready()
            && l.policy.status() == 0
            && let Some(status) = self.launch_refusal(&l, parent, server)?
        {
            l.policy.cancel(status);
        }
        if l.policy.status() != 0 {
            parent.cancel_scene(l.policy.status());
        }
        if let Some(done) = parent.progress_scene()? {
            assert_eq!(done.ticket, l.ticket);
            l.scene_committed = done.committed;
            if done.committed && done.status == 0 && l.policy.status() == 0 {
                self.next_ticket = self
                    .next_ticket
                    .checked_add(1)
                    .ok_or(Error(abi::NO_MEMORY as i64))?;
                self.running[l.plan.slot] = Some((self.next_ticket, l.plan.object));
                self.commit_activation(l.plan.slot, world, server)?;
            }
            if l.policy.accepts(done.ticket, Command::Activate) {
                assert!(
                    l.policy
                        .complete(done.ticket, Command::Activate, done.status, false)
                );
            }
        }
        self.launching = Some(l);
        Ok(())
    }

    /// Owner mutation waits for visual custody to end, but its arrival can
    /// cancel an uncommitted launch. Child payloads never gain owner authority.
    pub(in crate::service) fn cancel_scene_event(&mut self, event: &Event, parent: &mut Parent) {
        if !parent.scene_active() || self.launching.is_none() {
            return;
        }
        if let Event::Request(r) = event
            && r.control
            && let Some(words) = wire::decode(&r.bytes[..r.len])
        {
            self.launch_owner_changed(words);
        }
        if let Event::Disconnected {
            client,
            incarnation,
        } = event
            && let Some(l) = &mut self.launching
            && l.request.client == Some(*client)
            && l.request.incarnation == *incarnation
        {
            l.policy.cancel(abi::PEER_CLOSED);
        }
        if let Some(l) = &self.launching
            && l.policy.status() != 0
        {
            parent.cancel_scene(l.policy.status());
        }
    }
    pub(super) fn restore_launch_scene(
        &mut self,
        l: &Pending,
        parent: &mut Parent,
        world: &mut World,
        server: &Server,
    ) -> Result<(), Error> {
        let slot = l.plan.slot;
        if l.scene_committed
            && (self.active == Some(slot) || self.active == l.previous)
            && (self.slots[slot].as_ref().map(|c| c.task.raw()) == l.task
                || self.slots[slot].is_none())
        {
            // A processed cancellation can race a submitted COMMIT.
            // Restore the previous selection before reclaiming this task.
            if let Some(previous) = l.previous
                && self.slots[previous].as_ref().map(|c| c.task.raw()) == l.previous_task
                && l.previous_task.is_some()
            {
                let status = if self.active == l.previous {
                    // A cancellation already known at the commit reply
                    // never changed local visibility or input routes.
                    parent.switch_scene(world, Some(slot), previous)?
                } else {
                    self.activate(previous, parent, world, server)?
                };
                if status != 0 {
                    return Err(Error(status as i64));
                }
                time::storage_sample("launch-scene-restored", &[l.ticket.raw()]);
            }
        }
        Ok(())
    }
}
