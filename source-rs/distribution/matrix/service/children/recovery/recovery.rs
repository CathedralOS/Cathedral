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
        let launcher = self.launcher.as_ref().unwrap();
        let (placement, binding) = launcher.snapshot.expect("captured before retirement");
        let authority_matches =
            world.parent_matches(placement) && parent.storage_identity() == Some(binding);
        let vacant = self.slots[0].is_none() && server.available(0);
        match launcher
            .policy
            .action(authority_matches, vacant, launcher.result.is_some())
        {
            Action::Wait => return Ok(false),
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
                return Ok(false);
            }
            Action::RetireFallback => {
                // The request was acknowledged first. Remove context while stopping
                // fallback so its death cannot become another automatic attempt.
                let mut launcher = self.launcher.take().unwrap();
                self.stop(0, server, world, parent)?;
                launcher.policy.explicit_retired();
                self.launcher = Some(launcher);
                cathedral_user_runtime::write(b"Cathedral matrix: explicit launcher retry\n")?;
                return Ok(true);
            }
            Action::Fallback => return self.show_fallback(parent, world, server),
            Action::Read => {
                parent.storage_recover(launcher.plan)?;
                self.launcher.as_mut().unwrap().policy.read_started();
                return Ok(true);
            }
            Action::Admit => (),
        }
        let mut launcher = self.launcher.take().unwrap();
        let status = match launcher.result.take().unwrap() {
            Err(code) => Err(code),
            Ok(mut object) => {
                let admitted = self.run_object(launcher.plan, &mut object, parent, server, world);
                drop(object);
                match admitted {
                    Err(error) => Err(error.0 as u64),
                    Ok(false) => Err(abi::DENIED),
                    Ok(true) => match parent.storage_grant(server, 0, 0, 0)? {
                        Err(code) => Err(code),
                        Ok(()) if parent.storage_identity() != Some(binding) => Err(abi::DENIED),
                        Ok(()) => world.restore(placement, server.incarnation(0).unwrap(), true),
                    },
                }
            }
        };
        launcher.policy.read_finished(status.is_ok());
        if let Err(code) = status {
            if self.slots[0].is_some() {
                self.stop(0, server, world, parent)?;
            }
            time::storage_sample("launcher-recovery-refused", &[code]);
        } else {
            self.saved_storage[0] = Some((0, 0));
            launcher.task = self.slots[0].as_ref().unwrap().task.raw();
            launcher.incarnation = server.incarnation(0).unwrap();
            launcher.snapshot = None;
            self.management
                .grant(0, launcher.incarnation, self.active.unwrap_or(1))
                .map_err(|c| Error(c as i64))?;
            parent.grant_recovery(0, Some(launcher.incarnation));
            self.notify(world)?;
            cathedral_user_runtime::write(
                b"Cathedral matrix: launcher recovered; retry consumed\n",
            )?;
        }
        self.launcher = Some(launcher);
        Ok(true)
    }
}
