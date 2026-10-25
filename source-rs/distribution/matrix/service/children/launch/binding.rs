//! A refresh completion is not a grant: validate retained launch custody before committing it.
use super::*;
impl Children {
    pub(in crate::service) fn complete_binding(&mut self, bound: parent::Bound) {
        if let Some(l) = &mut self.launching
            && l.ticket == bound.ticket
            && l.bound.is_none()
            && l.policy.accepts(bound.ticket, Command::Bind)
        {
            l.bound = Some(bound.result);
        } else {
            time::storage_sample("launch-bind-stale", &[bound.ticket.raw()]);
        }
    }
    pub(super) fn finish_binding(
        &mut self,
        l: &mut Pending,
        parent: &mut Parent,
        server: &Server,
    ) -> Result<(), Error> {
        let Some(available) = l.bound.take() else {
            return Ok(());
        };
        // Cancelled work must never install a grant.
        if l.policy.status() != 0 {
            return Ok(());
        }
        // Keep the actual transport failure: retiring the stream also invalidates
        // authority, but that consequence must not hide a timeout as DENIED.
        let status = match available {
            Err(code) => code,
            Ok(available) => match self.launch_refusal(l, parent, server)? {
                Some(code) => code,
                None => parent
                    .install_storage_binding(server, l.plan.slot, 0, l.plan.scope, available)
                    .err()
                    .unwrap_or(0),
            },
        };
        if status == 0 {
            self.saved_storage[l.plan.slot] = Some((0, l.plan.scope));
            #[cfg(feature = "trace-storage")]
            time::storage_sample(
                "launch-bind-committed",
                &[l.plan.slot as u64, l.ticket.raw()],
            );
        }
        assert!(l.policy.complete(l.ticket, Command::Bind, status, false));
        Ok(())
    }
}
