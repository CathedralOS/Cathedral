//! Retrieve only the framebuffer mapping installed by the boot grant.
use super::Session;
use cathedral_contracts::user as abi;
pub(super) fn information(
    session: &Session,
    slot: usize,
    address: u64,
    length: u64,
) -> Result<u64, u64> {
    let fb = session
        .launches
        .iter()
        .find(|launch| launch.model.child == slot)
        .and_then(|launch| launch.framebuffer)
        .ok_or(abi::DENIED)?;
    if length != cathedral_contracts::display::INFO_BYTES as u64 {
        return Err(abi::INVALID_ARGUMENT);
    }
    // SAFETY: Kernel root and IRQs off; only owned writable RAM is a copy target.
    if !unsafe {
        session.tasks[slot]
            .space
            .as_ref()
            .unwrap()
            .copy_to_user(address, &fb.user_info())
    } {
        return Err(abi::BAD_ADDRESS);
    }
    Ok(0)
}
