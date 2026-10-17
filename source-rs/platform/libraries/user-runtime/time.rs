//! Boot-local monotonic ticks, not wall time. The current timer profiles are nominally
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
/// Opt-in storage diagnostics: local scope, opcode, monotonic ticks, account CPU.
/// Scope values are local to each producer, never caller authentication or global IDs.
pub fn storage_trace(label: &str, scope: u64, opcode: u64) {
    storage_sample(
        label,
        &[scope, opcode, now().unwrap_or(0), cpu_us().unwrap_or(0)],
    );
}
/// Bounded numeric diagnostic record. Callers compile this out in ordinary builds.
pub fn storage_sample(label: &str, values: &[u64]) {
    let mut bytes = [0; 256];
    let mut len = 0;
    for text in ["STORAGE_TRACE ", label] {
        let count = text.len().min(32);
        bytes[len..len + count].copy_from_slice(&text.as_bytes()[..count]);
        len += count;
    }
    for &value in values.iter().take(8) {
        bytes[len] = b' ';
        len += 1;
        let mut digits = [b'0'; 20];
        let mut value = value;
        let mut start = 19;
        loop {
            digits[start] += (value % 10) as u8;
            value /= 10;
            if value == 0 {
                break;
            }
            start -= 1;
        }
        bytes[len..len + 20 - start].copy_from_slice(&digits[start..]);
        len += 20 - start;
    }
    bytes[len] = b'\n';
    let _ = crate::write(&bytes[..len + 1]);
}
/// Opt-in lab diagnostics. Call sites compile this out of ordinary service builds.
/// Ticks and execution samples are approximate and logging perturbs scheduling.
pub fn trace(label: &str) {
    let mut bytes = [0; 128];
    let mut len = 0;
    for text in ["INPUT_TRACE ", label] {
        let count = text.len().min(48);
        bytes[len..len + count].copy_from_slice(&text.as_bytes()[..count]);
        len += count;
    }
    for value in [now().unwrap_or(0), cpu_us().unwrap_or(0)] {
        bytes[len] = b' ';
        len += 1;
        let mut digits = [b'0'; 20];
        let mut value = value;
        let mut start = 19;
        loop {
            digits[start] += (value % 10) as u8;
            value /= 10;
            if value == 0 {
                break;
            }
            start -= 1;
        }
        bytes[len..len + 20 - start].copy_from_slice(&digits[start..]);
        len += 20 - start;
    }
    bytes[len] = b'\n';
    let _ = crate::write(&bytes[..len + 1]);
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
