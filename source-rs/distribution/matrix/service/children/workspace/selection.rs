//! Dispatch source/distribution/matrix/service/children/workspace/main.omg commands while retaining
//! the chrome request and exact tasks until the scene commits or is restored.
use super::*;
use cathedral_stock_matrix::policy::{
    catalogue::Selection,
    operation::Ticket,
    workspace::{Command, State},
};

pub(in crate::service::children) struct Pending {
    pub(super) request: Request,
    pub(super) ticket: Ticket,
    pub(super) slot: usize,
    pub(super) task: u64,
    pub(super) caller_task: u64,
    pub(super) previous: Option<(usize, u64)>,
    pub(super) policy: State,
    command: Option<Command>,
}

impl Children {
    pub(in crate::service) fn select_app(
        &mut self,
        request: Request,
        app: u64,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        if !self.chrome(&request, world) {
            return server.reply(&request, &wire::encode(wire::error(abi::DENIED)));
        }
        let busy = parent.storing()
            || parent.upstream_busy()
            || self.loss.is_some()
            || self.launching.is_some()
            || self.selecting.is_some()
            || self.pending.iter().any(Option::is_some)
            || self.recovery_pending();
        let slot = match self.catalogue.select(app, busy) {
            Selection::Switch(slot) => slot,
            selection => {
                let status = match selection {
                    Selection::Missing => abi::DENIED,
                    Selection::Busy => abi::BUSY,
                    Selection::Current => 0,
                    Selection::Switch(_) => unreachable!(),
                };
                return server.reply(&request, &wire::encode(wire::error(status)));
            }
        };
        let Some(ticket) = self.operation_tickets.issue() else {
            return server.reply(&request, &wire::encode(wire::error(abi::NO_MEMORY)));
        };
        self.selecting = Some(Pending {
            request,
            ticket,
            slot,
            task: self.slots[slot].as_ref().unwrap().task.raw(),
            caller_task: self.slots[0].as_ref().unwrap().task.raw(),
            previous: self
                .catalogue
                .active()
                .map(|s| (s, self.slots[s].as_ref().unwrap().task.raw())),
            policy: State::new(ticket),
            command: None,
        });
        self.progress_selection(parent, world, server)
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
        loop {
            let command = p
                .command
                .take()
                .or_else(|| p.policy.command())
                .expect("live selection has a command");
            let (status, committed) = match command {
                Command::Begin => (
                    self.begin_selection_scene(&mut p, parent, world, server)?,
                    false,
                ),
                Command::Wait | Command::WaitRestore => {
                    let Some(outcome) =
                        self.poll_selection_scene(&mut p, command, parent, world, server)?
                    else {
                        p.command = Some(command);
                        self.selecting = Some(p);
                        return Ok(());
                    };
                    outcome
                }
                Command::Activate => {
                    let status = self
                        .commit_activation(p.slot, world, server)
                        .err()
                        .map_or(0, |e| e.0 as u64);
                    (status, false)
                }
                Command::Restore => {
                    // Retain both exact tasks until Omega observes restoration.
                    let previous = p.previous.map(|(slot, _)| slot);
                    let status = parent
                        .begin_scene(p.ticket, world, Some(p.slot), previous, false)
                        .err()
                        .map_or(0, |e| e.0 as u64);
                    time::storage_sample("selection-restoring", &[p.ticket.raw()]);
                    (status, false)
                }
                Command::Finish => {
                    assert!(p.policy.complete(p.ticket, command, 0, false));
                    return self.reply_selection(p, server);
                }
                Command::Failed => return Err(Error(abi::IO_ERROR as i64)),
            };
            assert!(p.policy.complete(p.ticket, command, status, committed));
        }
    }

    fn reply_selection(&self, p: Pending, server: &mut Server) -> Result<(), Error> {
        time::storage_sample("selection-complete", &[p.ticket.raw(), p.policy.status()]);
        // Never deliver an old selection result to a replacement chrome.
        if server.incarnation(0) == Some(p.request.incarnation)
            && self.slots[0].as_ref().map(|c| c.task.raw()) == Some(p.caller_task)
        {
            server.reply(&p.request, &wire::encode(wire::error(p.policy.status())))?;
        }
        Ok(())
    }
}
