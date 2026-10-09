#![no_std]
#![forbid(unsafe_code)]
//! Small translated-set-1 decoder for the lab's physical navigation keys.
use cathedral_contracts::input::{self as wire, Event};

#[derive(Default)]
pub struct Decoder {
    extended: bool,
    pause: u8,
    held: u8,
}
impl Decoder {
    pub fn reset(&mut self) -> Event {
        *self = Self::default();
        Event::reset()
    }
    pub fn feed(&mut self, byte: u8) -> Option<Event> {
        if self.pause != 0 {
            self.pause -= 1;
            return None;
        }
        if byte == 0xe1 {
            self.extended = false;
            self.pause = 5;
            return None;
        }
        if byte == 0xe0 {
            self.extended = true;
            return None;
        }
        let extended = core::mem::take(&mut self.extended);
        let key = match (extended, byte & 0x7f) {
            (true, 0x48) => wire::UP,
            (true, 0x50) => wire::DOWN,
            (true, 0x4b) => wire::LEFT,
            (true, 0x4d) => wire::RIGHT,
            (_, 0x1c) => wire::ENTER,
            (false, 0x3b) => wire::F1,
            (false, 0x3c) => wire::F2,
            _ => return None,
        };
        let mask = 1 << (key - 1);
        let state = if byte & 0x80 != 0 {
            if self.held & mask == 0 {
                return None;
            }
            self.held &= !mask;
            wire::RELEASE
        } else if self.held & mask != 0 {
            wire::REPEAT
        } else {
            self.held |= mask;
            wire::PRESS
        };
        Some(Event { key, state })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extended_press_repeat_release_and_reset() {
        let mut decoder = Decoder::default();
        for (byte, state) in [
            (0x4d, wire::PRESS),
            (0x4d, wire::REPEAT),
            (0xcd, wire::RELEASE),
        ] {
            assert_eq!(decoder.feed(0xe0), None);
            assert_eq!(
                decoder.feed(byte),
                Some(Event {
                    key: wire::RIGHT,
                    state
                })
            );
        }
        decoder.feed(0xe0);
        assert_eq!(decoder.feed(0xcd), None);
        decoder.feed(0xe0);
        assert_eq!(decoder.reset(), Event::reset());
        assert_eq!(decoder.feed(0x4d), None); // lost prefix cannot become a key
        assert_eq!(
            decoder.feed(0x1c),
            Some(Event {
                key: wire::ENTER,
                state: wire::PRESS
            })
        );
        decoder.reset();
        assert_eq!(decoder.feed(0x9c), None); // release from previous generation
    }
    #[test]
    fn unrelated_sequences_do_not_generate_navigation() {
        let mut decoder = Decoder::default();
        for byte in [
            0xe1, 0x1d, 0x45, 0xe1, 0x9d, 0xc5, // Pause
            0xe0, 0x2a, 0xe0, 0x37, 0xe0, 0xb7, 0xe0, 0xaa, // PrintScreen
            0xfa, 0xfe, 0x4b, 0xcb,
        ] {
            assert_eq!(decoder.feed(byte), None);
        }
        assert_eq!(
            decoder.feed(0x3b),
            Some(Event {
                key: wire::F1,
                state: wire::PRESS
            })
        );
    }
}
