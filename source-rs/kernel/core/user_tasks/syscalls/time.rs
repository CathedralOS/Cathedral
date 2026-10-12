//! Clock-granted tick and direct-CPU reads; authority precedes checked copyout.
use super::{Session, abi};
pub(super) fn read(
    session: &mut Session,
    slot: usize,
    [number, first, second, third]: [u64; 4],
    now: u64,
) -> Result<u64, u64> {
    if number == abi::CLOCK_HANDLE {
        return session.clock.handle(slot);
    }
    if number == abi::CLOCK_READ || number == abi::CPU_READ {
        session.clock.check(slot, first)?;
        if third != 8 {
            return Err(abi::INVALID_ARGUMENT);
        }
        let value = if number == abi::CPU_READ {
            session.scheduler.account_cpu(cathedral_arch::cpu_time_us());
            let origin = session
                .launches
                .iter()
                .find(|l| l.model.child == slot)
                .map_or(0, |l| l.cpu_origin);
            session
                .scheduler
                .cpu_report(slot)
                .direct_us
                .saturating_sub(origin)
        } else {
            now
        };
        // SAFETY: Kernel CR3, IRQs off; copy validates the complete writable range.
        if !unsafe {
            session.tasks[slot]
                .space
                .as_ref()
                .unwrap()
                .copy_to_user(second, &value.to_le_bytes())
        } {
            return Err(abi::BAD_ADDRESS);
        }
        return Ok(0);
    }
    Err(abi::UNKNOWN)
}
