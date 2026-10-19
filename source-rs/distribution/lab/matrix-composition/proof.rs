//! Test local-only views, source authority and replacement on actual processes.
use super::*;
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::input;
use cathedral_user_runtime::{
    ipc,
    task::{Child, Launch, Port},
};

fn start(index: u64) -> (Child, Pair) {
    let child = Launch::at(index).unwrap().spawn(0).unwrap();
    (child, Port::at(index).unwrap().connect().unwrap())
}
fn view(peer: Pair) -> wire::Packet {
    call(peer, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0])
}
fn next(peer: Pair, epoch: u64) -> wire::Packet {
    call(peer, [wire::NEXT, epoch, 0, 0, 0, 0, 0, 0])
}
pub fn run() -> u64 {
    let (_matrix, owner) = start(0);
    let mut peers = [start(1), start(2), start(3)];
    let views = peers.map(|(_, peer)| view(peer));
    for (i, (_, peer)) in peers.iter().enumerate() {
        assert_eq!(views[i][0], 0);
        assert_eq!(&views[i][2..4], &[300, 200]);
        assert_eq!(&views[i][5..], &[0; 3]); // No owner position or sibling count.
        for words in [
            [wire::PLACE, 0, 0, 0, 1, 1, 0, 0],
            [wire::REVOKE, 2, 0, 0, 0, 0, 0, 0],
            [wire::POINTER_SOURCE, 20, 450, 1, 0, 0, 0, 0],
            [wire::READ_PIXEL, 0, 0, 0, 0, 0, 0, 0],
            [wire::VIEW, 2, 0, 0, 0, 0, 0, 0],
        ] {
            assert_eq!(call(*peer, words), wire::error(abi::DENIED));
        }
        peer.0.send(&[0; 7]).unwrap();
        let mut bytes = [0; 64];
        let len = peer
            .1
            .receive_until(&mut bytes, time::after(300).unwrap())
            .unwrap();
        assert_eq!(wire::decode(&bytes[..len]), Some(wire::error(abi::DENIED)));
    }
    assert_eq!(
        call(owner, [wire::POINTER_SOURCE, 20, 450, 1, 0, 0, 0, 0]),
        [0; 8]
    );
    assert_eq!(
        &next(peers[2].1, views[2][1])[2..6],
        &[wire::POINTER, 20, 50, 1]
    );
    for i in 0..2 {
        assert_eq!(next(peers[i].1, views[i][1])[2], wire::NONE);
    }
    assert_eq!(
        call(
            owner,
            [
                wire::KEY_SOURCE,
                input::ENTER as u64,
                input::PRESS as u64,
                0,
                0,
                0,
                0,
                0
            ]
        ),
        [0; 8]
    );
    assert_eq!(next(peers[2].1, views[2][1])[2], wire::KEY);
    assert_eq!(call(owner, [wire::REVOKE, 1, 0, 0, 0, 0, 0, 0]), [0; 8]);
    assert_eq!(view(peers[1].1), wire::error(abi::DENIED));
    assert_eq!(view(peers[0].1), views[0]);
    assert_eq!(view(peers[2].1), views[2]);
    assert_eq!(
        call(owner, [wire::PLACE, 1, 0, 200, 300, 200, wire::KEY_MASK, 0]),
        [0; 8]
    );
    let old = view(peers[1].1)[1];
    peers[1].0.cancel().unwrap();
    peers[1].0.wait().unwrap();
    peers[1].1.0.close().unwrap();
    peers[1].1.1.close().unwrap();
    peers[1] = start(2);
    assert_eq!(view(peers[1].1), wire::error(abi::DENIED));
    assert_eq!(
        call(owner, [wire::PLACE, 1, 0, 200, 300, 200, wire::KEY_MASK, 0]),
        [0; 8]
    );
    assert_eq!(next(peers[1].1, old), wire::error(abi::DENIED));
    assert_eq!(view(peers[1].1)[2], 300);
    assert_eq!(view(peers[0].1), views[0]);
    assert_eq!(view(peers[2].1), views[2]);
    cathedral_user_runtime::write(b"Cathedral alternative matrix: three peers, local input, denied owner commands and replacement passed\n").unwrap();
    let (_, idle) = ipc::channel().unwrap();
    let _ = idle.receive(&mut [0; 64]);
    0
}
