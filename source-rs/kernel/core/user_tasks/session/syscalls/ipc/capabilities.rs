//! Validate complete user records before creating objects or moving any authority.
use super::{Right, Session, abi};
pub(in super::super) fn create(
    session: &mut Session,
    slot: usize,
    address: u64,
    second: u64,
    third: u64,
) -> Result<u64, u64> {
    if second != 0 || third != 0 {
        return Err(abi::INVALID_ARGUMENT);
    }
    let space = session.tasks[slot].space.as_ref().unwrap();
    if !space.writable_range(address, 16) {
        return Err(abi::BAD_ADDRESS);
    }
    let handles = session.ipc.create(slot)?;
    let mut bytes = [0; 16];
    bytes[..8].copy_from_slice(&handles[0].to_le_bytes());
    bytes[8..].copy_from_slice(&handles[1].to_le_bytes());
    // SAFETY: Whole destination checked before create. Serialized kernel root; no remap.
    assert!(unsafe { space.copy_to_user(address, &bytes) });
    Ok(0)
}
pub(in super::super) fn send_cap(
    session: &mut Session,
    slot: usize,
    handle: u64,
    address: u64,
    reserved: u64,
) -> Result<u64, u64> {
    session.ipc.check(slot, handle, Right::Send)?;
    if reserved != 0 {
        return Err(abi::INVALID_ARGUMENT);
    }
    let mut packet = [0; abi::IPC_PACKET_BYTES];
    // SAFETY: Checked read from task-owned backing, kernel root, interrupts disabled.
    if !unsafe {
        session.tasks[slot]
            .space
            .as_ref()
            .unwrap()
            .copy_from_user(address, &mut packet)
    } {
        return Err(abi::BAD_ADDRESS);
    }
    let length = u64::from_le_bytes(packet[..8].try_into().unwrap());
    let attachment = u64::from_le_bytes(packet[8..16].try_into().unwrap());
    if length > abi::MAX_MESSAGE as u64 || attachment == 0 {
        return Err(abi::INVALID_ARGUMENT);
    }
    session.ipc.send_cap(
        slot,
        handle,
        &packet[16..16 + length as usize],
        Some(attachment),
    )?;
    session.tasks[slot].report.ipc_sent += 1;
    Ok(length)
}

pub(in super::super) fn budget(
    session: &mut Session,
    slot: usize,
    address: u64,
    second: u64,
    third: u64,
) -> Result<u64, u64> {
    if second != 0 || third != 0 {
        return Err(abi::INVALID_ARGUMENT);
    }
    let words = session.ipc.budget(slot)?;
    let mut bytes = [0; abi::IPC_BUDGET_BYTES];
    for (word, target) in words.into_iter().zip(bytes.chunks_exact_mut(8)) {
        target.copy_from_slice(&word.to_le_bytes());
    }
    // SAFETY: Copy validates the entire caller-owned destination under kernel CR3.
    if !unsafe {
        session.tasks[slot]
            .space
            .as_ref()
            .unwrap()
            .copy_to_user(address, &bytes)
    } {
        return Err(abi::BAD_ADDRESS);
    }
    Ok(0)
}
