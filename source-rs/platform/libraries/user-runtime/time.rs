//! Boot-local monotonic ticks, not wall time. The current PIT profile is nominally
//! 100 Hz; deadlines use wrapping comparisons within half the u64 clock range.
use crate::{Error, abi, arch};

pub fn now() -> Result<u64, Error> {
    read(abi::CLOCK_READ)
}
/// Measured execution of this persistent account, including its kernel work.
/// Requires the existing clock grant; exposes no other account or machine total.
pub fn cpu_us() -> Result<u64, Error> {
    read(abi::CPU_READ)
}
fn read(operation: u64) -> Result<u64, Error> {
    let grant = arch::call(abi::CLOCK_HANDLE, 0, 0);
    if (grant as i64) < 0 {
        return Err(Error(grant as i64));
    }
    let mut ticks = 0u64;
    let status = arch::call3(operation, grant, (&raw mut ticks) as u64, 8);
    if (status as i64) < 0 {
        Err(Error(status as i64))
    } else {
        Ok(ticks)
    }
}
pub fn after(ticks: u64) -> Result<u64, Error> {
    if ticks > abi::MAX_INTERVAL {
        return Err(Error(abi::INVALID_ARGUMENT as i64));
    }
    Ok(now()?.wrapping_add(ticks))
}
pub fn reached(now: u64, deadline: u64) -> bool {
    now.wrapping_sub(deadline) <= abi::MAX_INTERVAL
}
