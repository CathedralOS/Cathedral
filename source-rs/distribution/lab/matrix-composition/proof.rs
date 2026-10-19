//! Test local-only views, source authority and replacement on actual processes.
use super::*;
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::input;
use cathedral_user_runtime::{
    ipc,
    task::{Child, Launch, Port},
};

#[derive(Clone, Copy)]
struct Peer {
    pair: Pair,
    slot: Option<u64>,
}
fn call(peer: Peer, mut words: wire::Packet) -> wire::Packet {
    if let Some(slot) = peer.slot {
        words[0] |= (slot + 1) << 32;
    }
    super::call(peer.pair, words)
}
fn start() -> (Child, Peer) {
    let loans = [
        Launch::at(1).unwrap(),
        Launch::at(2).unwrap(),
        Launch::at(3).unwrap(),
    ];
    let child = Launch::at(0).unwrap().spawn_tree(0, &[], &loans).unwrap();
    // Only the matrix owns the children's admission and control authority now.
    assert!(loans[0].spawn(0).is_err());
    assert!(Port::at(1).is_err());
    (
        child,
        Peer {
            pair: Port::at(0).unwrap().connect().unwrap(),
            slot: None,
        },
    )
}
fn view(peer: Peer) -> wire::Packet {
    call(peer, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0])
}
fn next(peer: Peer, epoch: u64) -> wire::Packet {
    call(peer, [wire::NEXT, epoch, 0, 0, 0, 0, 0, 0])
}
pub fn run() -> u64 {
    let (matrix, owner) = start();
    let peers = core::array::from_fn::<_, 3, _>(|i| Peer {
        pair: owner.pair,
        slot: Some(i as u64),
    });
    let views = peers.map(view);
    for (i, peer) in peers.iter().enumerate() {
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
        assert_eq!(
            call(*peer, [u32::MAX as u64, 0, 0, 0, 0, 0, 0, 0]),
            wire::error(abi::DENIED)
        );
    }
    assert_eq!(
        call(owner, [wire::POINTER_SOURCE, 20, 450, 1, 0, 0, 0, 0]),
        [0; 8]
    );
    assert_eq!(
        &next(peers[2], views[2][1])[2..6],
        &[wire::POINTER, 20, 50, 1]
    );
    for i in 0..2 {
        assert_eq!(next(peers[i], views[i][1])[2], wire::NONE);
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
    assert_eq!(next(peers[2], views[2][1])[2], wire::KEY);
    assert_eq!(call(owner, [wire::REVOKE, 1, 0, 0, 0, 0, 0, 0]), [0; 8]);
    assert_eq!(view(peers[1]), wire::error(abi::DENIED));
    assert_eq!(view(peers[0]), views[0]);
    assert_eq!(view(peers[2]), views[2]);
    assert_eq!(
        call(owner, [wire::PLACE, 1, 0, 200, 300, 200, wire::KEY_MASK, 0]),
        [0; 8]
    );
    let old = view(peers[1])[1];
    assert_eq!(call(owner, [wire::STOP, 1, 0, 0, 0, 0, 0, 0]), [0; 8]);
    assert_eq!(call(owner, [wire::SPAWN, 1, 0, 0, 0, 0, 0, 0]), [0; 8]);
    assert_eq!(view(peers[1]), wire::error(abi::DENIED));
    assert_eq!(
        call(owner, [wire::PLACE, 1, 0, 200, 300, 200, wire::KEY_MASK, 0]),
        [0; 8]
    );
    assert_eq!(next(peers[1], old), wire::error(abi::DENIED));
    assert_eq!(view(peers[1])[2], 300);
    assert_eq!(view(peers[0]), views[0]);
    assert_eq!(view(peers[2]), views[2]);
    // Repeated replacement exercises retained-endpoint teardown and channel reuse.
    for _ in 0..16 {
        assert_eq!(call(owner, [wire::STOP, 1, 0, 0, 0, 0, 0, 0]), [0; 8]);
        assert_eq!(call(owner, [wire::SPAWN, 1, 0, 0, 0, 0, 0, 0]), [0; 8]);
        assert_eq!(view(peers[1]), wire::error(abi::DENIED));
        assert_eq!(view(peers[0]), views[0]);
        assert_eq!(view(peers[2]), views[2]);
    }
    matrix.cancel().unwrap();
    matrix.wait().unwrap();
    owner.pair.0.close().unwrap();
    owner.pair.1.close().unwrap();
    // Entire subtree death returns all three loans; a new matrix can own them again.
    let (matrix, owner) = start();
    assert_eq!(
        view(Peer {
            pair: owner.pair,
            slot: Some(2)
        })[2],
        300
    );
    matrix.cancel().unwrap();
    matrix.wait().unwrap();
    owner.pair.0.close().unwrap();
    owner.pair.1.close().unwrap();
    cathedral_user_runtime::write(b"Cathedral alternative matrix: owned admission, rollback, local input and subtree reclamation passed\n").unwrap();
    let (_, idle) = ipc::channel().unwrap();
    let _ = idle.receive(&mut [0; 64]);
    0
}
