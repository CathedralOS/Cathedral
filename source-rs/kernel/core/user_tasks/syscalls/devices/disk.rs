//! Exclusive fixed-controller transport. No block addressing or storage policy.
use super::Session;
use cathedral_kernel_contracts::user as abi;
pub(in crate::user_tasks) fn syscall(
    session: &mut Session,
    slot: usize,
    call: (u64, u64, u64, u64),
) -> Result<u64, u64> {
    if !session
        .launches
        .iter()
        .any(|grant| grant.disk && grant.model.child == slot)
    {
        return Err(abi::DENIED);
    }
    let device = cathedral_arch::board::devices().disk.ok_or(abi::DENIED)?;
    let (number, first, second, third) = call;
    if number == abi::DISK_REGISTER_READ {
        // SAFETY: Caller owns this selected controller; adapter checks register restrictions.
        return unsafe { device.read(first) }
            .map(u64::from)
            .map_err(super::error);
    }
    if number == abi::DISK_REGISTER_WRITE {
        // SAFETY: Exclusive grant; adapter validates register/value before any device effect.
        return unsafe { device.write(first, second) }
            .map(|()| 0)
            .map_err(super::error);
    }
    if second > 1 || third != 512 {
        return Err(abi::INVALID_ARGUMENT);
    }
    let space = session.tasks[slot].space.as_ref().unwrap();
    let mut bytes = [0; 512];
    if second == 1 {
        // SAFETY: Kernel root; checked full source copy before any device effect.
        if !unsafe { space.copy_from_user(first, &mut bytes) } {
            return Err(abi::BAD_ADDRESS);
        }
    } else if !space.writable_range(first, bytes.len()) {
        return Err(abi::BAD_ADDRESS);
    }
    // SAFETY: Controller grant and complete user-buffer validation precede any device effect.
    // The selected adapter checks readiness and enforces the bounded transfer itself.
    unsafe { device.transfer(&mut bytes, second == 1) }.map_err(super::error)?;
    if second == 0 {
        // SAFETY: Entire writable destination validated above; mappings cannot change here.
        assert!(unsafe { space.copy_to_user(first, &bytes) });
    }
    Ok(0)
}
