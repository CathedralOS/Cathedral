//! Fail one participant while the other remains live, then replace the provider.
use super::*;
impl Session {
    pub(super) fn client_recovery(&mut self) {
        let right_identity = self.right.raw();
        for mode in [10, 7, 8] {
            if mode == 10 {
                self.a
                    .0
                    .send(&display::encode([mode, 0, 0, 0, 0, 0]))
                    .unwrap();
                assert!(matches!(
                    self.left.wait_until(time::after(300).unwrap()).unwrap(),
                    Outcome::Fault { vector: 6, .. }
                ));
            } else {
                assert_eq!(call(self.a, mode, 0)[0], 0);
                // A hostile peer spins or floods while this peer and control still progress.
                assert_eq!(call(self.b, 1, 10 + mode)[0], 0);
                self.left.cancel().unwrap();
                assert_eq!(self.left.wait().unwrap(), Outcome::Cancelled);
            }
            gone(self.display, 0);
            pixel(self.display, 80, 160, 0x101827);
            pixel(self.display, 400, 160, 0x79c99e);
            self.left = Launch::at(1).unwrap().spawn(mode).unwrap();
            self.a = connect(1);
            assert_eq!(call(self.a, 1, 1)[0], 0);
            assert_eq!(self.right.raw(), right_identity);
        }
        marker(
        b"Cathedral compositor: client fault spin flood and replacement preserved peer progress\n",
    );
    }
    pub(super) fn provider_recovery(&mut self) {
        self.provider.cancel().unwrap();
        assert_eq!(self.provider.wait().unwrap(), Outcome::Cancelled);
        self.provider = Launch::at(0).unwrap().spawn(1).unwrap();
        self.display = connect(0);
        assert_eq!(call(self.display, w::BACKGROUND, 0x101827)[0], 0);
        place(self.display, 0, 80, 160);
        place(self.display, 1, 400, 160);
        assert_eq!(call(self.a, 9, 0)[0], 0);
        assert_eq!(call(self.b, 9, 0)[0], 0);
        assert_eq!(call(self.a, 1, 1)[0], 0);
        assert_eq!(call(self.b, 1, 1)[0], 0);
        pixel(self.display, 136, 224, 0x59d9cc);
        pixel(self.display, 456, 224, 0x59d9cc);
        marker(b"Cathedral compositor: provider replacement reconnected both surviving clients\n");
    }
}
