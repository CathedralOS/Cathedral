#![no_std]
#![no_main]
//! One fixture executable, separately admitted as client/service/fault-test roles.
mod deadlines;
mod probes;
mod readiness;
use cathedral_contracts::user as abi;
use cathedral_user_runtime::{Error, ipc::Handle};
cathedral_user_runtime::entry!(main);

fn main(role: u64, stale: u64) -> u64 {
    match role {
        0 => server(),
        1 => client(stale),
        2 | 5 => {
            let input = Handle::bootstrap(0).unwrap();
            let expected = if role == 2 {
                abi::PEER_CLOSED
            } else {
                abi::REVOKED
            };
            assert_eq!(input.receive(&mut [0; 64]), Err(Error(expected as i64)));
            0
        }
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
        7 => probes::sender(),
        8 => probes::receiver(),
        9 => deadlines::receiver(stale),
        10 => deadlines::sender(stale),
        11 => deadlines::denied(),
        12 => readiness::receiver(),
        13 => readiness::sender(),
        _ => 254,
    }
}

fn server() -> u64 {
    let input = Handle::bootstrap(0).unwrap();
    let output = Handle::bootstrap(1).unwrap();
    let mut buffer = [0; 64];
    for _ in 0..32 {
        assert_eq!(input.receive(&mut buffer).unwrap(), 64);
        // Receiving another task's handle bits does not confer its authority.
        let foreign = Handle::from_raw(u64::from_le_bytes(buffer[..8].try_into().unwrap()));
        assert_eq!(foreign.send(b"stolen"), Err(Error(abi::BAD_HANDLE as i64)));
        output.send(&buffer).unwrap();
    }
    assert_eq!(
        input.receive(&mut buffer),
        Err(Error(abi::PEER_CLOSED as i64))
    );
    0
}
fn client(stale: u64) -> u64 {
    let output = Handle::bootstrap(0).unwrap();
    let input = Handle::bootstrap(1).unwrap();
    assert_eq!(
        Handle::from_raw(stale).send(b"stale"),
        Err(Error(abi::BAD_HANDLE as i64))
    );
    assert!(Handle::bootstrap(2).is_err());
    assert_eq!(input.send(b"wrong right"), Err(Error(abi::DENIED as i64)));
    assert_eq!(output.revoke(), Err(Error(abi::DENIED as i64)));
    let mut received = [0; 64];
    for round in 0..32 {
        let mut sent = [round; 64];
        sent[..8].copy_from_slice(&output.raw().to_le_bytes());
        output.send(&sent).unwrap();
        assert_eq!(input.receive(&mut received).unwrap(), 64);
        assert_eq!(sent, received);
    }
    output.raw() // Boot feeds these retired-session bits to the next client.
}

// Let the peer reach its receive, then require boot's measured blocked count.
// A timer race that bypassed the wait would fail the fixture, not pass silently.
fn settle_receiver() {
    for _ in 0..32 {
        cathedral_user_runtime::yield_now();
    }
}
