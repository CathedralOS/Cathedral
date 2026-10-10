//! Exclusive fixed-controller transport. No block addressing or storage policy.
use super::Session;
use cathedral_contracts::user as abi;
pub(in super::super) fn syscall(
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
    let (number, first, second, third) = call;
    if number == abi::DISK_REGISTER_READ {
        if !(1..=8).contains(&first) {
            return Err(abi::INVALID_ARGUMENT);
        }
        // SAFETY: This principal owns the fixed controller. No arbitrary port access.
        return Ok(u64::from(unsafe { cathedral_arch::disk_read(first as u8) }));
    }
    if number == abi::DISK_REGISTER_WRITE {
        let valid = match first {
            2 => second == 1, // Only single-sector operations.
            3..=5 => second <= 255,
            6 => (0xe0..=0xef).contains(&second), // LBA28, master only.
            7 => matches!(second, 0x20 | 0x30 | 0xe7 | 0xec),
            8 => matches!(second, 2 | 6), // IRQs disabled, optional channel reset.
            _ => false,
        };
        if !valid {
            return Err(abi::INVALID_ARGUMENT);
        }
        // SAFETY: Exclusive controller; bounded whitelist excludes DMA and other devices.
        unsafe {
            cathedral_arch::disk_write(first as u8, second as u8);
        }
        return Ok(0);
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
    // SAFETY: Exclusive fixed controller, non-clearing alternate status read.
    if unsafe { cathedral_arch::disk_read(8) } & 0xa9 != 8 {
        return Err(abi::WOULD_BLOCK);
    }
    // SAFETY: DRQ set, no busy/error/device-fault; exactly one bounded sector.
    unsafe {
        cathedral_arch::disk_transfer(&mut bytes, second == 1);
    }
    if second == 0 {
        // SAFETY: Entire writable destination validated above; mappings cannot change here.
        assert!(unsafe { space.copy_to_user(first, &bytes) });
    }
    Ok(0)
}
