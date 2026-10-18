//! Root slot 0 alternates between the stored picker and boot-bundled recovery.
//! Only a real retry request retries admission. A dead picker shows recovery;
//! a dead recovery screen does not create an automatic restart loop.
use super::*;
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Login {
    Stopped,
    Picker,
    Recovery,
}
impl Login {
    pub(crate) fn start(&mut self, matrix: Pair, extent: [u64; 2], failed: bool) {
        #[cfg(feature = "login-recovery-probe")]
        {
            // A transient admission refusal on the first attempt only. Manual
            // retry must then traverse the real path lookup/admission sequence.
            use core::sync::atomic::{AtomicBool, Ordering};
            static FIRST: AtomicBool = AtomicBool::new(true);
            if FIRST.swap(false, Ordering::Relaxed) {
                self.recover(matrix, extent, abi::IO_ERROR);
                return;
            }
        }
        match chooser(self, matrix, extent, failed) {
            Ok(()) => *self = Self::Picker,
            Err(code) => self.recover(matrix, extent, code),
        }
    }
    pub(crate) fn recover(&mut self, matrix: Pair, extent: [u64; 2], code: u64) {
        time::storage_sample("login-start-refused", &[code]);
        // A failed lookup leaves the existing recovery screen and its input
        // state alive. A failed admission may already have retired it.
        if *self == Self::Recovery && health(matrix)[0] != abi::PEER_CLOSED {
            return;
        }
        stop(matrix, 0).expect("login cleanup");
        request(matrix, [wire::RUN_PAGES, 0, 32, 0, 0, 0, 0, 0]).unwrap();
        request(matrix, [wire::RUN, 0, 0, 0, 0, 0, 0, 0]).expect("bundled recovery admission");
        select_user(matrix, extent).expect("recovery placement");
        *self = Self::Recovery;
    }
    pub(crate) fn poll(&mut self, matrix: Pair, extent: [u64; 2]) {
        if *self == Self::Stopped {
            return;
        }
        let reply = health(matrix);
        match *self {
            Self::Picker if reply[0] == abi::PEER_CLOSED => self.recover(matrix, extent, reply[0]),
            Self::Recovery if reply[0] == 0 && reply[1] == 1 => {
                cathedral_user_runtime::write(b"Cathedral desktop: sign-in retry requested\n")
                    .unwrap();
                self.start(matrix, extent, false);
            }
            _ => (),
        }
    }
}
fn health(matrix: Pair) -> wire::Packet {
    call(
        matrix,
        wire::route([wire::HEALTH, 0, 0, 0, 0, 0, 0, 0], 0).unwrap(),
    )
}
