//! Recursive loss checks must preserve dirty and slow descendants. An old clean
//! answer must never authorize a later close after the descendant becomes dirty.
use super::*;
impl Fixture {
    pub(super) fn loss(&mut self) {
        let probe = [wire::LOSS, 0, 0, 0, 0, 0, 0, 0];
        self.owner(0, probe);
        // A nominally clean reply with reserved data is malformed, not consent.
        for reserved in [1, u64::MAX] {
            self.owner(2, [crate::leaf::LOSS_PROBE, 0, 0, 0, reserved, 0, 0, 0]);
            assert_ne!(self.query(0, probe), [0; 8]);
            self.sibling_close(abi::BUSY);
            assert_eq!(self.view(2)[0], 0);
        }
        self.owner(2, [crate::leaf::LOSS_PROBE, 1, 0, 1, 0, 0, 0, 0]);
        assert_ne!(self.query(0, probe), [0; 8]);
        assert_eq!(self.view(2)[0], 0);
        assert_eq!(self.view(3)[0], 0);

        self.owner(2, [crate::leaf::LOSS_PROBE, 0, 150, 1, 0, 0, 0, 0]);
        assert_ne!(self.query(0, probe), [0; 8]);
        let until = time::after(200).unwrap();
        while !time::reached(time::now().unwrap(), until) {
            cathedral_user_runtime::yield_now();
        }
        // The delayed clean reply is queued now. All new observations are dirty.
        for _ in 0..3 {
            assert_ne!(self.query(0, probe), [0; 8]);
        }
        assert_eq!(self.view(2)[0], 0);
        self.sibling_close(abi::BUSY);
        assert_eq!(self.view(2)[0], 0);
        self.owner(2, [crate::leaf::LOSS_PROBE, 0, 0, 0, 0, 0, 0, 0]);
        self.owner(0, probe);
        self.sibling_close(0);
        assert_eq!(self.query(1, probe), wire::error(abi::PEER_CLOSED));
        assert_eq!(self.view(3)[0], 0);
        self.admit_inner();
        self.layout();
        self.owner(0, probe);
        // Exhaust the outside leaf's unanswered-request budget. It stays alive,
        // then releases eight old clean replies and reports dirty on new probes.
        self.owner(3, [crate::leaf::LOSS_BATCH_PROBE, 0, 0, 0, 0, 0, 0, 0]);
        for _ in 0..9 {
            assert_ne!(self.query(0, probe), [0; 8]);
        }
        let until = time::after(200).unwrap();
        while !time::reached(time::now().unwrap(), until) {
            cathedral_user_runtime::yield_now();
        }
        assert_ne!(self.query(0, probe), [0; 8]);
        assert_eq!(self.view(3)[0], 0);
        self.owner(3, [crate::leaf::LOSS_PROBE, 0, 0, 0, 0, 0, 0, 0]);
        self.owner(0, probe);
        cathedral_user_runtime::write(b"Cathedral matrix: recursive loss kept dirty and delayed descendants; clean subtree closed and reopened\n").unwrap();
    }

    fn sibling_close(&mut self, expected: u64) {
        self.owner(
            3,
            [crate::leaf::DETACHED_PROBE, wire::CLOSE, 0, 0, 0, 0, 0, 0],
        );
        let until = time::after(300).unwrap();
        loop {
            let reply = self.query(3, [crate::leaf::LAST_PROBE, 0, 0, 0, 0, 0, 0, 0]);
            if reply[0] != abi::WOULD_BLOCK {
                assert_eq!(reply[0], expected);
                return;
            }
            assert!(!time::reached(time::now().unwrap(), until));
        }
    }
}
