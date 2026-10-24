// Experimental physical-key events. Text, layout, focus and shortcuts are not
// part of this transport. NEXT yields one event or IDLE; RESET clears held-key state.
// The accepted direction is wiki/spec/human_surface/input.md. These lab constants
// are not its production encoding and do not implement its full delivery contract.
// Logical channel identity/discovery is specified in input_channels.md beside it;
// description.rs owns bounded virtual-channel hints carried by matrix IPC.
pub const NEXT: [u8; 1] = [1];
pub const HEALTH: [u8; 1] = [2];
pub const F3: u8 = 9;
pub const F4: u8 = 10;
pub const F5: u8 = 11;
pub const F6: u8 = 12;
pub const F7: u8 = 13;
pub const F8: u8 = 14;
pub const RESET: u8 = 0;
pub const UP: u8 = 1;
pub const DOWN: u8 = 2;
pub const LEFT: u8 = 3;
pub const RIGHT: u8 = 4;
pub const ENTER: u8 = 5;
pub const F1: u8 = 6;
pub const F2: u8 = 7;
/// A completed NEXT with no physical event in the provider's sampling interval.
pub const IDLE: u8 = 8;
// Physical letter/digit positions; layout/text interpretation belongs to the app.
pub const A: u8 = 15;
pub const Z: u8 = A + 25;
pub const DIGIT_0: u8 = 41;
pub const DIGIT_9: u8 = DIGIT_0 + 9;
pub const SPACE: u8 = 51;
pub const BACKSPACE: u8 = 52;
pub const LAST_KEY: u8 = BACKSPACE;
pub const KEY_MASK: u64 = ((1u64 << LAST_KEY) - 1) & !(1 << (IDLE - 1));
pub const RELEASE: u8 = 0;
pub const PRESS: u8 = 1;
pub const REPEAT: u8 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Event {
    pub key: u8,
    pub state: u8,
}
impl Event {
    pub const fn idle() -> Self {
        Self {
            key: IDLE,
            state: RELEASE,
        }
    }
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
            [IDLE, RELEASE] => Some(Self::idle()),
            [key @ (UP..=F2 | F3..=LAST_KEY), state @ RELEASE..=REPEAT] => Some(Self {
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
        for key in (UP..=F2).chain(F3..=LAST_KEY) {
            for state in RELEASE..=REPEAT {
                let event = Event { key, state };
                assert_eq!(Event::decode(&event.encode()), Some(event));
            }
        }
        assert_eq!(Event::decode(&[0, 0]), Some(Event::reset()));
        assert_eq!(Event::decode(&[IDLE, RELEASE]), Some(Event::idle()));
        for bytes in [&[][..], &[1], &[1, 1, 0], &[0, 1], &[8, 1], &[1, 3]] {
            assert_eq!(Event::decode(bytes), None);
        }
    }
}
