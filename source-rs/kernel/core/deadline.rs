//! Absolute deadlines on the lab's wrapping monotonic tick counter.
const HALF: u64 = 1 << 63;
use cathedral_contracts::user as abi;

/// Boot-bound read authority. Reserved child slots require an explicit grant; they never inherit access.
pub struct Clock {
    epoch: u64,
    readers: u8,
}
impl Clock {
    pub fn new(epoch: u64, tasks: usize, readers: &[usize]) -> Result<Self, u64> {
        if !(1..=crate::ipc::MAX_EPOCH).contains(&epoch) || tasks > crate::ipc::MAX_TASKS {
            return Err(abi::INVALID_ARGUMENT);
        }
        let mut clock = Self { epoch, readers: 0 };
        for &reader in readers {
            if reader >= tasks {
                return Err(abi::INVALID_ARGUMENT);
            }
            clock.readers |= 1 << reader;
        }
        Ok(clock)
    }
    pub fn handle(&self, caller: usize) -> Result<u64, u64> {
        if caller >= crate::ipc::MAX_TASKS || self.readers & (1 << caller) == 0 {
            return Err(abi::DENIED);
        }
        Ok((self.epoch << 16) | ((caller as u64) << 8) | 131)
    }
    pub fn check(&self, caller: usize, handle: u64) -> Result<(), u64> {
        if self.handle(caller)? == handle {
            Ok(())
        } else {
            Err(abi::BAD_HANDLE)
        }
    }
}

/// Exactly half a clock cycle is ambiguous and is rejected. Other values are
/// interpreted within the nearest half-cycle, including already-past deadlines.
pub fn valid(now: u64, deadline: u64) -> bool {
    deadline.wrapping_sub(now) != HALF
}
pub fn reached(now: u64, deadline: u64) -> bool {
    now.wrapping_sub(deadline) < HALF
}
/// Completion already available at observation wins over an expired deadline.
pub fn timed_out(now: u64, deadline: Option<u64>, ready: bool) -> bool {
    !ready && deadline.is_some_and(|deadline| reached(now, deadline))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clocks_require_explicit_caller_bound_boot_grants() {
        let clock = Clock::new(1, 3, &[0, 2]).unwrap();
        let ticket = clock.handle(0).unwrap();
        assert_eq!(clock.handle(1), Err(abi::DENIED));
        assert_eq!(clock.check(2, ticket), Err(abi::BAD_HANDLE));
        assert_eq!(
            Clock::new(2, 3, &[0]).unwrap().check(0, ticket),
            Err(abi::BAD_HANDLE)
        );
        assert!(Clock::new(1, 3, &[3]).is_err());
        assert!(Clock::new(0, 3, &[]).is_err());
    }
    #[test]
    fn deadlines_expire_at_equality_and_survive_wrap() {
        assert!(!reached(4, 5));
        assert!(reached(5, 5));
        assert!(reached(6, 5));
        assert!(!reached(u64::MAX - 2, 2));
        assert!(!reached(1, 2));
        assert!(reached(2, 2));
        assert!(reached(0, u64::MAX));
        assert!(!valid(7, 7 + HALF));
        assert!(valid(7, 6 + HALF));
    }
    #[test]
    fn timeout_does_not_override_ready_or_unbounded_wait() {
        assert!(timed_out(10, Some(10), false));
        assert!(!timed_out(10, Some(10), true));
        assert!(!timed_out(11, Some(10), true));
        assert!(!timed_out(11, None, false));
    }
}
