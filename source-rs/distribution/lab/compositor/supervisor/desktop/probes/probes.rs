// Guest evidence over real capabilities and IPC, before interactive startup.
use super::*;
impl Desktop {
    pub(super) fn repaint_probe(&mut self) {
        let start = time::now().unwrap();
        for _ in 0..8 {
            assert_eq!(call(&mut self.base.display, w::BACKGROUND, 0x101827)[0], 0);
        }
        let ticks = time::now().unwrap() - start;
        let mut line =
            *b"Cathedral session: 8 full repaints ticks=000000 (10ms ticks, includes IPC)\n";
        let begin = b"Cathedral session: 8 full repaints ticks=".len();
        for i in 0..6 {
            line[begin + 5 - i] = b'0' + ((ticks / 10u64.pow(i as u32)) % 10) as u8;
        }
        marker(&line);
    }
    pub(super) fn boundaries(&mut self) {
        let display = &mut self.base.display;
        let host = exchange(display, [s::HOST_GRANT, 1, 2, 0, 0, 0])[1];
        assert_eq!(
            exchange(&mut self.base.a, [s::HOST_PLACE, host, 10, 10, 200, 160])[0],
            abi::DENIED
        );
        assert_eq!(
            exchange(
                &mut self.base.b,
                [s::HOST_PLACE, host, u64::MAX, 0, 200, 160]
            )[0],
            abi::INVALID_ARGUMENT
        );
        assert_eq!(
            exchange(&mut self.base.b, [s::HOST_PLACE, host, 100, 80, 200, 160])[0],
            0
        );
        pixel(display, 655, 359, 0x79c99e);
        pixel(display, 656, 359, 0x101827);
        let focus = exchange(display, [s::FOCUS_GRANT, 1, 0x10, 1, 0, 0])[1];
        assert_eq!(
            exchange(
                &mut self.base.b,
                [s::FOCUS_DELEGATE, focus, 2, s::KEY_MASK, 0, 0]
            )[0],
            abi::DENIED
        );
        let child = exchange(&mut self.base.b, [s::FOCUS_DELEGATE, focus, 2, 0x10, 0, 0]);
        assert_eq!(child[0], 0);
        assert_eq!(
            exchange(&mut self.c, [s::FOCUS_SELECT, child[1], 0, 0, 0, 0])[0],
            abi::DENIED
        );
        assert_eq!(
            exchange(&mut self.c, [s::KEY_SOURCE, 5, 1, 0, 0, 0])[0],
            abi::DENIED
        );
        let before = call(display, p::STATE, 0);
        assert_eq!(
            exchange(&mut self.c, [p::DEVICE_BATCH, 1, 20, 0, 0, 0])[0],
            abi::DENIED
        );
        assert_eq!(call(display, p::STATE, 0), before);
        assert_eq!(
            exchange(&mut self.base.b, [s::FOCUS_REVOKE, focus, 0, 0, 0, 0])[0],
            0
        );
        assert_eq!(
            exchange(&mut self.c, [s::FOCUS_SELECT, child[1], 2, 0, 0, 0])[0],
            abi::DENIED
        );
        assert_eq!(
            exchange(&mut self.base.b, [s::HOST_REVOKE, host, 0, 0, 0, 0])[0],
            0
        );
        assert_eq!(
            exchange(&mut self.base.b, [s::HOST_PLACE, host, 0, 0, 20, 20])[0],
            abi::DENIED
        );
        marker(b"Cathedral session: real child scope clipping wrong-owner stale hosting and attenuated focus denials passed\n");
    }
    pub(super) fn idle_probe(&mut self) {
        waits::check();
        // Drain configuration cancellations before recording a genuinely quiet interval.
        crate::transport::idle(50);
        let before = self.apps().map(|pair| health(pair).unwrap());
        let stats = call(&mut self.base.display, p::STATS, 0);
        crate::transport::idle(150);
        let after = self.apps().map(|pair| health(pair).unwrap());
        let end = call(&mut self.base.display, p::STATS, 0);
        for i in 0..3 {
            assert_eq!(before[i], after[i]);
        }
        assert_eq!(&stats[2..5], &end[2..5]); // No new event waits, deliveries or redraws.
        assert_eq!(end[1] - stats[1], 1); // Only the second statistics request.
        marker(b"Cathedral session: idle interval had zero app event wakes, empty polls, scene submissions or redraws\n");
    }
}
