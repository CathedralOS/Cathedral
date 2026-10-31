//! Test local-only views, source authority and replacement on actual processes.
use super::*;
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::input;
use cathedral_user_runtime::{
    ipc,
    task::{Child, Launch, Port},
};

/// The owner's control link; a slot tag in the high word relays through that peer.
struct Lab {
    owner: Owner,
    matrix: Child,
}
impl Lab {
    fn call(&mut self, slot: Option<u64>, mut words: wire::Packet) -> wire::Packet {
        if let Some(slot) = slot {
            words[0] |= (slot + 1) << 32;
        }
        match self.owner.try_call(words) {
            Ok(reply) => reply,
            // A dead matrix explains itself through its exit status.
            Err(error) => panic!(
                "matrix call {words:?} failed: {error:?}; matrix outcome {:?}",
                self.matrix.wait_until(time::after(300).unwrap())
            ),
        }
    }
    fn owner(&mut self, words: wire::Packet) -> wire::Packet {
        self.call(None, words)
    }
    fn view(&mut self, slot: u64) -> wire::Packet {
        self.call(Some(slot), [wire::VIEW, 0, 0, 0, 0, 0, 0, 0])
    }
    fn next(&mut self, slot: u64, epoch: u64) -> wire::Packet {
        self.call(Some(slot), [wire::NEXT, epoch, 0, 0, 0, 0, 0, 0])
    }
}
fn start() -> (Child, Lab) {
    let loans = [
        Launch::at(1).unwrap(),
        Launch::at(2).unwrap(),
        Launch::at(3).unwrap(),
    ];
    let child = Launch::at(0).unwrap().spawn_tree(0, &[], &loans).unwrap();
    // Only the matrix owns the children's admission and control authority now.
    assert!(loans[0].spawn(0).is_err());
    assert!(Port::at(1).is_err());
    let owner = Owner::connect(Port::at(0).unwrap().connect().unwrap());
    (
        child,
        Lab {
            owner,
            matrix: child,
        },
    )
}
pub fn run() -> u64 {
    let (matrix, mut lab) = start();
    #[allow(unused_mut)]
    let mut views = [0, 1, 2].map(|slot| lab.view(slot));
    use cathedral_platform_contracts::{imports, services};
    for slot in 0..3 {
        for _ in 0..16 {
            assert_eq!(
                lab.call(Some(slot), services::request(imports::INTERACTION, 1, 1, 0)),
                wire::error(services::TAKEN)
            );
            assert_eq!(
                lab.call(Some(slot), services::request(imports::STORAGE, 1, 1, 0)),
                wire::error(if slot == 2 {
                    services::TAKEN
                } else {
                    services::UNAVAILABLE
                })
            );
            assert_eq!(
                lab.call(Some(slot), services::request(99, 1, 1, 0)),
                wire::error(services::UNAVAILABLE)
            );
        }
        assert_eq!(
            lab.call(Some(slot), services::request(imports::INTERACTION, 2, 2, 0)),
            wire::error(services::INCOMPATIBLE)
        );
        let mut forged = services::request(imports::INTERACTION, 1, 1, 0);
        forged[6] = 1; // A requested sibling identity cannot select another offer.
        assert_eq!(lab.call(Some(slot), forged), wire::error(services::INVALID));
    }
    for slot in 0..3 {
        let view = views[slot as usize];
        assert_eq!(view[0], 0);
        assert_eq!(&view[2..4], &[300, 200]);
        assert_eq!(&view[5..], &[0; 3]); // No owner position or sibling count.
        for words in [
            [wire::PLACE, 0, 0, 0, 1, 1, 0, 0],
            [wire::REVOKE, 2, 0, 0, 0, 0, 0, 0],
            [wire::POINTER_SOURCE, 20, 450, 1, 0, 0, 0, 0],
            [wire::READ_PIXEL, 0, 0, 0, 0, 0, 0, 0],
            [wire::VIEW, 2, 0, 0, 0, 0, 0, 0],
        ] {
            assert_eq!(lab.call(Some(slot), words), wire::error(abi::DENIED));
        }
        assert_eq!(
            lab.call(Some(slot), [u32::MAX as u64, 0, 0, 0, 0, 0, 0, 0]),
            wire::error(abi::DENIED)
        );
    }
    assert_eq!(
        lab.owner([wire::POINTER_SOURCE, 20, 450, 1, 0, 0, 0, 0]),
        [0; 8]
    );
    assert_eq!(&lab.next(2, views[2][1])[2..6], &[wire::POINTER, 20, 50, 1]);
    for slot in 0..2 {
        assert_eq!(lab.next(slot, views[slot as usize][1])[2], wire::NONE);
    }
    assert_eq!(
        lab.owner([
            wire::KEY_SOURCE,
            input::ENTER as u64,
            input::PRESS as u64,
            0,
            0,
            0,
            0,
            0
        ]),
        [0; 8]
    );
    assert_eq!(lab.next(2, views[2][1])[2], wire::KEY);
    assert_eq!(lab.owner([wire::REVOKE, 1, 0, 0, 0, 0, 0, 0]), [0; 8]);
    assert_eq!(lab.view(1), wire::error(abi::DENIED));
    assert_eq!(lab.view(0), views[0]);
    assert_eq!(lab.view(2), views[2]);
    assert_eq!(
        lab.owner([wire::PLACE, 1, 0, 200, 300, 200, wire::KEY_MASK, 0]),
        [0; 8]
    );
    let old = lab.view(1)[1];
    assert_eq!(lab.owner([wire::STOP, 1, 0, 0, 0, 0, 0, 0]), [0; 8]);
    assert_eq!(lab.owner([wire::SPAWN, 1, 0, 0, 0, 0, 0, 0]), [0; 8]);
    assert_eq!(lab.view(1), wire::error(abi::DENIED));
    assert_eq!(
        lab.owner([wire::PLACE, 1, 0, 200, 300, 200, wire::KEY_MASK, 0]),
        [0; 8]
    );
    assert_eq!(lab.next(1, old), wire::error(abi::DENIED));
    assert_eq!(lab.view(1)[2], 300);
    assert_eq!(lab.view(0), views[0]);
    assert_eq!(lab.view(2), views[2]);
    #[cfg(feature = "omega-matrix")]
    surfaces(&mut lab, &mut views);
    // Repeated replacement exercises retained-endpoint teardown and channel reuse.
    for _ in 0..16 {
        assert_eq!(lab.owner([wire::STOP, 1, 0, 0, 0, 0, 0, 0]), [0; 8]);
        assert_eq!(lab.owner([wire::SPAWN, 1, 0, 0, 0, 0, 0, 0]), [0; 8]);
        assert_eq!(lab.view(1), wire::error(abi::DENIED));
        assert_eq!(lab.view(0), views[0]);
        assert_eq!(lab.view(2), views[2]);
    }
    // Reclaim fully retained and partially transferred offers repeatedly. A VIEW
    // round trip proves each fixture reached its deliberately interrupted stage.
    for _ in 0..16 {
        for mode in [1, 2] {
            assert_eq!(lab.owner([wire::STOP, 2, 0, 0, 0, 0, 0, 0]), [0; 8]);
            assert_eq!(lab.owner([wire::SPAWN, 2, mode, 0, 0, 0, 0, 0]), [0; 8]);
            assert_eq!(lab.view(2), wire::error(abi::DENIED));
            assert_eq!(lab.view(0), views[0]);
        }
    }
    matrix.cancel().unwrap();
    matrix.wait().unwrap();
    lab.owner.close();
    // Entire subtree death returns all three loans; a new matrix can own them again.
    let (matrix, mut lab) = start();
    assert_eq!(lab.view(2)[2], 300);
    matrix.cancel().unwrap();
    matrix.wait().unwrap();
    lab.owner.close();
    cathedral_user_runtime::write(b"Cathedral alternative matrix: scoped acquisition, interrupted handoffs, rollback and subtree reclamation passed\n").unwrap();
    let (_, idle) = ipc::channel().unwrap();
    let _ = idle.receive(&mut [0; 64]);
    0
}

/// Surfaces on the Omega matrix: allowance ranges, child-local names, clipping
/// and translation, refusals, frame demand, visibility and retirement through
/// replacement. The proof-only owner words above 900 read the last presented
/// packet, count flushed erasures and drive the owner side of demand.
#[cfg(feature = "omega-matrix")]
fn surfaces(lab: &mut Lab, views: &mut [wire::Packet; 3]) {
    const LAST_PRESENTED: u64 = 900;
    const ERASED: u64 = 901;
    const REQUEST_FRAME: u64 = 902;
    const FRAME_READY: u64 = 903;
    const RELEASE_FRAME: u64 = 904;
    const VISIBLE: u64 = 905;
    fn poll(lab: &mut Lab) -> wire::Packet {
        lab.call(Some(2), [wire::POLL, 0, 0, 0, 0, 0, 0, 0])
    }
    for slot in 0..3 {
        assert_eq!(
            lab.owner([wire::SURFACE_LIMIT, slot, 4, 0, 0, 0, 0, 0]),
            [0; 8]
        );
    }
    // Twelve virtual surfaces are fully allotted; a fifth rank has no room.
    assert_eq!(
        lab.owner([wire::SURFACE_LIMIT, 0, 5, 0, 0, 0, 0, 0]),
        wire::error(abi::NO_MEMORY)
    );
    let epoch = views[2][1];
    let info = [wire::SURFACE_INFO, 0, 0, 0, 0, 0, 0, 0];
    assert_eq!(lab.call(Some(2), info), [0, 4, 0, 0, 0, 0, 0, 0]);
    let create = |rank| [wire::SURFACE_CREATE, epoch, rank, 0, 0, 0, 0, 0];
    assert_eq!(lab.call(Some(2), create(0)), [0, 1, 0, 0, 0, 0, 0, 0]);
    assert_eq!(lab.call(Some(2), create(0)), wire::error(abi::BUSY));
    assert_eq!(lab.call(Some(2), create(4)), wire::error(abi::DENIED));
    assert_eq!(
        lab.call(Some(2), [wire::SURFACE_CREATE, epoch + 1, 1, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    for rank in 1..4 {
        assert_eq!(
            lab.call(Some(2), create(rank)),
            [0, rank + 1, 0, 0, 0, 0, 0, 0]
        );
    }
    assert_eq!(lab.call(Some(2), create(0)), wire::error(abi::NO_MEMORY));
    assert_eq!(lab.call(Some(2), info), [0, 4, 4, 0, 0, 0, 0, 0]);
    // Paint near the corner: clipped to the view, then offset by the third region.
    assert_eq!(
        lab.call(Some(2), [wire::PAINT, epoch, 1, 290, 190, 50, 50, 0x123456]),
        [0; 8]
    );
    assert_eq!(
        lab.owner([LAST_PRESENTED, 0, 0, 0, 0, 0, 0, 0]),
        [wire::PAINT, 1, 9, 290, 590, 10, 10, 0x123456]
    );
    for bad in [
        [wire::PAINT, epoch, 1, 0, 0, 1, 1, 0x1000000],
        [wire::PAINT, epoch + 1, 1, 0, 0, 1, 1, 0],
    ] {
        assert_eq!(lab.call(Some(2), bad), wire::error(abi::DENIED));
    }
    assert_eq!(
        lab.call(Some(2), [wire::PAINT, epoch, 99, 0, 0, 1, 1, 0]),
        wire::error(abi::BAD_HANDLE)
    );
    // A sibling cannot name this surface.
    assert_eq!(
        lab.call(Some(0), [wire::PAINT, views[0][1], 1, 0, 0, 1, 1, 0]),
        wire::error(abi::BAD_HANDLE)
    );
    assert_eq!(
        lab.call(Some(2), [wire::ERASE, epoch, 2, 0, 0, 0, 0, 0]),
        [0; 8]
    );
    assert_eq!(
        lab.owner([LAST_PRESENTED, 0, 0, 0, 0, 0, 0, 0]),
        [wire::ERASE, 1, 10, 0, 0, 0, 0, 0]
    );
    assert_eq!(lab.call(Some(2), info), [0, 4, 3, 0, 0, 0, 0, 0]);
    // Frame demand: the owner's request shows in the poll until the source
    // acknowledges that ticket; a stale ticket is refused on both sides.
    assert_eq!(poll(lab)[4] & (wire::FRAME_REQUESTED | wire::NO_FRAMES), 0);
    assert_eq!(poll(lab)[4] >> wire::SURFACE_BITS, 4);
    assert_eq!(
        lab.owner([REQUEST_FRAME, 2, 0, 0, 0, 0, 0, 0]),
        [0, 1, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(
        lab.owner([REQUEST_FRAME, 2, 0, 0, 0, 0, 0, 0]),
        wire::error(abi::BUSY)
    );
    assert_ne!(poll(lab)[4] & wire::FRAME_REQUESTED, 0);
    assert_eq!(
        lab.call(Some(2), [wire::FRAME_DEMAND, epoch, 0, 0, 0, 0, 0, 0]),
        [0, epoch, 1, 0, 0, 0, 0, 0]
    );
    assert_eq!(
        lab.call(Some(2), [wire::FRAME_READY, epoch, 7, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    assert_eq!(lab.owner([FRAME_READY, 2, 1, 0, 0, 0, 0, 0]), [0; 8]);
    assert_eq!(
        lab.call(Some(2), [wire::FRAME_READY, epoch, 1, 0, 0, 0, 0, 0]),
        [0; 8]
    );
    assert_eq!(
        lab.owner([FRAME_READY, 2, 1, 0, 0, 0, 0, 0]),
        [0, 1, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(poll(lab)[4] & wire::FRAME_REQUESTED, 0);
    assert_eq!(
        lab.owner([RELEASE_FRAME, 2, 7, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    assert_eq!(lab.owner([RELEASE_FRAME, 2, 1, 0, 0, 0, 0, 0]), [0; 8]);
    assert_eq!(
        lab.call(Some(2), [wire::FRAME_DEMAND, epoch, 0, 0, 0, 0, 0, 0]),
        [0, epoch, 0, 0, 0, 0, 0, 0]
    );
    // Hidden: no demand and no visible output, with the same view and names.
    assert_eq!(lab.owner([VISIBLE, 2, 0, 0, 0, 0, 0, 0]), [0; 8]);
    assert_ne!(poll(lab)[4] & wire::NO_FRAMES, 0);
    assert_eq!(wire::poll_view(poll(lab)), views[2]);
    assert_eq!(
        lab.call(Some(2), [wire::PAINT, epoch, 1, 1, 2, 3, 4, 0xabcdef]),
        [0; 8]
    );
    assert_eq!(
        lab.owner([LAST_PRESENTED, 0, 0, 0, 0, 0, 0, 0]),
        [wire::PAINT, 1, 9, 0, 0, 0, 0, 0xabcdef]
    );
    assert_eq!(lab.owner([VISIBLE, 2, 1, 0, 0, 0, 0, 0]), [0; 8]);
    assert_eq!(poll(lab)[4] & wire::NO_FRAMES, 0);
    // Replacement retires every surface and the allowance; the owner reissues it.
    assert_eq!(lab.owner([ERASED, 0, 0, 0, 0, 0, 0, 0]), [0; 8]);
    assert_eq!(lab.owner([wire::STOP, 2, 0, 0, 0, 0, 0, 0]), [0; 8]);
    assert_eq!(lab.owner([wire::SPAWN, 2, 0, 0, 0, 0, 0, 0]), [0; 8]);
    assert_eq!(
        lab.owner([ERASED, 0, 0, 0, 0, 0, 0, 0]),
        [0, 3, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(lab.call(Some(2), info), wire::error(abi::DENIED));
    assert_eq!(
        lab.owner([wire::PLACE, 2, 0, 400, 300, 200, wire::KEY_MASK, 0]),
        [0; 8]
    );
    assert_eq!(lab.call(Some(2), info), [0; 8]);
    assert_eq!(
        lab.owner([wire::SURFACE_LIMIT, 2, 4, 0, 0, 0, 0, 0]),
        [0; 8]
    );
    assert_eq!(lab.call(Some(2), info), [0, 4, 0, 0, 0, 0, 0, 0]);
    views[2] = lab.view(2);
    assert_eq!(&views[2][2..4], &[300, 200]);
}
