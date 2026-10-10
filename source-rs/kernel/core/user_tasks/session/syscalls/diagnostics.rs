//! Bounded diagnostic output; no filesystem or console-service semantics.
use super::Session;
use cathedral_contracts::user as abi;
pub(super) fn write(
    session: &mut Session,
    slot: usize,
    address: u64,
    length: u64,
) -> Result<u64, u64> {
    if length > abi::MAX_WRITE as u64 {
        return Err(abi::INVALID_ARGUMENT);
    }
    let mut buffer = [0; abi::MAX_WRITE];
    // SAFETY: Kernel root, IRQs off; full source range checked against owned pages.
    if !unsafe {
        session.tasks[slot]
            .space
            .as_ref()
            .unwrap()
            .copy_from_user(address, &mut buffer[..length as usize])
    } {
        return Err(abi::BAD_ADDRESS);
    }
    if !(session.output)(&buffer[..length as usize]) {
        return Err(abi::IO_ERROR);
    }
    session.tasks[slot].report.writes += 1;
    Ok(length)
}
