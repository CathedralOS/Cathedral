//! Owner exit/fault/revocation must wake a parked receiver with the correct terminal status.
use crate::{Error, Handle, abi, probes};
pub(super) fn receiver(role: u64) -> u64 {
    let input = Handle::bootstrap(0).unwrap();
    let expected = if role == 2 {
        abi::PEER_CLOSED
    } else {
        abi::REVOKED
    };
    assert_eq!(input.receive(&mut [0; 64]), Err(Error(expected as i64)));
    0
}
pub(super) fn owner(role: u64) -> u64 {
    match role {
        3 => {
            settle_receiver();
            0
        }
        4 => {
            settle_receiver();
            probes::fault()
        }
        6 => {
            let output = Handle::bootstrap(0).unwrap();
            let revoke = Handle::bootstrap(1).unwrap();
            settle_receiver();
            revoke.revoke().unwrap();
            assert_eq!(output.send(b"revoked"), Err(Error(abi::REVOKED as i64)));
            0
        }
        _ => unreachable!(),
    }
}

// Let the peer reach its receive, then require boot's measured blocked count.
// A timer race that bypassed the wait would fail the fixture, not pass silently.
fn settle_receiver() {
    for _ in 0..32 {
        cathedral_user_runtime::yield_now();
    }
}
