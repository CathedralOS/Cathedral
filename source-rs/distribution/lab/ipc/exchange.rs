//! Echo peers prove caller-local names, endpoint rights and message identity.
use crate::{Error, Handle, abi};
use cathedral_kernel_contracts::lab::{ECHO_BURSTS, ECHO_ROUNDS};

pub(super) fn server() -> u64 {
    let input = Handle::bootstrap(0).unwrap();
    let output = Handle::bootstrap(1).unwrap();
    let mut buffer = [0; 64];
    for _ in 0..ECHO_ROUNDS * ECHO_BURSTS {
        assert_eq!(input.receive(&mut buffer).unwrap(), 64);
        // The same local number names our receive right, never the sender's right.
        let foreign = Handle::from_raw(u64::from_le_bytes(buffer[..8].try_into().unwrap()));
        assert_eq!(foreign.send(b"stolen"), Err(Error(abi::DENIED as i64)));
        output.send(&buffer).unwrap();
    }
    assert_eq!(
        input.receive(&mut buffer),
        Err(Error(abi::PEER_CLOSED as i64))
    );
    0
}
pub(super) fn client() -> u64 {
    let output = Handle::bootstrap(0).unwrap();
    let input = Handle::bootstrap(1).unwrap();
    assert_eq!(
        Handle::from_raw(0).send(b"invalid"),
        Err(Error(abi::BAD_HANDLE as i64))
    );
    assert!(Handle::bootstrap(2).is_err());
    assert_eq!(input.send(b"wrong right"), Err(Error(abi::DENIED as i64)));
    assert_eq!(output.revoke(), Err(Error(abi::DENIED as i64)));
    let mut received = [0; 64];
    let clock = || {
        cathedral_user_runtime::time::stamp()
            .map(|stamp| stamp[1])
            .ok()
    };
    let started = clock();
    let mut burst_started = started;
    let mut fastest = u64::MAX;
    for burst in 0..ECHO_BURSTS {
        for round in 0..ECHO_ROUNDS {
            let mut sent = [(burst * ECHO_ROUNDS + round) as u8; 64];
            sent[..8].copy_from_slice(&output.raw().to_le_bytes());
            output.send(&sent).unwrap();
            assert_eq!(input.receive(&mut received).unwrap(), 64);
            assert_eq!(sent, received);
        }
        let finished = clock();
        if let (Some(began), Some(finished)) = (burst_started, finished) {
            fastest = fastest.min(finished.saturating_sub(began));
        }
        burst_started = finished;
    }
    // Bursts of round trips between two tasks, each a send, the peer's receive
    // and send, and this receive: the kernel switch cost that every forwarding
    // hop pays, measured without a matrix in the way. The fastest burst is that
    // cost with no timer tick or host preemption inside it; the whole exchange
    // shows what landed in the rest. Only when this role was granted a clock.
    if let (Some(started), Some(finished)) = (started, burst_started) {
        cathedral_user_runtime::time::storage_sample(
            "ipc-roundtrip",
            &[
                ECHO_ROUNDS as u64,
                fastest,
                finished.saturating_sub(started),
            ],
        );
    }
    output.close().unwrap();
    assert_eq!(output.send(b"stale"), Err(Error(abi::BAD_HANDLE as i64)));
    0
}
