//! Retain a chrome request until the prepared switch commits or is restored.
use super::*;
use cathedral_stock_matrix::policy::operation::Ticket;

pub(in crate::service::children) struct Pending {
    pub(super) request: Request,
    pub(super) ticket: Ticket,
    pub(super) slot: usize,
    pub(super) task: u64,
    pub(super) caller_task: u64,
    pub(super) previous: Option<(usize, u64)>,
    pub(super) status: u64,
    pub(super) restoring: bool,
}

impl Children {
    pub(in crate::service) fn select_app(
        &mut self,
        request: Request,
        app: u64,
        parent: &mut Parent,
        world: &World,
        server: &mut Server,
    ) -> Result<(), Error> {
        let slot =
            (1..BACKGROUND).find(|s| self.running[*s].is_some_and(|(ticket, _)| ticket == app));
        let status = if !self.chrome(&request, world) || slot.is_none() {
            Some(abi::DENIED)
        } else if parent.storing()
            || parent.upstream_busy()
            || self.loss.is_some()
            || self.launching.is_some()
            || self.selecting.is_some()
            || self.pending.iter().any(Option::is_some)
            || self.recovery_pending()
        {
            Some(abi::BUSY)
        } else if self.active == slot {
            Some(0)
        } else {
            None
        };
        if let Some(status) = status {
            return server.reply(&request, &wire::encode(wire::error(status)));
        }
        let slot = slot.unwrap();
        let Some(ticket) = self.operation_tickets.issue() else {
            return server.reply(&request, &wire::encode(wire::error(abi::NO_MEMORY)));
        };
        let mut pending = Pending {
            request,
            ticket,
            slot,
            task: self.slots[slot].as_ref().unwrap().task.raw(),
            caller_task: self.slots[0].as_ref().unwrap().task.raw(),
            previous: self
                .active
                .map(|s| (s, self.slots[s].as_ref().unwrap().task.raw())),
            status: 0,
            restoring: false,
        };
        if let Some(status) = self.selection_refusal(&pending, world, server)? {
            pending.status = status;
            return self.reply_selection(pending, server);
        }
        if let Err(error) = parent.begin_scene(
            ticket,
            world,
            self.active,
            Some(slot),
            cfg!(feature = "selection-delay"),
        ) {
            pending.status = error.0 as u64;
            return self.reply_selection(pending, server);
        }
        time::storage_sample("selection-started", &[ticket.raw(), slot as u64]);
        self.selecting = Some(pending);
        Ok(())
    }

    pub(in crate::service) fn progress_selection(
        &mut self,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        let Some(mut p) = self.selecting.take() else {
            return Ok(());
        };
        if !p.restoring
            && p.status == 0
            && parent.scene_commit_ready()
            && let Some(status) = self.selection_refusal(&p, world, server)?
        {
            p.status = status;
        }
        if !p.restoring && p.status != 0 {
            parent.cancel_scene(p.status);
        }
        let Some(done) = parent.progress_scene()? else {
            self.selecting = Some(p);
            return Ok(());
        };
        assert_eq!(done.ticket, p.ticket);
        if p.restoring {
            if !done.committed || done.status != 0 {
                return Err(Error(abi::IO_ERROR as i64));
            }
            time::storage_sample("selection-restored", &[p.ticket.raw()]);
            return self.reply_selection(p, server);
        }
        if p.status == 0 {
            p.status = done.status;
            if p.status == 0
                && let Some(status) = self.selection_refusal(&p, world, server)?
            {
                p.status = status;
            }
        }
        if done.committed && p.status == 0 {
            self.commit_activation(p.slot, world, server)?;
            return self.reply_selection(p, server);
        }
        if done.committed {
            // Local visibility/input never changed. Deferred mutations cannot
            // replace either task until restoration releases visual custody.
            let previous = p.previous.map(|(slot, _)| slot);
            parent.begin_scene(p.ticket, world, Some(p.slot), previous, false)?;
            p.restoring = true;
            time::storage_sample("selection-restoring", &[p.ticket.raw()]);
            self.selecting = Some(p);
            return Ok(());
        }
        self.reply_selection(p, server)
    }

    fn reply_selection(&self, p: Pending, server: &mut Server) -> Result<(), Error> {
        time::storage_sample("selection-complete", &[p.ticket.raw(), p.status]);
        // Never deliver an old selection result to a replacement chrome.
        if server.incarnation(0) == Some(p.request.incarnation)
            && self.slots[0].as_ref().map(|c| c.task.raw()) == Some(p.caller_task)
        {
            server.reply(&p.request, &wire::encode(wire::error(p.status)))?;
        }
        Ok(())
    }
}
