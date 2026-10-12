//! Echo peers prove caller-local names, endpoint rights and message identity.
use crate::{Error, Handle, abi};

pub(super) fn server() -> u64 {
    let input = Handle::bootstrap(0).unwrap();
    let output = Handle::bootstrap(1).unwrap();
    let mut buffer = [0; 64];
    for _ in 0..32 {
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
    for round in 0..32 {
        let mut sent = [round; 64];
        sent[..8].copy_from_slice(&output.raw().to_le_bytes());
        output.send(&sent).unwrap();
        assert_eq!(input.receive(&mut received).unwrap(), 64);
        assert_eq!(sent, received);
    }
    output.close().unwrap();
    assert_eq!(output.send(b"stale"), Err(Error(abi::BAD_HANDLE as i64)));
    0
}
