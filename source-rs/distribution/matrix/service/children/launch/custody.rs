//! Exact caller, slot and owner-change checks; no IPC handle enters Omega.
use super::*;
impl Children {
    pub(super) fn launch_refusal(
        &mut self,
        l: &Pending,
        parent: &Parent,
        server: &Server,
    ) -> Result<Option<u64>, Error> {
        if let Some(task) = l.task {
            if self.slots[l.plan.slot].as_ref().map(|c| c.task.raw()) != Some(task) {
                return Ok(Some(abi::BUSY));
            }
        } else if self.slots[l.plan.slot].is_some() || !server.available(l.plan.slot) {
            return Ok(Some(abi::BUSY));
        }
        if l.binding.is_some() && parent.storage_identity() != l.binding {
            return Ok(Some(abi::DENIED));
        }
        if !l.policy.picker_retired()
            && let Some(caller) = l.request.client
        {
            if server.incarnation(caller) != Some(l.request.incarnation)
                || self.slots[caller].as_ref().map(|c| c.task.raw()) != l.caller_task
                || self.completed(caller)?
            {
                return Ok(Some(abi::PEER_CLOSED));
            }
            if parent
                .storage_binding(server, caller)
                .map(|(_, scope)| scope)
                != Some(l.plan.stamp)
            {
                return Ok(Some(abi::DENIED));
            }
        }
        Ok(None)
    }
    pub(in crate::service) fn launch_owner_changed(&mut self, words: wire::Packet) {
        let Some(l) = &mut self.launching else {
            return;
        };
        let slot = words[1] as usize;
        if matches!(
            words[0],
            wire::BIND | wire::CONFIGURE | wire::RESTORE_STORAGE
        ) || ((slot == l.plan.slot || Some(slot) == l.request.client)
            && matches!(
                words[0],
                wire::STOP
                    | wire::PLACE
                    | wire::REVOKE
                    | wire::SURFACE_LIMIT
                    | wire::STORAGE_GRANT
                    | wire::STORAGE_REVOKE
                    | wire::LAUNCH_IMPORTS
                    | wire::LAUNCH_IMAGES
                    | wire::RUN_PAGES
                    | wire::RUN_MEMORY
                    | wire::LAUNCH_POLICY
                    | wire::LAUNCH_PLACE
                    | wire::SPAWN
                    | wire::RUN
            ))
        {
            l.policy.cancel(abi::DENIED);
        }
    }
    pub(super) fn finish_launch(
        &mut self,
        l: &mut Pending,
        server: &mut Server,
    ) -> Result<(), Error> {
        // The old reply endpoint must never disconnect or create a receipt for
        // a replacement connection in the same local slot.
        let status = l.policy.status();
        if status != 0 {
            time::storage_sample("stored-launch-refused", &[l.plan.slot as u64, status]);
        }
        if l.request
            .client
            .is_some_and(|caller| server.incarnation(caller) != Some(l.request.incarnation))
        {
            return Ok(());
        }
        let reply = if status != 0 {
            wire::error(status)
        } else if let Some(observer) = l.request.client {
            let receipt = self
                .observations
                .record(
                    observer,
                    l.request.incarnation,
                    l.task.expect("published task"),
                )
                .expect("checked receipt capacity");
            [0, l.plan.slot as u64, receipt, 0, 0, 0, 0, 0]
        } else {
            [0; 8]
        };
        server.reply(&l.request, &wire::encode(reply))
    }
}
