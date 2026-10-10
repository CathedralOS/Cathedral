//! Scope, snapshot and retained redraw evidence through real IPC.
use super::*;
impl Session {
    pub(super) fn drawing_checks(&mut self) {
        assert_eq!(call(self.display, w::BACKGROUND, 0x101827)[0], 0);
        place(self.display, 0, 80, 160);
        place(self.display, 1, 400, 160);
        assert_eq!(call(self.a, 1, 1)[0], 0);
        assert_eq!(call(self.b, 1, 1)[0], 0);
        pixel(self.display, 80, 160, 0xe96f6f);
        pixel(self.display, 79, 160, 0x101827);
        pixel(self.display, 128, 216, 0x779bea);
        pixel(self.display, 192, 216, 0xe96f6f);
        pixel(self.display, 136, 224, 0x59d9cc);
        pixel(self.display, 400, 160, 0x79c99e);
        let stale = call(self.a, 1, 2)[1];
        assert_eq!(call(self.a, 3, 2)[0], abi::BAD_HANDLE);
        assert_eq!(call(self.a, 2, 3)[0], abi::INVALID_ARGUMENT);
        assert_eq!(call(self.a, 11, 0)[0], abi::DENIED);
        assert_eq!(call(self.display, w::STATUS, 0)[2], 2);
        assert_eq!(exchange(self.a, [4, 3, stale, 0, 0, 0])[0], abi::BAD_HANDLE);
        let foreign = call(self.b, 5, 0)[1];
        assert_eq!(exchange(self.a, [4, 3, foreign, 0, 0, 0])[0], abi::DENIED);
        assert_eq!(call(self.b, 6, 2)[0], 0); // Forged request must leave the true offer available.
        for revision in 3..19 {
            assert_eq!(call(self.a, 1, revision)[0], 0);
        }
        marker(b"Cathedral compositor: scopes nested clips atomic snapshots stale and foreign offers passed\n");
        place(self.display, 0, 350, 160);
        pixel(self.display, 80, 160, 0x101827);
        pixel(self.display, 350, 160, 0xe96f6f);
        pixel(self.display, 400, 160, 0x79c99e);
        place(self.display, 1, 700, 160);
        pixel(self.display, 400, 160, 0xe96f6f);
        place(self.display, 0, 80, 160);
        place(self.display, 1, 400, 160);
        marker(
            b"Cathedral compositor: movement overlap and exposure redrawn from retained content\n",
        );
    }
}
