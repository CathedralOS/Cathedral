// Omega sequences ordinary read/admit/bind/place/publish and failure rollback.
// request.rs establishes authority; custody.rs rejects stale work; steps.rs owns mechanisms.
use super::*;
use cathedral_stock_matrix::policy::{
    launch::{Command, State},
    operation::Ticket,
};
use cathedral_user_runtime::memory::Object;
pub(super) struct Pending {
    request: Request,
    plan: parent::LaunchRequest,
    ticket: Ticket,
    policy: State,
    image: Option<Object>,
    task: Option<u64>,
    caller_task: Option<u64>,
    selection: bool,
    binding: Option<[u64; 4]>,
    #[cfg(any(feature = "launch-bind-failure", feature = "launch-place-failure"))]
    probe: bool,
}
impl Children {
    pub(super) fn begin_launch(
        &mut self,
        request: Request,
        plan: parent::LaunchRequest,
        parent: &Parent,
        server: &mut Server,
    ) -> Result<(), Error> {
        if self.launching.is_some() || parent.storing() {
            return server.reply(&request, &wire::encode(wire::error(abi::BUSY)));
        }
        let Some(ticket) = self.operation_tickets.issue() else {
            return server.reply(&request, &wire::encode(wire::error(abi::NO_MEMORY)));
        };
        let selection = self.login && !self.entered && plan.slot == 1;
        let desktop = self.desktop && request.client == Some(0);
        #[cfg(any(feature = "launch-bind-failure", feature = "launch-place-failure"))]
        let probe = desktop && plan.slot == 2 && !self.launch_probe_used;
        #[cfg(any(feature = "launch-bind-failure", feature = "launch-place-failure"))]
        {
            self.launch_probe_used |= probe;
        }
        self.launching = Some(Pending {
            caller_task: request
                .client
                .and_then(|s| self.slots.get(s))
                .and_then(Option::as_ref)
                .map(|c| c.task.raw()),
            request,
            plan,
            ticket,
            policy: State::new(ticket, selection, request.client.is_some(), desktop),
            image: None,
            task: None,
            selection,
            binding: parent.storage_identity(),
            #[cfg(any(feature = "launch-bind-failure", feature = "launch-place-failure"))]
            probe,
        });
        Ok(())
    }
    pub(in crate::service) fn complete_launch(
        &mut self,
        launched: parent::Launched,
    ) -> Result<(), Error> {
        match launched.origin {
            parent::LaunchOrigin::Recovery(ticket) => {
                self.complete_recovery(ticket, launched.result)
            }
            parent::LaunchOrigin::Caller(_, ticket) => {
                if let Some(l) = self.launching.as_mut()
                    && l.ticket == ticket
                    && l.policy.complete(
                        ticket,
                        Command::Read,
                        launched.result.as_ref().err().copied().unwrap_or(0),
                        false,
                    )
                {
                    l.image = launched.result.ok();
                } else {
                    time::storage_sample("launch-read-stale", &[ticket.raw()]);
                }
            }
        }
        Ok(())
    }
    pub(in crate::service) fn progress_launch(
        &mut self,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<bool, Error> {
        if parent.storing() || parent.upstream_busy() {
            return Ok(false);
        }
        let Some(mut l) = self.launching.take() else {
            return Ok(false);
        };
        // All host steps except the read currently finish in this turn. They never
        // dispatch another inbound owner request behind the workflow's back.
        while let Some(command) = l.policy.command() {
            if !matches!(
                command,
                Command::Rollback | Command::RestorePicker | Command::Finish
            ) && l.policy.status() == 0
                && let Some(code) = self.launch_refusal(&l, parent, server)?
            {
                l.policy.cancel(code);
                continue;
            }
            if command == Command::Read {
                match parent.storage_launch(l.request, l.plan, l.ticket) {
                    Ok(()) => {
                        self.launching = Some(l);
                        return Ok(true);
                    }
                    Err(e) => {
                        assert!(l.policy.complete(l.ticket, command, e.0 as u64, false));
                        continue;
                    }
                }
            }
            if command == Command::Finish {
                self.finish_launch(&mut l, server)?;
                assert!(l.policy.complete(l.ticket, command, 0, false));
                return Ok(true);
            }
            let result = self.launch_step(command, &mut l, parent, world, server);
            // Teardown failures cannot be acknowledged as successful reclamation.
            if matches!(
                command,
                Command::RetirePicker | Command::Rollback | Command::RestorePicker
            ) {
                result.as_ref().map_err(|e| Error(e.0))?;
            }
            let (status, storage) = match result {
                Ok(storage) => (0, storage),
                Err(e) => (e.0 as u64, false),
            };
            assert!(l.policy.complete(l.ticket, command, status, storage));
        }
        self.launching = Some(l);
        Ok(false)
    }
}
