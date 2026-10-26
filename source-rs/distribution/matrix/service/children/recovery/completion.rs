//! Custody of ticketed read results and the host mechanisms behind Omega's admission command.
use super::*;
use cathedral_stock_matrix::policy::recovery::Ticket;
use cathedral_user_runtime::memory::Object;
impl Children {
    pub(in crate::service::children) fn complete_recovery(
        &mut self,
        ticket: Ticket,
        result: Result<Object, u64>,
    ) {
        let accepted = self
            .launcher
            .as_mut()
            .is_some_and(|launcher| launcher.policy.read_completed(ticket, result.is_ok()));
        if !accepted {
            // Dropping also releases any sealed image from the stale exchange.
            time::storage_sample("launcher-read-stale", &[ticket.raw()]);
            return;
        }
        match result {
            Ok(object) => self.launcher.as_mut().unwrap().result = Some((ticket, object)),
            Err(code) => time::storage_sample("launcher-recovery-refused", &[code]),
        }
    }
    pub(super) fn admit_recovery(
        &mut self,
        ticket: Ticket,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<bool, Error> {
        let mut launcher = self.launcher.take().unwrap();
        let (placement, binding) = launcher.snapshot.expect("captured before retirement");
        let (held, mut object) = launcher.result.take().expect("accepted read completion");
        // A command can use only the attachment actually held for this operation.
        assert_eq!(held, ticket, "recovery attachment custody");
        let admitted = self.run_object(launcher.plan, &mut object, parent, server, world);
        drop(object);
        let status = match admitted {
            Err(error) => Err(error.0 as u64),
            Ok(false) => Err(abi::DENIED),
            Ok(true) => match parent.storage_grant(server, 0, 0, 0)? {
                Err(code) => Err(code),
                Ok(()) if parent.storage_identity() != Some(binding) => Err(abi::DENIED),
                Ok(()) => world.restore(placement, server.incarnation(0).unwrap(), true),
            },
        };
        assert!(launcher.policy.admission_finished(ticket, status.is_ok()));
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
                .grant(
                    0,
                    launcher.incarnation,
                    self.catalogue.active().unwrap_or(1),
                )
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
