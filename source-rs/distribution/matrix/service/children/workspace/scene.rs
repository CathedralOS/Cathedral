//! Execute Omega's scene commands while retaining authenticated task custody.
use super::*;
use cathedral_stock_matrix::policy::workspace::Command;
use selection::Pending;

impl Children {
    pub(super) fn begin_selection_scene(
        &mut self,
        p: &mut Pending,
        parent: &mut Parent,
        world: &World,
        server: &Server,
    ) -> Result<u64, Error> {
        if let Some(status) = self.selection_refusal(p, world, server)? {
            p.policy.cancel(status);
        }
        if p.policy.status() != 0 {
            return Ok(p.policy.status());
        }
        if let Err(e) = parent.begin_scene(
            p.ticket,
            world,
            self.active,
            Some(p.slot),
            cfg!(feature = "selection-delay"),
        ) {
            return Ok(e.0 as u64);
        }
        time::storage_sample("selection-started", &[p.ticket.raw(), p.slot as u64]);
        Ok(0)
    }

    pub(super) fn poll_selection_scene(
        &mut self,
        p: &mut Pending,
        command: Command,
        parent: &mut Parent,
        world: &World,
        server: &Server,
    ) -> Result<Option<(u64, bool)>, Error> {
        if command == Command::Wait {
            if p.policy.status() == 0
                && parent.scene_commit_ready()
                && let Some(status) = self.selection_refusal(p, world, server)?
            {
                p.policy.cancel(status);
            }
            if p.policy.status() != 0 {
                parent.cancel_scene(p.policy.status());
            }
        }
        let Some(done) = parent.progress_scene()? else {
            return Ok(None);
        };
        assert_eq!(done.ticket, p.ticket);
        if command == Command::Wait
            && p.policy.status() == 0
            && done.status == 0
            && let Some(status) = self.selection_refusal(p, world, server)?
        {
            p.policy.cancel(status);
        }
        if command == Command::WaitRestore && done.committed && done.status == 0 {
            time::storage_sample("selection-restored", &[p.ticket.raw()]);
        }
        Ok(Some((done.status, done.committed)))
    }
}
