//! Authenticate the input owner and validate the whole destination before device effects.
use super::*;
pub(in crate::user_tasks) fn virtual_pointer(
    session: &Session,
    slot: usize,
    command: u64,
    data: u64,
    address: u64,
) -> Result<u64, u64> {
    if owner(session) != Some(slot) {
        return Err(abi::DENIED);
    }
    let space = session.tasks[slot].space.as_ref().unwrap();
    if !space.writable_range(address, 16) {
        return Err(abi::BAD_ADDRESS);
    }
    let device = arch::board::devices().keyboard.ok_or(abi::DENIED)?;
    // SAFETY: Exclusive grant and full destination validation precede the effect.
    let words = unsafe { device.virtual_pointer(command, data) }
        .map_err(crate::user_tasks::syscalls::devices::error)?;
    let mut bytes = [0; 16];
    for (word, out) in words.iter().zip(bytes.chunks_exact_mut(4)) {
        out.copy_from_slice(&word.to_le_bytes());
    }
    // SAFETY: Checked writable range, kernel root and IRQs masked.
    assert!(unsafe { space.copy_to_user(address, &bytes) });
    Ok(0)
}
/// Restore relative input when the owner is retired, including after a crash.
pub(super) fn reset(device: &arch::board::Keyboard) {
    // SAFETY: Called with the same exclusive custody as PS/2 queue retirement.
    unsafe {
        if device
            .virtual_pointer(10, 0)
            .is_ok_and(|r| r[1] == 0x564d5868 && r[0] != u32::MAX)
        {
            let _ = device.virtual_pointer(41, 0xf5);
        }
    }
}
