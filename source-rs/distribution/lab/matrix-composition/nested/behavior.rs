//! Assertions cross both IPC edges; this fixture never mutates the inner World.
use super::parent::Parent;
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::{input, matrix as wire};
pub(super) fn check(parent: &mut Parent) {
    let views = [0, 1, 2].map(|slot| parent.view(slot));
    for view in views {
        assert_eq!(view[0], 0);
        assert_eq!(&view[2..5], &[240, 150, 16]);
        assert_eq!(&view[5..], &[0; 3]);
    }
    // The parent owns (100,80); inner peer 2 owns (0,300). Only local (20,30) escapes.
    assert_eq!(
        parent
            .world
            .source_command([wire::POINTER_SOURCE, 120, 410, 1, 0, 0, 0, 0]),
        [0; 8]
    );
    assert_eq!(
        &parent.next(2, views[2][1])[2..6],
        &[wire::POINTER, 20, 30, 1]
    );
    assert_eq!(parent.next(0, views[0][1])[2], wire::NONE);
    assert_eq!(parent.next(1, views[1][1])[2], wire::NONE);
    for state in [input::PRESS, input::RELEASE] {
        assert_eq!(
            parent.world.source_command([
                wire::KEY_SOURCE,
                input::ENTER as u64,
                state as u64,
                0,
                0,
                0,
                0,
                0
            ]),
            [0; 8]
        );
    }
    // Both edges must survive even though the leaf did not poll between them.
    for state in [input::PRESS, input::RELEASE] {
        assert_eq!(
            &parent.next(2, views[2][1])[2..5],
            &[wire::KEY, input::ENTER as u64, state as u64]
        );
    }
    for state in [input::PRESS, input::RELEASE] {
        assert_eq!(
            parent.world.source_command([
                wire::KEY_SOURCE,
                input::UP as u64,
                state as u64,
                0,
                0,
                0,
                0,
                0
            ]),
            [0; 8]
        );
    }
    assert_eq!(parent.next(2, views[2][1])[2], wire::NONE);
    for words in [
        [wire::PLACE, 0, 0, 0, 800, 800, 31, 0],
        [wire::KEY_SOURCE, 5, 1, 0, 0, 0, 0, 0],
        [wire::REVOKE, 1, 0, 0, 0, 0, 0, 0],
    ] {
        assert_eq!(parent.call(words, Some(2)), wire::error(abi::DENIED));
    }
    // Even owner placement cannot grant keys the upstream parent withheld.
    assert_eq!(
        parent.call([wire::PLACE, 0, 0, 0, 240, 150, 31, 0], None),
        wire::error(abi::DENIED)
    );
    #[cfg(feature = "omega-matrix")]
    {
        // Surfaces cross both edges: the matrix allots its inherited allowance,
        // forwards a leaf's creation and paint, and this parent sees the result
        // clipped to the leaf's view and placed at (100,80) plus (0,300).
        assert_eq!(
            parent.call([wire::SURFACE_LIMIT, 2, 5, 0, 0, 0, 0, 0], None),
            wire::error(abi::NO_MEMORY)
        );
        assert_eq!(
            parent.call([wire::SURFACE_LIMIT, 2, 4, 0, 0, 0, 0, 0], None),
            [0; 8]
        );
        let info = [wire::SURFACE_INFO, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(parent.call(info, Some(2)), [0, 4, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            parent.call(
                [wire::SURFACE_CREATE, views[2][1], 0, 0, 0, 0, 0, 0],
                Some(2)
            ),
            [0, 1, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(
            parent.call(
                [wire::PAINT, views[2][1], 1, 10, 20, 50, 200, 0xabcdef],
                Some(2)
            ),
            [0; 8]
        );
        assert_eq!(
            parent.world.replay(0, 0),
            Some([wire::PAINT, 1, 100, 110, 400, 50, 130, 0xabcdef])
        );
        // Withdrawn demand reaches the leaf through the matrix's next poll.
        parent.world.render_demand(false);
        let poll = [wire::POLL, 0, 0, 0, 0, 0, 0, 0];
        assert_ne!(parent.call(poll, Some(2))[4] & wire::NO_FRAMES, 0);
        parent.world.render_demand(true);
        assert_eq!(parent.call(poll, Some(2))[4] & wire::NO_FRAMES, 0);
        // A channel this parent registers and describes reaches the leaf
        // described, under the matrix's own names; its retirement and then its
        // group's drain through the matrix in order, one poll per event.
        use cathedral_platform_contracts::input::description::{self, Description};
        let group = parent.world.register_input_group().unwrap();
        let channel = parent.world.register_input_stream(group).unwrap();
        let tablet = Description::new(description::POINTER, "Tablet").unwrap();
        parent.world.describe_input_stream(channel, tablet).unwrap();
        parent.world.pointer_stream(channel, 120, 410, 0).unwrap();
        let announced = parent.next(2, views[2][1]);
        assert_eq!(announced[2], wire::DESCRIBED);
        assert_eq!(&announced[3..6], &tablet.words());
        let motion = parent.next(2, views[2][1]);
        assert_eq!(&motion[2..6], &[wire::POINTER, 20, 30, 0]);
        assert_eq!(&motion[6..], &announced[6..]);
        parent.world.retire_input_stream(channel).unwrap();
        let retired = parent.next(2, views[2][1]);
        assert_eq!((retired[2], retired[6]), (wire::RETIRED, motion[6]));
        parent.world.release_input_group(group).unwrap();
        let ended = parent.next(2, views[2][1]);
        assert_eq!((ended[2], ended[7]), (wire::GROUP_RETIRED, motion[7]));
        assert!(parent.world.input_ready());
        // Revoking the leaf erases its surface upstream before anything else.
        assert_eq!(
            parent.call([wire::REVOKE, 2, 0, 0, 0, 0, 0, 0], None),
            [0; 8]
        );
        assert_eq!(parent.call(info, Some(2)), wire::error(abi::DENIED));
        assert_eq!(parent.world.replay(0, 0), None);
    }
    // A new parent view epoch invalidates old placements, including on resize.
    parent.place(120, 330, 0);
    assert_eq!(parent.view(2), wire::error(abi::DENIED));
    assert_eq!(
        parent.call([wire::PLACE, 2, 0, 300, 240, 150, 0, 0], None),
        [0; 8]
    );
    let small = parent.view(2);
    assert_eq!(&small[2..5], &[120, 30, 0]);
    assert_ne!(small[1], views[2][1]);
    assert_eq!(parent.next(2, views[2][1]), wire::error(abi::DENIED));
    // Leave an upstream event queued: revoke/regrant must not replay it later.
    assert_eq!(
        parent
            .world
            .source_command([wire::POINTER_SOURCE, 130, 390, 0, 0, 0, 0, 0]),
        [0; 8]
    );
    // Revocation cannot be repaired by restoring the parent's view alone.
    assert_eq!(
        parent
            .world
            .owner_command([wire::REVOKE, 0, 0, 0, 0, 0, 0, 0]),
        [0; 8]
    );
    for slot in 0..3 {
        assert_eq!(parent.view(slot), wire::error(abi::DENIED));
    }
    parent.place(240, 450, 16);
    for slot in 0..3 {
        assert_eq!(parent.view(slot), wire::error(abi::DENIED));
    }
    assert_eq!(
        parent.call([wire::PLACE, 2, 0, 300, 240, 150, 16, 0], None),
        [0; 8]
    );
    let fresh = parent.view(2);
    assert_eq!(&fresh[2..5], &[240, 150, 16]);
    assert_ne!(fresh[1], views[2][1]);
    assert_eq!(parent.next(2, views[2][1]), wire::error(abi::DENIED));
    assert_eq!(parent.next(2, fresh[1])[2], wire::NONE);
    assert_eq!(parent.view(0), wire::error(abi::DENIED));
}
