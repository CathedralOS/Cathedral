//! Repeated creator death cannot renew a pinned account; sibling capacity stays usable.
mod worker;
use super::*;
use cathedral_user_runtime::task::{Child, Launch, Outcome, Port};
pub use worker::run as worker;
type Pair = (Handle, Handle);
fn spawn(index: u64, mode: u64) -> (Child, Pair) {
    let child = Launch::at(index).unwrap().spawn(mode).unwrap();
    (child, Port::at(index).unwrap().connect().unwrap())
}
fn charged(pair: Pair, expected: u64) {
    let (bytes, attachment) = receive(pair.1);
    assert!(attachment.is_none());
    let words: [u64; 4] =
        core::array::from_fn(|i| u64::from_le_bytes(bytes[i * 8..i * 8 + 8].try_into().unwrap()));
    assert_eq!(words, [8, expected, 16, 0]);
}
fn release(handle: Handle) {
    let mut bytes = [0; 64];
    assert_eq!(handle.receive(&mut bytes).unwrap(), 8);
    assert_eq!(&bytes[..8], b"survives");
    assert_eq!(
        handle.receive(&mut bytes),
        Err(Error(abi::PEER_CLOSED as i64))
    );
    handle.close().unwrap();
}
pub fn run() {
    let baseline = ipc::budget().unwrap();
    assert_eq!(
        (baseline.charged_channels, baseline.charged_handles),
        (0, 0)
    );
    for _ in 0..2 {
        let mut held = [None; 8];
        for (used, retained) in held.iter_mut().enumerate() {
            let (child, pair) = spawn(0, 7);
            charged(pair, used as u64);
            pair.0.send(b"create").unwrap();
            let (_, attachment) = receive(pair.1);
            *retained = Some(attachment.unwrap());
            assert_eq!(child.wait().unwrap(), Outcome::Returned(71));
        }
        let (exhausted, pair) = spawn(0, 7);
        charged(pair, 8);
        pair.0.send(b"create").unwrap();
        assert_eq!(&receive(pair.1).0[..9], b"exhausted");
        let own = ipc::budget().unwrap();
        assert_eq!((own.charged_channels, own.charged_handles), (0, 8));
        let (sibling, sibling_pair) = spawn(4, 8);
        assert_eq!(&receive(sibling_pair.1).0[..8], b"reserved");
        assert_eq!(ipc::budget().unwrap(), own); // No global pool or sibling use in the query.
        release(held[0].take().unwrap());
        pair.0.send(b"retry").unwrap();
        assert_eq!(&receive(pair.1).0[..9], b"recovered");
        assert_eq!(exhausted.wait().unwrap(), Outcome::Returned(72));
        sibling.cancel().unwrap();
        assert_eq!(sibling.wait().unwrap(), Outcome::Cancelled);
        for handle in held.into_iter().flatten() {
            release(handle);
        }
        assert_eq!(ipc::budget().unwrap(), baseline);
    }
    cathedral_user_runtime::write(b"Cathedral IPC: persistent sponsor charges, restart exhaustion, sibling reservations and final-close recovery passed\n").unwrap();
}
