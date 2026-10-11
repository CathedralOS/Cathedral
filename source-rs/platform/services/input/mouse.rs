//! Standard three-byte PS/2 packets. Only complete packets produce state updates.
#[derive(Default)]
pub struct Mouse {
    packet: [u8; 3],
    used: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Motion {
    pub dx: i16,
    pub dy: i16,
    pub buttons: u8,
    pub reset: bool,
}
impl Mouse {
    pub fn reset(&mut self) -> Motion {
        self.used = 0;
        Motion {
            dx: 0,
            dy: 0,
            buttons: 0,
            reset: true,
        }
    }
    pub fn feed(&mut self, byte: u8) -> Option<Motion> {
        if self.used == 0 && byte & 8 == 0 {
            return None;
        }
        self.packet[self.used] = byte;
        self.used += 1;
        if self.used < 3 {
            return None;
        }
        self.used = 0;
        let [header, x, y] = self.packet;
        if header & 0xc0 != 0 {
            return Some(self.reset());
        }
        let signed = |value: u8, negative: bool| i16::from(value) - if negative { 256 } else { 0 };
        Some(Motion {
            dx: signed(x, header & 0x10 != 0),
            dy: -signed(y, header & 0x20 != 0),
            buttons: header & 7,
            reset: false,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn packets_preserve_sign_buttons_and_reset_after_loss() {
        let mut mouse = Mouse::default();
        assert_eq!(mouse.feed(0), None);
        assert_eq!(mouse.feed(0x39), None);
        assert_eq!(mouse.feed(0xfe), None);
        assert_eq!(
            mouse.feed(0xfd),
            Some(Motion {
                dx: -2,
                dy: 3,
                buttons: 1,
                reset: false
            })
        );
        mouse.feed(8);
        mouse.reset();
        assert_eq!(mouse.feed(0xc8), None);
        assert_eq!(mouse.feed(0), None);
        assert!(mouse.feed(0).unwrap().reset);
        assert_eq!(mouse.feed(8), None);
        assert_eq!(mouse.feed(255), None);
        let motion = mouse.feed(255).unwrap();
        assert_eq!((motion.dx, motion.dy), (255, -255));
    }
}
