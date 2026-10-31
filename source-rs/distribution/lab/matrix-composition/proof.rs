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
    #[cfg(feature = "omega-matrix")]
    input_model(&mut lab);
    #[cfg(feature = "omega-matrix")]
    delegation(&mut lab, &mut views);
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

/// The input model on the Omega matrix: owner-registered groups and channels,
/// keyboard selection, the directory and descriptions a recipient may observe,
/// retirement that waits for every recipient, bounded producer grants, capacity
/// limits and cursor preferences. The proof-only owner word 906 resolves the
/// cursor a local stream currently presents.
#[cfg(feature = "omega-matrix")]
fn input_model(lab: &mut Lab) {
    use cathedral_platform_contracts::input::description::{self, Description};
    const CURSOR: u64 = 906;
    fn created(packet: wire::Packet) -> u64 {
        assert_eq!(packet[0], 0);
        packet[1]
    }
    let views = [0, 1, 2].map(|slot| lab.view(slot));
    let epoch = |slot: usize| views[slot][1];
    // Owner-registered names are local to this world and never reused.
    let group = created(lab.owner([wire::INPUT_GROUP_CREATE, 0, 0, 0, 0, 0, 0, 0]));
    let channel = created(lab.owner([wire::INPUT_CHANNEL_CREATE, group, 0, 0, 0, 0, 0, 0]));
    assert_eq!(
        lab.owner([wire::INPUT_CHANNEL_CREATE, 99, 0, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    assert_eq!(
        lab.owner([wire::INPUT_POINTER, 99, 20, 450, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    assert_eq!(
        lab.owner([wire::INPUT_POINTER, channel, 20, 450, 1, 0, 0, 0]),
        [0; 8]
    );
    assert_eq!(
        &lab.next(2, epoch(2))[2..],
        &[wire::POINTER, 20, 50, 1, 1, 1]
    );
    assert_eq!(
        lab.owner([wire::INPUT_POINTER, channel, 20, 450, 0, 0, 0, 0]),
        [0; 8]
    );
    assert_eq!(&lab.next(2, epoch(2))[2..6], &[wire::POINTER, 20, 50, 0]);
    // Keys follow the group's selection; the press selected the third region.
    let key = |state: u64| {
        [
            wire::INPUT_KEY,
            channel,
            input::ENTER as u64,
            state,
            0,
            0,
            0,
            0,
        ]
    };
    for state in [input::PRESS, input::RELEASE] {
        assert_eq!(lab.owner(key(state as u64)), [0; 8]);
        assert_eq!(
            &lab.next(2, epoch(2))[2..5],
            &[wire::KEY, input::ENTER as u64, state as u64]
        );
    }
    // INPUT_SELECT moves the selection to the first region without a pointer;
    // the old target sees its queued keys cancelled.
    assert_eq!(
        lab.owner([wire::INPUT_SELECT, group, 4, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    assert_eq!(
        lab.owner([wire::INPUT_SELECT, 99, 1, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    assert_eq!(
        lab.owner([wire::INPUT_SELECT, group, 1, 0, 0, 0, 0, 0]),
        [0; 8]
    );
    assert_eq!(lab.next(2, epoch(2))[2], wire::CANCEL_KEYS);
    for state in [input::PRESS, input::RELEASE] {
        assert_eq!(lab.owner(key(state as u64)), [0; 8]);
        assert_eq!(
            &lab.next(0, epoch(0))[2..],
            &[wire::KEY, input::ENTER as u64, state as u64, 0, 1, 1]
        );
    }
    assert_eq!(lab.next(2, epoch(2))[2], wire::NONE);
    // The directory shows each recipient only its own names, one revision per membership.
    let directory = |lab: &mut Lab, slot: u64, revision: u64, after: u64| {
        lab.call(
            Some(slot),
            [
                wire::INPUT_DIRECTORY,
                views[slot as usize][1],
                revision,
                after,
                0,
                0,
                0,
                0,
            ],
        )
    };
    let row = directory(lab, 0, 0, 0);
    assert_eq!(&row[3..], &[1, 1, wire::CHANNEL_LIVE, wire::KEY_MASK, 0]);
    let revision = row[2];
    assert_eq!(
        directory(lab, 0, revision, 1),
        [0, epoch(0), revision, 0, 0, 0, 0, 0]
    );
    assert_eq!(directory(lab, 0, revision + 1, 0), wire::error(abi::BUSY));
    assert_eq!(directory(lab, 0, 0, 1), wire::error(abi::DENIED));
    assert_eq!(
        lab.call(
            Some(0),
            [wire::INPUT_DIRECTORY, epoch(0) + 1, 0, 0, 0, 0, 0, 0]
        ),
        wire::error(abi::DENIED)
    );
    assert_eq!(directory(lab, 1, 0, 0), [0, epoch(1), 1, 0, 0, 0, 0, 0]);
    // Retirement waits for every recipient; the group cannot go before its channel.
    assert_eq!(
        lab.owner([wire::INPUT_CHANNEL_RETIRE, channel, 0, 0, 0, 0, 0, 0]),
        [0; 8]
    );
    assert_eq!(
        lab.owner(key(input::RELEASE as u64)),
        wire::error(abi::DENIED)
    );
    assert_eq!(
        lab.owner([wire::INPUT_GROUP_RELEASE, group, 0, 0, 0, 0, 0, 0]),
        wire::error(abi::BUSY)
    );
    let retiring = directory(lab, 0, 0, 0);
    assert_eq!(&retiring[3..6], &[1, 1, wire::CHANNEL_RETIRING]);
    assert_ne!(retiring[2], revision);
    for slot in [0, 2] {
        assert_eq!(
            &lab.next(slot, epoch(slot as usize))[2..],
            &[wire::RETIRED, 0, 0, 0, 1, 1]
        );
    }
    assert_eq!(
        lab.owner([wire::INPUT_GROUP_RELEASE, group, 0, 0, 0, 0, 0, 0]),
        [0; 8]
    );
    for slot in [0, 2] {
        assert_eq!(
            &lab.next(slot, epoch(slot as usize))[2..],
            &[wire::GROUP_RETIRED, 0, 0, 0, 0, 1]
        );
    }
    assert_eq!(lab.next(1, epoch(1))[2], wire::NONE);
    assert_eq!(directory(lab, 0, 0, 0)[3], 0);
    assert_eq!(
        lab.owner([wire::INPUT_CHANNEL_CREATE, group, 0, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    assert_eq!(
        lab.owner([wire::INPUT_POINTER, channel, 20, 450, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    let fresh = created(lab.owner([wire::INPUT_GROUP_CREATE, 0, 0, 0, 0, 0, 0, 0]));
    assert!(fresh > group);
    let fresh_channel = created(lab.owner([wire::INPUT_CHANNEL_CREATE, fresh, 0, 0, 0, 0, 0, 0]));
    assert!(fresh_channel > channel);
    assert_eq!(
        lab.owner([wire::INPUT_CHANNEL_RETIRE, fresh_channel, 0, 0, 0, 0, 0, 0]),
        [0; 8]
    );
    assert_eq!(
        lab.owner([wire::INPUT_GROUP_RELEASE, fresh, 0, 0, 0, 0, 0, 0]),
        [0; 8]
    );
    // A producer grant: private channels in a private group, by generation.
    let info = [wire::PRODUCER_INFO, 0, 0, 0, 0, 0, 0, 0];
    assert_eq!(lab.call(Some(0), info), wire::error(abi::DENIED));
    let grant = |slot: u64, count: u64, rights: u64| {
        [wire::INPUT_PRODUCER_GRANT, slot, count, rights, 0, 0, 0, 0]
    };
    assert_eq!(lab.owner(grant(0, 1, wire::PRODUCER_FOCUS)), [0; 8]);
    assert_eq!(
        lab.owner(grant(0, 1, wire::PRODUCER_FOCUS)),
        wire::error(abi::BUSY)
    );
    assert_eq!(lab.owner(grant(1, 5, 0)), wire::error(abi::DENIED));
    assert_eq!(lab.owner(grant(1, 1, 2)), wire::error(abi::DENIED));
    assert_eq!(
        lab.call(Some(0), info),
        [0, 1, 300, 600, wire::KEY_MASK, 1, wire::PRODUCER_FOCUS, 0]
    );
    assert_eq!(lab.call(Some(1), info), wire::error(abi::DENIED));
    let described = Description::new(description::POINTER | description::KEYS, "Virtual input")
        .unwrap()
        .words();
    let describe = |words: [u64; 3], generation: u64, ordinal: u64| {
        [
            wire::PRODUCER_DESCRIBE,
            generation,
            ordinal,
            words[0],
            words[1],
            words[2],
            0,
            0,
        ]
    };
    assert_eq!(lab.call(Some(0), describe(described, 1, 1)), [0; 8]);
    assert_eq!(lab.call(Some(0), describe(described, 1, 1)), [0; 8]);
    assert_eq!(
        lab.call(
            Some(0),
            describe([described[0], described[1], description::POINTER], 1, 1)
        ),
        wire::error(abi::BUSY)
    );
    assert_eq!(
        lab.call(Some(0), describe([0xff, 0, description::POINTER], 1, 1)),
        wire::error(abi::INVALID_ARGUMENT)
    );
    assert_eq!(
        lab.call(Some(0), describe(described, 1, 2)),
        wire::error(abi::DENIED)
    );
    assert_eq!(
        lab.call(Some(0), describe(described, 2, 1)),
        wire::error(abi::DENIED)
    );
    let pointer = |generation: u64, ordinal: u64, x: u64, y: u64, buttons: u64| {
        [
            wire::PRODUCER_POINTER,
            generation,
            ordinal,
            x,
            y,
            buttons,
            0,
            0,
        ]
    };
    assert_eq!(lab.call(Some(0), pointer(1, 1, 20, 450, 1)), [0; 8]);
    let announced = lab.next(2, epoch(2));
    assert_eq!(announced[2], wire::DESCRIBED);
    assert_eq!(&announced[3..6], &described);
    assert_eq!(&announced[6..], &[2, 2]);
    assert_eq!(
        &lab.next(2, epoch(2))[2..],
        &[wire::POINTER, 20, 50, 1, 2, 2]
    );
    let row = directory(lab, 2, 0, 0);
    assert_eq!(&row[3..], &[2, 2, wire::CHANNEL_LIVE, wire::KEY_MASK, 0]);
    let description_of = |lab: &mut Lab, revision: u64, name: u64| {
        lab.call(
            Some(2),
            [
                wire::INPUT_DESCRIPTION,
                epoch(2),
                revision,
                name,
                0,
                0,
                0,
                0,
            ],
        )
    };
    assert_eq!(
        description_of(lab, row[2], 2),
        [
            0,
            epoch(2),
            row[2],
            2,
            described[0],
            described[1],
            described[2],
            0
        ]
    );
    assert_eq!(description_of(lab, row[2], 1), wire::error(abi::DENIED));
    assert_eq!(description_of(lab, 0, 2), wire::error(abi::DENIED));
    assert_eq!(lab.call(Some(0), pointer(1, 1, 20, 450, 0)), [0; 8]);
    assert_eq!(&lab.next(2, epoch(2))[2..6], &[wire::POINTER, 20, 50, 0]);
    for bad in [
        pointer(1, 1, 300, 450, 0),
        pointer(2, 1, 20, 450, 0),
        pointer(1, 2, 20, 450, 0),
    ] {
        assert_eq!(lab.call(Some(0), bad), wire::error(abi::DENIED));
    }
    // The focus right let that press select; keys follow it.
    let producer_key = |generation: u64, state: u64| {
        [
            wire::PRODUCER_KEY,
            generation,
            1,
            input::ENTER as u64,
            state,
            0,
            0,
            0,
        ]
    };
    for state in [input::PRESS, input::RELEASE] {
        assert_eq!(lab.call(Some(0), producer_key(1, state as u64)), [0; 8]);
        assert_eq!(
            &lab.next(2, epoch(2))[2..],
            &[wire::KEY, input::ENTER as u64, state as u64, 0, 2, 2]
        );
    }
    // Without that right a producer delivers pointer events but never selects.
    assert_eq!(lab.owner(grant(1, 1, 0)), [0; 8]);
    assert_eq!(
        lab.call(Some(1), info),
        [0, 1, 300, 600, wire::KEY_MASK, 1, 0, 0]
    );
    assert_eq!(lab.call(Some(1), pointer(1, 1, 20, 50, 1)), [0; 8]);
    assert_eq!(
        &lab.next(0, epoch(0))[2..],
        &[wire::POINTER, 20, 50, 1, 2, 2]
    );
    assert_eq!(
        lab.call(Some(1), producer_key(1, input::PRESS as u64)),
        [0; 8]
    );
    assert_eq!(lab.next(0, epoch(0))[2], wire::NONE);
    assert_eq!(
        lab.call(Some(1), producer_key(1, input::RELEASE as u64)),
        [0; 8]
    );
    assert_eq!(lab.call(Some(1), pointer(1, 1, 20, 50, 0)), [0; 8]);
    assert_eq!(&lab.next(0, epoch(0))[2..6], &[wire::POINTER, 20, 50, 0]);
    // Release retires the held channel; capacity returns only as recipients drain.
    assert_eq!(
        lab.call(Some(0), [wire::PRODUCER_RELEASE, 1, 0, 0, 0, 0, 0, 0]),
        [0; 8]
    );
    assert_eq!(lab.call(Some(0), info), wire::error(abi::DENIED));
    assert_eq!(lab.owner(grant(0, 3, 0)), wire::error(abi::NO_MEMORY));
    assert_eq!(&lab.next(2, epoch(2))[2..], &[wire::RETIRED, 0, 0, 0, 2, 2]);
    assert_eq!(lab.owner(grant(0, 3, 0)), wire::error(abi::NO_MEMORY));
    assert_eq!(lab.owner(grant(0, 2, 0)), [0; 8]);
    assert_eq!(
        lab.call(Some(0), info),
        [0, 2, 300, 600, wire::KEY_MASK, 2, 0, 0]
    );
    assert_eq!(
        &lab.next(2, epoch(2))[2..],
        &[wire::GROUP_RETIRED, 0, 0, 0, 0, 2]
    );
    // Owner revocation ends a grant the same way; the next registration reaps its group.
    assert_eq!(
        lab.owner([wire::INPUT_PRODUCER_REVOKE, 1, 0, 0, 0, 0, 0, 0]),
        [0; 8]
    );
    assert_eq!(lab.call(Some(1), info), wire::error(abi::DENIED));
    assert_eq!(&lab.next(0, epoch(0))[2..], &[wire::RETIRED, 0, 0, 0, 2, 2]);
    assert_eq!(
        lab.owner([wire::INPUT_PRODUCER_REVOKE, 0, 0, 0, 0, 0, 0, 0]),
        [0; 8]
    );
    assert_eq!(lab.call(Some(0), info), wire::error(abi::DENIED));
    // Capacity limits are the recipient's own ceiling; the cursor preference
    // follows the pointer's current target.
    let fresh = created(lab.owner([wire::INPUT_GROUP_CREATE, 0, 0, 0, 0, 0, 0, 0]));
    assert_eq!(
        &lab.next(0, epoch(0))[2..],
        &[wire::GROUP_RETIRED, 0, 0, 0, 0, 2]
    );
    let fresh_channel = created(lab.owner([wire::INPUT_CHANNEL_CREATE, fresh, 0, 0, 0, 0, 0, 0]));
    let capacity = |lab: &mut Lab, epoch: u64, channels: u64, groups: u64| {
        lab.call(
            Some(1),
            [wire::INPUT_CAPACITY, epoch, channels, groups, 0, 0, 0, 0],
        )
    };
    assert_eq!(
        capacity(lab, epoch(1), 0, 0),
        [0, epoch(1), 0, 0, 0, 0, 0, 0]
    );
    let hover = |x: u64, y: u64| [wire::INPUT_POINTER, fresh_channel, x, y, 0, 0, 0, 0];
    assert_eq!(lab.owner(hover(20, 250)), wire::error(abi::NO_MEMORY));
    assert_eq!(lab.next(1, epoch(1))[2], wire::NONE);
    assert_eq!(capacity(lab, epoch(1), 5, 1), wire::error(abi::DENIED));
    assert_eq!(capacity(lab, epoch(1) + 1, 1, 1), wire::error(abi::DENIED));
    assert_eq!(
        capacity(lab, epoch(1), 1, 1),
        [0, epoch(1), 1, 1, 0, 0, 0, 0]
    );
    assert_eq!(lab.owner(hover(20, 250)), [0; 8]);
    assert_eq!(
        &lab.next(1, epoch(1))[2..],
        &[wire::POINTER, 20, 50, 0, 1, 1]
    );
    assert_eq!(capacity(lab, epoch(1), 0, 0), wire::error(abi::BUSY));
    assert_eq!(capacity(lab, epoch(1), 1, 0), wire::error(abi::BUSY));
    let preference = |lab: &mut Lab, name: u64, mode: u64| {
        lab.call(
            Some(1),
            [wire::CURSOR_PREFERENCE, epoch(1), name, mode, 0, 0, 0, 0],
        )
    };
    assert_eq!(preference(lab, 1, wire::CURSOR_HIDDEN), [0; 8]);
    assert_eq!(
        preference(lab, 99, wire::CURSOR_HIDDEN),
        wire::error(abi::DENIED)
    );
    assert_eq!(preference(lab, 1, 2), wire::error(abi::DENIED));
    let cursor = [CURSOR, fresh_channel, 0, 0, 0, 0, 0, 0];
    assert_eq!(
        lab.owner(cursor),
        [0, 1, fresh_channel, 0, 1, wire::CURSOR_HIDDEN, 0, 0]
    );
    assert_eq!(lab.owner(hover(20, 50)), [0; 8]);
    assert_eq!(
        lab.owner(cursor),
        [0, 1, fresh_channel, 0, 1, wire::CURSOR_DEFAULT, 0, 0]
    );
    assert_eq!(lab.next(0, epoch(0))[2], wire::POINTER);
    assert_eq!(lab.owner([CURSOR, 99, 0, 0, 0, 0, 0, 0]), [0; 8]);
    assert_eq!(
        lab.owner([wire::INPUT_CHANNEL_RETIRE, fresh_channel, 0, 0, 0, 0, 0, 0]),
        [0; 8]
    );
    assert_eq!(lab.owner(cursor), [0; 8]);
    for slot in [0, 1] {
        assert_eq!(lab.next(slot, epoch(slot as usize))[2], wire::RETIRED);
    }
    assert_eq!(
        lab.owner([wire::INPUT_GROUP_RELEASE, fresh, 0, 0, 0, 0, 0, 0]),
        [0; 8]
    );
    for slot in [0, 1] {
        assert_eq!(lab.next(slot, epoch(slot as usize))[2], wire::GROUP_RETIRED);
    }
    assert_eq!(
        capacity(lab, epoch(1), 4, 4),
        [0, epoch(1), 4, 4, 0, 0, 0, 0]
    );
}

/// Delegated authority on the Omega matrix: the layout controller moves every
/// live view under one validated arrangement, the owner's placement snapshot
/// reissues the controller's view to its replacement, a management grant lets
/// one child observe and terminate a sibling's exact task, and launch receipts
/// stay local to the endpoint that recorded them. Owner words 907 to 913 are
/// proof-only orchestration; the management and launch words are the platform's.
#[cfg(feature = "omega-matrix")]
fn delegation(lab: &mut Lab, views: &mut [wire::Packet; 3]) {
    const GRANT_LAYOUT: u64 = 907;
    const PLACEMENT: u64 = 908;
    const RESTORE: u64 = 909;
    const LAYOUT_ROW: u64 = 910;
    const LAYOUT_APPLY: u64 = 911;
    const LAUNCH_RECORD: u64 = 912;
    const LAUNCH_FINISHED: u64 = 913;
    let row = |index: u64, y: u64, height: u64| [LAYOUT_ROW, index, 0, y, 300, height, 0, 0];
    let apply = |epoch: u64| [LAYOUT_APPLY, epoch, 0, 0, 0, 0, 0, 0];
    *views = [0, 1, 2].map(|slot| lab.view(slot));
    // Swap the first two regions. Nothing moves without the owner's grant, under
    // another slot or epoch, or when any rectangle fails to fit.
    for (index, y) in [(0, 200), (1, 0), (2, 400)] {
        assert_eq!(lab.call(Some(0), row(index, y, 200)), [0; 8]);
    }
    assert_eq!(
        lab.call(Some(0), apply(views[0][1])),
        wire::error(abi::DENIED)
    );
    assert_eq!(lab.owner([GRANT_LAYOUT, 0, 0, 0, 0, 0, 0, 0]), [0; 8]);
    assert_eq!(
        lab.call(Some(1), apply(views[1][1])),
        wire::error(abi::DENIED)
    );
    assert_eq!(
        lab.call(Some(0), apply(views[0][1] + 1)),
        wire::error(abi::DENIED)
    );
    assert_eq!(lab.call(Some(0), row(1, 0, 700)), [0; 8]);
    assert_eq!(
        lab.call(Some(0), apply(views[0][1])),
        wire::error(abi::INVALID_ARGUMENT)
    );
    assert_eq!(lab.call(Some(0), row(1, 0, 200)), [0; 8]);
    assert_eq!(lab.call(Some(0), apply(views[0][1])), [0; 8]);
    // Unchanged extents keep every view; the pointer now finds the second peer
    // at the top. The first sample only unblocks the source's interrupted hold.
    assert_eq!(lab.view(0), views[0]);
    assert_eq!(lab.view(1), views[1]);
    for _ in 0..2 {
        assert_eq!(
            lab.owner([wire::POINTER_SOURCE, 20, 50, 0, 0, 0, 0, 0]),
            [0; 8]
        );
    }
    assert_eq!(&lab.next(1, views[1][1])[2..6], &[wire::POINTER, 20, 50, 0]);
    assert_eq!(lab.next(0, views[0][1])[2], wire::NONE);
    // A shrunk third region changes only that view's epoch.
    assert_eq!(lab.call(Some(0), row(2, 400, 150)), [0; 8]);
    assert_eq!(lab.call(Some(0), apply(views[0][1])), [0; 8]);
    let shrunk = lab.view(2);
    assert_eq!(&shrunk[2..4], &[300, 150]);
    assert_ne!(shrunk[1], views[2][1]);
    assert_eq!(lab.view(0), views[0]);
    assert_eq!(lab.call(Some(0), row(2, 400, 200)), [0; 8]);
    assert_eq!(lab.call(Some(0), apply(views[0][1])), [0; 8]);
    views[2] = lab.view(2);
    assert_eq!(&views[2][2..4], &[300, 200]);
    // Only the live controller's placed, backed view can be captured; its
    // replacement receives exactly that view, its surface range and the grant.
    assert_eq!(lab.owner([PLACEMENT, 1, 0, 0, 0, 0, 0, 0]), [0; 8]);
    assert_eq!(
        lab.owner([PLACEMENT, 0, 0, 0, 0, 0, 0, 0]),
        [0, 1, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(lab.owner([wire::STOP, 0, 0, 0, 0, 0, 0, 0]), [0; 8]);
    assert_eq!(lab.owner([wire::SPAWN, 0, 0, 0, 0, 0, 0, 0]), [0; 8]);
    assert_eq!(lab.view(0), wire::error(abi::DENIED));
    assert_eq!(lab.owner([RESTORE, 0, 1, 0, 0, 0, 0, 0]), [0; 8]);
    views[0] = lab.view(0);
    assert_eq!(&views[0][2..5], &[300, 200, wire::KEY_MASK]);
    assert_eq!(
        lab.call(Some(0), [wire::SURFACE_INFO, 0, 0, 0, 0, 0, 0, 0]),
        [0, 4, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(
        lab.owner([RESTORE, 0, 1, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    assert_eq!(lab.call(Some(0), apply(views[0][1])), [0; 8]);
    assert_eq!(lab.view(1), views[1]);
    // Management: a grant lets one child observe and terminate a sibling's
    // exact task. Tickets follow the task, not the slot, and die with the grant.
    let task = [wire::MANAGED_TASK, 0, 0, 0, 0, 0, 0, 0];
    let terminate = |ticket: u64| [wire::TERMINATE_TASK, ticket, 0, 0, 0, 0, 0, 0];
    assert_eq!(lab.call(Some(0), task), wire::error(abi::DENIED));
    assert_eq!(
        lab.owner([wire::MANAGEMENT_GRANT, 0, 1, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    assert_eq!(
        lab.owner([wire::MANAGEMENT_GRANT, 5, 1, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    assert_eq!(
        lab.owner([wire::MANAGEMENT_GRANT, 0, 3, 0, 0, 0, 0, 0]),
        [0; 8]
    );
    assert_eq!(lab.call(Some(0), task), [0, 1, 0, 0, 0, 0, 0, 0]);
    assert_eq!(lab.call(Some(0), task), [0, 1, 0, 0, 0, 0, 0, 0]);
    assert_eq!(lab.call(Some(1), task), wire::error(abi::DENIED));
    assert_eq!(lab.call(Some(0), terminate(2)), wire::error(abi::DENIED));
    assert_eq!(lab.call(Some(1), terminate(1)), wire::error(abi::DENIED));
    assert_eq!(lab.call(Some(0), terminate(1)), [0; 8]);
    assert_eq!(lab.call(Some(0), task), [0; 8]);
    assert_eq!(lab.owner([wire::SPAWN, 2, 0, 0, 0, 0, 0, 0]), [0; 8]);
    assert_eq!(
        lab.owner([wire::PLACE, 2, 0, 400, 300, 200, wire::KEY_MASK, 0]),
        [0; 8]
    );
    assert_eq!(
        lab.owner([wire::SURFACE_LIMIT, 2, 4, 0, 0, 0, 0, 0]),
        [0; 8]
    );
    views[2] = lab.view(2);
    assert_eq!(lab.call(Some(0), task), [0, 2, 0, 0, 0, 0, 0, 0]);
    assert_eq!(lab.call(Some(0), terminate(1)), wire::error(abi::DENIED));
    assert_eq!(
        lab.owner([wire::MANAGEMENT_GRANT, 0, 0, 0, 0, 0, 0, 0]),
        [0; 8]
    );
    assert_eq!(lab.call(Some(0), task), wire::error(abi::DENIED));
    // Launch receipts stay local to the endpoint that recorded them and follow
    // the exact child; a replacement connection starts counting afresh.
    let status = |receipt: u64| [wire::LAUNCH_STATUS, receipt, 0, 0, 0, 0, 0, 0];
    assert_eq!(lab.call(Some(1), status(1)), wire::error(abi::DENIED));
    assert_eq!(
        lab.owner([LAUNCH_RECORD, 1, 777, 0, 0, 0, 0, 0]),
        [0, 1, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(
        lab.call(Some(1), status(1)),
        [0, wire::LAUNCH_RUNNING, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(lab.call(Some(0), status(1)), wire::error(abi::DENIED));
    assert_eq!(lab.call(Some(1), status(2)), wire::error(abi::DENIED));
    assert_eq!(
        lab.owner([LAUNCH_FINISHED, 777, wire::LAUNCH_EXITED, 0, 0, 0, 0, 0]),
        [0, 2, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(
        lab.owner([LAUNCH_FINISHED, 777, wire::LAUNCH_EXITED, 0, 0, 0, 0, 0]),
        [0; 8]
    );
    assert_eq!(
        lab.call(Some(1), status(1)),
        [0, wire::LAUNCH_EXITED, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(lab.owner([wire::STOP, 1, 0, 0, 0, 0, 0, 0]), [0; 8]);
    assert_eq!(lab.owner([wire::SPAWN, 1, 0, 0, 0, 0, 0, 0]), [0; 8]);
    assert_eq!(lab.call(Some(1), status(1)), wire::error(abi::DENIED));
    assert_eq!(
        lab.owner([LAUNCH_RECORD, 1, 778, 0, 0, 0, 0, 0]),
        [0, 1, 0, 0, 0, 0, 0, 0]
    );
}
