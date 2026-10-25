// Capture policy before teardown, read stored code asynchronously, then reissue
// the launcher view. Exhaustion may admit owner-supplied fallback code in that
// same slot; only its authenticated request can start another read.
use super::*;
impl Children {
    pub(super) fn capture_recovery(&mut self, slot: usize, world: &World, parent: &Parent) {
        if slot != 0 {
            return;
        }
        let Some(launcher) = &mut self.launcher else {
            return;
        };
        if launcher.snapshot.is_some() {
            return;
        }
        if self.slots[0].as_ref().map(|child| child.task.raw()) != Some(launcher.task)
            || self.saved_storage[0] != Some((0, 0))
        {
            #[cfg(feature = "trace-storage")]
            time::storage_sample("launcher-recovery-identity", &[]);
            self.launcher = None;
            return;
        }
        match (
            world.placement(0, launcher.incarnation),
            parent.storage_identity(),
        ) {
            (Some(placement), Some(binding)) => launcher.snapshot = Some((placement, binding)),
            _ => {
                #[cfg(feature = "trace-storage")]
                time::storage_sample("launcher-recovery-no-authority", &[]);
                self.launcher = None;
            }
        }
    }
    pub(in crate::service) fn recovery_owner_changed(&mut self, words: wire::Packet) {
        if words[0] == wire::BIND
            || (words[1] == 0
                && matches!(
                    words[0],
                    wire::PLACE
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
                        | wire::MANAGEMENT_GRANT
                ))
        {
            self.launcher = None;
        }
    }
    pub(in crate::service) fn recovery_pending(&self) -> bool {
        self.launcher
            .as_ref()
            .is_some_and(|l| l.policy.ready(false))
    }
    pub(in crate::service) fn recover(
        &mut self,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<bool, Error> {
        let blocked = parent.upstream_busy()
            || parent.storing()
            || self.loss.is_some()
            || self.pending.iter().any(Option::is_some);
        if !self
            .launcher
            .as_ref()
            .is_some_and(|l| l.policy.ready(blocked))
        {
            return Ok(false);
        }
        parent.refresh(world)?;
        let launcher = self.launcher.as_mut().unwrap();
        let (placement, binding) = launcher.snapshot.expect("captured before retirement");
        let authority_matches =
            world.parent_matches(placement) && parent.storage_identity() == Some(binding);
        let vacant = self.slots[0].is_none() && server.available(0);
        match launcher.policy.action(authority_matches, vacant) {
            Action::Wait => Ok(false),
            Action::Cancel => {
                #[cfg(feature = "trace-storage")]
                time::storage_sample(
                    if authority_matches {
                        "launcher-recovery-slot-replaced"
                    } else {
                        "launcher-recovery-parent-changed"
                    },
                    &[],
                );
                self.launcher = None;
                Ok(false)
            }
            Action::RetireFallback => {
                // The request was acknowledged first. Remove context while stopping
                // fallback so its death cannot become another automatic attempt.
                let mut launcher = self.launcher.take().unwrap();
                self.stop(0, server, world, parent)?;
                launcher.policy.explicit_retired();
                self.launcher = Some(launcher);
                cathedral_user_runtime::write(b"Cathedral matrix: explicit launcher retry\n")?;
                Ok(true)
            }
            Action::Fallback => self.show_fallback(parent, world, server),
            Action::Read => {
                let Some(ticket) = self.operation_tickets.issue() else {
                    launcher.policy.read_unavailable();
                    time::storage_sample("launcher-recovery-ticket-exhausted", &[]);
                    return Ok(true);
                };
                launcher.policy.read_started(ticket);
                let plan = launcher.plan;
                if let Err(error) = parent.storage_recover(plan, ticket) {
                    self.complete_recovery(ticket, Err(error.0 as u64));
                }
                Ok(true)
            }
            Action::Admit(ticket) => self.admit_recovery(ticket, parent, world, server),
        }
    }
}
