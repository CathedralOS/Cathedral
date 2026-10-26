//! Distribution-owned instructions and visible focus/recovery feedback.
use super::*;
impl Desktop {
    pub(super) fn chrome(&mut self) {
        for (x, y, text) in [
            (80, 40, &b"CATHEDRAL LAB"[..]),
            (80, 76, b"CLICK A BUTTON"),
            (400, 76, b"ARROWS / ENTER"),
            (700, 160, b"APP ONE"),
            (700, 190, b"INDEPENDENT APP"),
            (700, 240, b"HOST AND CHILD"),
            (700, 270, b"SEPARATE APPS"),
            (700, 300, b"CHILD: ENTER"),
            (80, 480, b"F4 RESET DISPLAY"),
            (80, 510, b"F5 RESET INPUT"),
            (80, 540, b"F6 RESET HOST"),
            (80, 570, b"F7 SYNTH CLICK"),
            (400, 480, b"ESC CANCEL INPUT"),
            (400, 510, b"DRAG IN THE BOX"),
            (400, 540, b"WHITE: MOUSE"),
            (400, 570, b"GOLD: SYNTHETIC"),
        ] {
            self.label(x, y, text);
        }
        self.last_focus = u64::MAX;
        self.update_focus();
        self.notice(b"READY");
        if self.pressure.is_some() {
            self.label(80, 620, b"PRESSURE ACTIVE");
        }
    }
    pub(super) fn update_focus(&mut self) {
        let focus = call(&mut self.base.display, s::FOCUS_STATE, 0)[1];
        if focus == self.last_focus {
            return;
        }
        self.last_focus = focus;
        self.label(
            400,
            420,
            match focus {
                1 => b"KEYS: APP ONE",
                2 => b"KEYS: HOST",
                3 => b"KEYS: CHILD",
                _ => b"KEYS: NONE",
            },
        );
    }
    pub(super) fn notice(&mut self, text: &[u8]) {
        self.label(80, 420, text);
    }
    fn label(&mut self, x: u64, y: u64, text: &[u8]) {
        let packet = display::encode_text(x, y, 2, 0xc5d3e8, text).unwrap();
        let (reply, len) = self
            .base
            .display
            .call(&packet, time::after(300).unwrap())
            .unwrap();
        assert_eq!(display::decode(&reply[..len]).unwrap()[0], 0);
    }
}
