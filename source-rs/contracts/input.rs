//! Experimental physical-key events. Text, layout, focus and shortcuts are not
//! part of this transport. NEXT consumes one event; RESET clears held-key state.
pub const NEXT: [u8; 1] = [1];
pub const RESET: u8 = 0;
pub const UP: u8 = 1;
pub const DOWN: u8 = 2;
pub const LEFT: u8 = 3;
pub const RIGHT: u8 = 4;
pub const ENTER: u8 = 5;
pub const F1: u8 = 6;
pub const F2: u8 = 7;
pub const RELEASE: u8 = 0;
pub const PRESS: u8 = 1;
pub const REPEAT: u8 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Event {
    pub key: u8,
    pub state: u8,
}
impl Event {
    pub const fn reset() -> Self {
        Self {
            key: RESET,
            state: RELEASE,
        }
    }
    pub fn encode(self) -> [u8; 2] {
        [self.key, self.state]
    }
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        match bytes {
            [RESET, RELEASE] => Some(Self::reset()),
            [key @ UP..=F2, state @ RELEASE..=REPEAT] => Some(Self {
                key: *key,
                state: *state,
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wire_rejects_unknown_keys_states_and_lengths() {
        for key in UP..=F2 {
            for state in RELEASE..=REPEAT {
                let event = Event { key, state };
                assert_eq!(Event::decode(&event.encode()), Some(event));
            }
        }
        assert_eq!(Event::decode(&[0, 0]), Some(Event::reset()));
        for bytes in [&[][..], &[1], &[1, 1, 0], &[0, 1], &[8, 1], &[1, 3]] {
            assert_eq!(Event::decode(bytes), None);
        }
    }
}
