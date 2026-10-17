//! Pure bounded ATA phase observation, exercised independently of guest syscalls.
const DEADLINE: u64 = 50;
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum State {
    Ready,
    Wait,
    Failed,
    Timeout,
}
pub(crate) fn classify(status: u8, data: bool, elapsed: u64) -> State {
    if status == 0 || status == 255 {
        return State::Failed;
    }
    if status & 0x80 == 0 {
        if status & 0x21 != 0 {
            return State::Failed;
        }
        if (status & 8 != 0) == data {
            return State::Ready;
        }
    }
    if elapsed >= DEADLINE {
        State::Timeout
    } else {
        State::Wait
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn busy_bits_are_not_a_device_error_and_only_matching_phase_completes() {
        assert_eq!(classify(0xa1, false, 49), State::Wait);
        assert_eq!(classify(0xc0, false, 50), State::Timeout);
        assert_eq!(classify(0x48, false, 49), State::Wait);
        assert_eq!(classify(0x40, true, 50), State::Timeout);
        assert_eq!(classify(0x48, true, 50), State::Ready);
        assert_eq!(classify(0x40, false, 500), State::Ready);
        for status in [0, 255, 0x41, 0x60] {
            assert_eq!(classify(status, false, 0), State::Failed);
        }
    }
}
