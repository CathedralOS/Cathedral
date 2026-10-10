//! Refresh only graph edges touching a newly admitted incarnation.
use super::{Session, dispatch, next_epoch};
use cathedral_contracts::user as abi;

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
    dispatch::wake_receivers(session);
}
pub(super) fn syscall(
    session: &mut Session,
    slot: usize,
    number: u64,
    first: u64,
) -> Result<u64, u64> {
    if number == abi::LINK_HANDLE {
        let mut ports = session
            .links
            .iter()
            .filter_map(|link| link.model.port(slot).ok());
        let first_port = ports.next().ok_or(abi::DENIED)?;
        return if first == 0 {
            Ok(first_port)
        } else {
            ports.nth((first - 1) as usize).ok_or(abi::BAD_HANDLE)
        };
    }
    let link = session
        .links
        .iter_mut()
        .find(|link| link.model.port(slot) == Ok(first))
        .ok_or(abi::BAD_HANDLE)?;
    link.model.accept(slot, first)?;
    Ok(session.ipc.accept_child(link.endpoint_base, slot))
}
