//! Disposable nested launch fixtures: real owner revocation/stop during a held read.
use super::*;
#[derive(Default)]
pub(super) struct Probe {
    observed: Option<u64>,
    done: bool,
}
impl Probe {
    pub fn apply(&mut self, matrix: Pair, height: u64) {
        if self.done {
            return;
        }
        let ordinary = cfg!(any(
            feature = "launch-read-revoke-probe",
            feature = "launch-read-stop-probe"
        ));
        let now = time::now().unwrap();
        let Some(observed) = self.observed else {
            let draft = call(matrix, [wire::READ_PIXEL, 21, 26, 0, 0, 0, 0, 0]);
            let bar = call(matrix, [wire::READ_PIXEL, 40, height - 152, 0, 0, 0, 0, 0]);
            if draft[..2] == [0, 0x9b7028] && (ordinary || bar[..2] == [0, 0x101827]) {
                self.observed = Some(now);
            }
            return;
        };
        // Submit in the last part of the three-second gate, inside ordinary
        // forwarded-control deadlines. The request is still queued before completion.
        if !time::reached(now, observed.wrapping_add(if ordinary { 275 } else { 225 })) {
            return;
        }
        let op = if cfg!(any(
            feature = "launcher-read-stop-probe",
            feature = "launch-read-stop-probe"
        )) {
            wire::STOP
        } else {
            wire::STORAGE_REVOKE
        };
        let mut words = [
            op,
            if ordinary && op == wire::STORAGE_REVOKE {
                2
            } else {
                0
            },
            0,
            0,
            0,
            0,
            0,
            0,
        ];
        for _ in 0..3 {
            words = wire::route(words, 1).unwrap();
        }
        cathedral_user_runtime::write(b"Cathedral desktop: read cancellation requested\n").unwrap();
        // The crashed task is already reclaimed. STOP cancels pending recovery
        // before reporting that its old peer is gone; revocation succeeds normally.
        let expected = if op == wire::STOP && !ordinary {
            wire::error(abi::PEER_CLOSED)
        } else {
            [0; 8]
        };
        assert_eq!(call(matrix, words), expected);
        cathedral_user_runtime::write(b"Cathedral desktop: read cancellation acknowledged\n")
            .unwrap();
        self.done = true;
    }
}
