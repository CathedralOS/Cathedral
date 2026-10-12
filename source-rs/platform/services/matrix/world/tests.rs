use super::*;
use cathedral_contracts::input;

#[test]
fn retained_geometry_moves_without_child_traffic_and_revocation_erases_it() {
    let mut w = world();
    place(&mut w, 0, 10, 20);
    let epoch = view(&mut w, 0)[1];
    w.surface(0, 7, [wire::PAINT, epoch, 0, 5, 7, 400, 400, 0x123456])
        .unwrap();
    assert!(w.take_notification(0));
    place(&mut w, 0, 200, 200);
    assert!(!w.take_notification(0)); // Position is outside the child's view.
    assert_eq!(
        w.replay(0)[0].unwrap(),
        [wire::PAINT, 1, 1, 205, 207, 95, 73, 0x123456]
    );
    w.revoke(0).unwrap();
    assert!(w.take_notification(0));
    place(&mut w, 0, 10, 20);
    assert_eq!(w.replay(0), [None; 3]);
}

#[test]
fn wake_hints_coalesce_and_remain_child_local() {
    let mut w = world();
    place(&mut w, 0, 0, 0);
    place(&mut w, 1, 200, 0);
    w.take_notification(0);
    w.take_notification(1);
    w.pointer(10, 10, 0);
    w.pointer(11, 11, 0);
    assert!(w.take_notification(0));
    assert!(!w.take_notification(0));
    assert!(!w.take_notification(1));
    let epoch = view(&mut w, 0)[1];
    assert_eq!(next(&mut w, 0, epoch)[2], wire::POINTER);
    assert_eq!(next(&mut w, 0, epoch)[2], wire::POINTER);
}

#[test]
fn nested_surfaces_clip_translate_and_cannot_name_a_siblings_surface() {
    let mut outer = world();
    place(&mut outer, 0, 100, 100);
    let parent = view(&mut outer, 0);
    let mut inner = World::default();
    inner.inherit(Grant {
        epoch: parent[1],
        width: parent[2],
        height: parent[3],
        keys: parent[4],
    });
    place(&mut inner, 0, 10, 20);
    let leaf = view(&mut inner, 0);
    let paint = [wire::PAINT, leaf[1], 0, 5, 7, 400, 400, 0x123456];
    let local = inner.surface(0, 7, paint).unwrap();
    assert_eq!(local, [wire::PAINT, parent[1], 1, 15, 27, 85, 53, 0x123456]);
    assert_eq!(
        outer.surface(0, 7, local).unwrap(),
        [wire::PAINT, 1, 5, 115, 127, 85, 53, 0x123456]
    );
    for (at, value) in [(1, leaf[1] + 1), (2, 4), (3, u64::MAX), (7, 0x1000000)] {
        let mut invalid = paint;
        invalid[at] = value;
        assert_eq!(inner.surface(0, 7, invalid), Err(abi::DENIED));
    }
    assert_eq!(inner.surface(1, 7, paint), Err(abi::DENIED));
    outer.revoke(0).unwrap();
    assert_eq!(outer.surface(0, 7, local), Err(abi::DENIED));
    // Explicit teardown invalidates the old incarnation before owner regranting.
    outer.disconnected(0);
    place(&mut outer, 0, 200, 200);
    assert_eq!(
        outer.request(0, 99, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0])[0],
        0
    );
}
fn world() -> World {
    let mut world = World::default();
    world.inherit(Grant {
        epoch: 1,
        width: 800,
        height: 600,
        keys: wire::KEY_MASK,
    });
    world
}
fn place(w: &mut World, slot: usize, x: u64, y: u64) {
    w.place(
        slot,
        Rect {
            x,
            y,
            width: 100,
            height: 80,
        },
        wire::KEY_MASK,
    )
    .unwrap();
}
fn view(w: &mut World, slot: usize) -> wire::Packet {
    w.request(slot, 7, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0])
}
fn next(w: &mut World, slot: usize, epoch: u64) -> wire::Packet {
    w.request(slot, 7, [wire::NEXT, epoch, 0, 0, 0, 0, 0, 0])
}

#[test]
fn parent_position_siblings_and_unused_parent_extent_are_unobservable() {
    let mut a = world();
    let mut b = world();
    place(&mut a, 0, 10, 20);
    place(&mut b, 0, 310, 220);
    assert_eq!(view(&mut a, 0), view(&mut b, 0));
    place(&mut b, 1, 500, 400);
    for _ in 0..20 {
        b.revoke(1).unwrap();
        place(&mut b, 1, 500, 400);
    }
    b.inherit(Grant {
        epoch: 1,
        width: 900,
        height: 700,
        keys: wire::KEY_MASK,
    });
    place(&mut b, 0, 330, 250);
    let av = view(&mut a, 0);
    assert_eq!(av, view(&mut b, 0));
    a.pointer(15, 26, 1);
    b.pointer(335, 256, 1);
    assert_eq!(next(&mut a, 0, av[1]), next(&mut b, 0, av[1]));
    a.pointer(799, 599, 1);
    b.pointer(899, 699, 1);
    assert_eq!(next(&mut a, 0, av[1]), next(&mut b, 0, av[1]));
}
#[test]
fn unsupported_queries_never_disclose_presence_or_geometry() {
    let mut w = world();
    place(&mut w, 0, 40, 50);
    for op in [
        wire::PLACE,
        wire::REVOKE,
        wire::BIND,
        wire::HEALTH,
        u64::MAX,
    ] {
        assert_eq!(
            w.request(0, 7, [op, 0, 0, 0, 0, 0, 0, 0]),
            wire::error(abi::DENIED)
        );
    }
    assert_eq!(
        w.request(0, 7, [wire::VIEW, 1, 0, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    assert_eq!(view(&mut w, 1), wire::error(abi::DENIED));
}
#[test]
fn the_same_world_routes_two_levels_with_clamped_capture_and_key_attenuation() {
    let mut outer = world();
    place(&mut outer, 0, 100, 100);
    let grant = view(&mut outer, 0);
    let mut inner = World::default();
    inner.inherit(Grant {
        epoch: grant[1],
        width: grant[2],
        height: grant[3],
        keys: grant[4],
    });
    inner
        .place(
            0,
            Rect {
                x: 10,
                y: 10,
                width: 40,
                height: 30,
            },
            1 << (input::ENTER - 1),
        )
        .unwrap();
    let leaf = view(&mut inner, 0);
    for (x, y, buttons) in [(115, 117, 1), (799, 599, 1), (799, 599, 0)] {
        outer.pointer(x, y, buttons);
        let e = next(&mut outer, 0, grant[1]);
        inner.pointer(e[3], e[4], e[5]);
        let e = next(&mut inner, 0, leaf[1]);
        assert_eq!(e[2], wire::POINTER);
        assert!(e[3] < 40 && e[4] < 30);
    }
    inner.key(input::UP as u64, input::PRESS as u64);
    assert_eq!(next(&mut inner, 0, leaf[1])[2], wire::NONE);
    inner.key(input::ENTER as u64, input::PRESS as u64);
    assert_eq!(next(&mut inner, 0, leaf[1])[2], wire::KEY);
}
#[test]
fn parent_revocation_drops_queued_input_and_requires_explicit_descendant_regrant() {
    let mut w = world();
    place(&mut w, 0, 10, 10);
    let old = view(&mut w, 0);
    w.pointer(20, 20, 1);
    w.inherit(Grant::default());
    assert_eq!(next(&mut w, 0, old[1]), wire::error(abi::DENIED));
    w.inherit(Grant {
        epoch: 2,
        width: 800,
        height: 600,
        keys: wire::KEY_MASK,
    });
    assert_eq!(view(&mut w, 0), wire::error(abi::DENIED));
    place(&mut w, 0, 10, 10);
    assert_ne!(view(&mut w, 0)[1], old[1]);
    assert_eq!(next(&mut w, 0, old[1]), wire::error(abi::DENIED));
}
#[test]
fn replacing_a_connection_cannot_inherit_its_predecessors_grant() {
    let mut w = world();
    place(&mut w, 0, 0, 0);
    let old = view(&mut w, 0);
    assert_eq!(
        w.request(0, 8, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    place(&mut w, 0, 0, 0);
    assert_eq!(
        w.request(0, 8, [wire::NEXT, old[1], 0, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
}

#[test]
fn rapid_parent_revoke_and_regrant_cannot_revive_descendants() {
    let mut w = world();
    place(&mut w, 0, 10, 10);
    let old = view(&mut w, 0);
    // The next observation skips the intermediate invalid parent grant.
    w.inherit(Grant {
        epoch: 2,
        width: 800,
        height: 600,
        keys: wire::KEY_MASK,
    });
    assert_eq!(view(&mut w, 0), wire::error(abi::DENIED));
    assert_eq!(next(&mut w, 0, old[1]), wire::error(abi::DENIED));
}

#[test]
fn focus_cancellation_does_not_transfer_a_held_key_to_a_sibling() {
    let mut w = world();
    place(&mut w, 0, 0, 0);
    place(&mut w, 1, 200, 0);
    let a = view(&mut w, 0)[1];
    let b = view(&mut w, 1)[1];
    w.pointer(10, 10, 1);
    w.pointer(10, 10, 0);
    w.key(input::ENTER as u64, input::PRESS as u64);
    w.pointer(210, 10, 1);
    w.pointer(210, 10, 0);
    assert_eq!(next(&mut w, 0, a)[2], wire::CANCEL);
    assert_eq!(next(&mut w, 0, a)[2], wire::NONE);
    assert_eq!(next(&mut w, 1, b)[2], wire::POINTER);
    assert_eq!(next(&mut w, 1, b)[2], wire::POINTER);
    w.key(input::ENTER as u64, input::REPEAT as u64);
    w.key(input::ENTER as u64, input::RELEASE as u64);
    assert_eq!(next(&mut w, 1, b)[2], wire::NONE);
    w.key(input::ENTER as u64, input::PRESS as u64);
    assert_eq!(next(&mut w, 1, b)[2], wire::KEY);
}

#[test]
fn overflowing_one_child_does_not_emit_events_or_metadata_to_its_sibling() {
    let mut w = world();
    place(&mut w, 0, 0, 0);
    place(&mut w, 1, 200, 0);
    let a = view(&mut w, 0)[1];
    let b = view(&mut w, 1);
    for _ in 0..9 {
        w.pointer(10, 10, 0);
    }
    assert_eq!(next(&mut w, 0, a)[2], wire::CANCEL);
    assert_eq!(view(&mut w, 1), b);
    assert_eq!(next(&mut w, 1, b[1])[2], wire::NONE);
}

#[test]
fn cancellation_cannot_turn_a_held_button_into_a_fresh_press() {
    let mut w = world();
    place(&mut w, 0, 0, 0);
    let epoch = view(&mut w, 0)[1];
    w.pointer(10, 10, 1);
    w.cancel();
    assert_eq!(next(&mut w, 0, epoch)[2], wire::CANCEL);
    w.pointer(10, 10, 1);
    w.pointer(10, 10, 0);
    assert_eq!(next(&mut w, 0, epoch)[2], wire::NONE);
    w.pointer(10, 10, 1);
    assert_eq!(next(&mut w, 0, epoch)[2], wire::POINTER);
    w.key(input::ENTER as u64, input::RELEASE as u64);
    w.key(input::ENTER as u64, input::REPEAT as u64);
    assert_eq!(next(&mut w, 0, epoch)[2], wire::NONE);
}

#[test]
fn atomic_poll_is_local_reports_more_and_drops_events_on_rebinding() {
    let mut w = world();
    place(&mut w, 0, 10, 20);
    place(&mut w, 1, 200, 0);
    let sibling = view(&mut w, 1);
    w.pointer(15, 27, 1);
    w.pointer(16, 28, 0);
    let first = w.request(0, 7, [wire::POLL, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(wire::poll_view(first), view(&mut w, 0));
    assert_ne!(first[5] & wire::MORE, 0);
    assert_eq!(&wire::poll_event(first)[2..6], &[wire::POINTER, 5, 7, 1]);
    let last = w.request(0, 7, [wire::POLL, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(last[5] & wire::MORE, 0);
    assert_eq!(&wire::poll_event(last)[2..6], &[wire::POINTER, 6, 8, 0]);
    assert_eq!(view(&mut w, 1), sibling);
    w.key(input::ENTER as u64, input::PRESS as u64);
    w.revoke(0).unwrap();
    assert_eq!(
        w.request(0, 7, [wire::POLL, 0, 0, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    place(&mut w, 0, 10, 20);
    let fresh = w.request(0, 7, [wire::POLL, 0, 0, 0, 0, 0, 0, 0]);
    assert_ne!(fresh[1], first[1]);
    assert_eq!(wire::poll_event(fresh)[2], wire::NONE);
}
