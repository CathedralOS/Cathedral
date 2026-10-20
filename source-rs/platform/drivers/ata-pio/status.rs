//! Pure bounded ATA phase observation, exercised independently of guest syscalls.
// Data readiness is short; a durable host-backed flush may take longer under
// load. Both remain bounded, below the provider's end-to-end request deadline.
pub(crate) fn deadline(operation: u8) -> u64 {
    if operation == 0xe7 { 250 } else { 50 }
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum State {
    Ready,
    Wait,
    Failed,
    Timeout,
}
pub(crate) fn classify(status: u8, data: bool, elapsed: u64, limit: u64) -> State {
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
    if elapsed >= limit {
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
        assert_eq!(classify(0xa1, false, 49, 50), State::Wait);
        assert_eq!(classify(0xc0, false, 50, 50), State::Timeout);
        assert_eq!(classify(0x48, false, 49, 50), State::Wait);
        assert_eq!(classify(0x40, true, 50, 50), State::Timeout);
        assert_eq!(classify(0x48, true, 50, 50), State::Ready);
        assert_eq!(classify(0x40, false, 500, 50), State::Ready);
        assert_eq!(classify(0xc0, false, 50, deadline(0xe7)), State::Wait);
        assert_eq!(classify(0xc0, false, 250, deadline(0xe7)), State::Timeout);
        assert_eq!(classify(0x41, false, 50, deadline(0xe7)), State::Failed);
        for status in [0, 255, 0x41, 0x60] {
            assert_eq!(classify(status, false, 0, 50), State::Failed);
        }
    }
}
