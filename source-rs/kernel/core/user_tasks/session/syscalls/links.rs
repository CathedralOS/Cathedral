//! Accept boot-approved peer links; no launch or lifecycle authority.
use super::Session;
use cathedral_contracts::user as abi;
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
