//! Root slot 0 alternates between the stored picker and boot-bundled recovery.
//! Only a real retry request retries admission. A dead picker shows recovery;
//! a dead recovery screen does not create an automatic restart loop.
use super::*;
// A 32-page card beside its retained predecessor while replacing its image.
const RECOVERY_PAGES: u64 = 64;
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Login {
    Stopped,
    Starting,
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
        self.screen(matrix, extent, true);
    }
    pub(crate) fn waiting(&mut self, matrix: Pair, extent: [u64; 2]) {
        self.screen(matrix, extent, false);
    }
    fn screen(&mut self, matrix: Pair, extent: [u64; 2], failed: bool) {
        use crate::recovery_protocol as protocol;
        let state = if failed {
            Self::Recovery
        } else {
            Self::Starting
        };
        let mode = if failed {
            protocol::FAILED
        } else {
            protocol::STARTING
        };
        // A failed lookup leaves the existing recovery screen and its input
        // state alive. A failed admission may already have retired it.
        if matches!(*self, Self::Recovery | Self::Starting) && health(matrix)[0] != abi::PEER_CLOSED
        {
            if *self != state {
                // Retire queued input/focus before enabling a different state.
                // A click during readiness must not become a later retry.
                request(matrix, [wire::REVOKE, 0, 0, 0, 0, 0, 0, 0]).unwrap();
                request(
                    matrix,
                    wire::route([protocol::STATUS, mode, 0, 0, 0, 0, 0, 0], 0).unwrap(),
                )
                .unwrap();
                select_user(matrix, extent).unwrap();
            }
            *self = state;
            return;
        }
        stop(matrix, 0).expect("login cleanup");
        request(matrix, [wire::RUN_PAGES, 0, RECOVERY_PAGES, 0, 0, 0, 0, 0]).unwrap();
        request(matrix, [wire::RUN, 0, 0, mode, 0, 0, 0, 0]).expect("bundled recovery admission");
        select_user(matrix, extent).expect("recovery placement");
        *self = state;
    }
    pub(crate) fn poll(&mut self, matrix: Pair, extent: [u64; 2]) -> bool {
        if matches!(*self, Self::Stopped | Self::Starting) {
            return false;
        }
        let reply = health(matrix);
        match *self {
            Self::Picker if reply[0] == abi::PEER_CLOSED => self.recover(matrix, extent, reply[0]),
            Self::Recovery if reply[0] == 0 && reply[1] == 1 => {
                cathedral_user_runtime::write(b"Cathedral desktop: sign-in retry requested\n")
                    .unwrap();
                return true;
            }
            _ => (),
        }
        false
    }
}
fn health(matrix: Pair) -> wire::Packet {
    call(
        matrix,
        wire::route([wire::HEALTH, 0, 0, 0, 0, 0, 0, 0], 0).unwrap(),
    )
}
