//! Guest evidence over real capabilities and IPC, before interactive startup.
use super::*;
mod waits;
impl Desktop {
    pub(super) fn boundaries(&mut self) {
        let display = self.base.display;
        let host = exchange(display, [s::HOST_GRANT, 1, 2, 0, 0, 0])[1];
        assert_eq!(
            exchange(self.base.a, [s::HOST_PLACE, host, 10, 10, 200, 160])[0],
            abi::DENIED
        );
        assert_eq!(
            exchange(self.base.b, [s::HOST_PLACE, host, u64::MAX, 0, 200, 160])[0],
            abi::INVALID_ARGUMENT
        );
        assert_eq!(
            exchange(self.base.b, [s::HOST_PLACE, host, 100, 80, 200, 160])[0],
            0
        );
        pixel(display, 655, 359, 0x79c99e);
        pixel(display, 656, 359, 0x101827);
        let focus = exchange(display, [s::FOCUS_GRANT, 1, 0x10, 1, 0, 0])[1];
        assert_eq!(
            exchange(
                self.base.b,
                [s::FOCUS_DELEGATE, focus, 2, s::KEY_MASK, 0, 0]
            )[0],
            abi::DENIED
        );
        let child = exchange(self.base.b, [s::FOCUS_DELEGATE, focus, 2, 0x10, 0, 0]);
        assert_eq!(child[0], 0);
        assert_eq!(
            exchange(self.c, [s::FOCUS_SELECT, child[1], 0, 0, 0, 0])[0],
            abi::DENIED
        );
        assert_eq!(
            exchange(self.c, [s::KEY_SOURCE, 5, 1, 0, 0, 0])[0],
            abi::DENIED
        );
        assert_eq!(
            exchange(self.base.b, [s::FOCUS_REVOKE, focus, 0, 0, 0, 0])[0],
            0
        );
        assert_eq!(
            exchange(self.c, [s::FOCUS_SELECT, child[1], 2, 0, 0, 0])[0],
            abi::DENIED
        );
        assert_eq!(
            exchange(self.base.b, [s::HOST_REVOKE, host, 0, 0, 0, 0])[0],
            0
        );
        assert_eq!(
            exchange(self.base.b, [s::HOST_PLACE, host, 0, 0, 20, 20])[0],
            abi::DENIED
        );
        marker(b"Cathedral session: real child scope clipping wrong-owner stale hosting and attenuated focus denials passed\n");
    }
    pub(super) fn idle_probe(&self) {
        waits::check(self.base.display, self.base.a);
        // Drain configuration cancellations before recording a genuinely quiet interval.
        let _ = self
            .base
            .display
            .1
            .receive_until(&mut [0; 64], time::after(50).unwrap());
        let before = self.apps().map(|pair| health(pair).unwrap());
        let stats = call(self.base.display, p::STATS, 0);
        let _ = self
            .base
            .display
            .1
            .receive_until(&mut [0; 64], time::after(150).unwrap());
        let after = self.apps().map(|pair| health(pair).unwrap());
        let end = call(self.base.display, p::STATS, 0);
        for i in 0..3 {
            assert_eq!(before[i], after[i]);
        }
        assert_eq!(&stats[2..5], &end[2..5]); // No new event waits, deliveries or redraws.
        assert_eq!(end[1] - stats[1], 1); // Only the second statistics request.
        marker(b"Cathedral session: idle interval had zero app event wakes, empty polls, scene submissions or redraws\n");
    }
}
