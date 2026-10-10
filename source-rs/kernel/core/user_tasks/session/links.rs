//! Refresh only graph edges touching a newly admitted incarnation.
use super::{Session, next_epoch, syscalls};

pub(super) fn admit(session: &mut Session, slot: usize) {
    for link in &mut session.links {
        if link.model.admit(slot) {
            // Epoch admission is bounded alongside task admission. Exhaustion is
            // checked before mapping a new child, so this cannot wrap or reuse tickets.
            let epoch = next_epoch().expect("link epoch exhausted");
            session.ipc.prepare_pair(
                link.endpoint_base,
                link.model.spec.client,
                link.model.spec.service,
                epoch,
            );
        }
    }
    syscalls::wake_receivers(session);
}
