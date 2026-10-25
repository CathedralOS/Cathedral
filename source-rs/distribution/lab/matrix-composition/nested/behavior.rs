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
