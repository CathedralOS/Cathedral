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
        let (words, len) = match (number, third) {
            (abi::CPU_READ, 8) => ([execution_us(session, slot), 0, 0], 8),
            (abi::CLOCK_READ, 8) => ([now, 0, 0], 8),
            // A trace stamp: the tick, the microsecond clock it is derived
            // from and this task's execution, in one read.
            (abi::CLOCK_READ, 24) => (
                [
                    now,
                    cathedral_arch::cpu_time_us(),
                    execution_us(session, slot),
                ],
                24,
            ),
            _ => return Err(abi::INVALID_ARGUMENT),
        };
        let mut record = [0; 24];
        for (bytes, word) in record.chunks_exact_mut(8).zip(words) {
            bytes.copy_from_slice(&word.to_le_bytes());
        }
        // SAFETY: Kernel CR3, IRQs off; copy validates the complete writable range.
        if !unsafe {
            session.tasks[slot]
                .space
                .as_ref()
                .unwrap()
                .copy_to_user(second, &record[..len])
        } {
            return Err(abi::BAD_ADDRESS);
        }
        return Ok(0);
    }
    Err(abi::UNKNOWN)
}
/// This account's measured execution since its launch origin, kernel work included.
fn execution_us(session: &mut Session, slot: usize) -> u64 {
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
}
