use super::*;
#[test]
fn retained_geometry_moves_without_child_traffic_and_revocation_erases_it() {
    let mut w = world();
    place(&mut w, 0, 10, 20);
    let epoch = view(&mut w, 0)[1];
    let name = surface(&mut w, 0, 0, 1);
    w.surface(0, 7, [wire::PAINT, epoch, name, 5, 7, 400, 400, 0x123456])
        .unwrap();
    assert!(w.take_notification(0));
    place(&mut w, 0, 200, 200);
    assert!(!w.take_notification(0)); // Position is outside the child's view.
    assert_eq!(
        w.replay(0, 0).unwrap(),
        [wire::PAINT, 1, 1, 205, 207, 95, 73, 0x123456]
    );
    w.revoke(0).unwrap();
    assert!(w.take_notification(0));
    place(&mut w, 0, 10, 20);
    assert!((0..wire::MAX_SURFACES).all(|rank| w.replay(0, rank).is_none()));
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
    inner.surface_capacity(8);
    inner.surface_limit(0, 4).unwrap();
    place(&mut inner, 0, 10, 20);
    let leaf = view(&mut inner, 0);
    let proxy = surface(&mut outer, 0, 0, 5);
    let name = surface(&mut inner, 0, 0, proxy);
    let paint = [wire::PAINT, leaf[1], name, 5, 7, 400, 400, 0x123456];
    let local = inner.surface(0, 7, paint).unwrap();
    assert_eq!(local, [wire::PAINT, parent[1], 1, 15, 27, 85, 53, 0x123456]);
    assert_eq!(
        outer.surface(0, 7, local).unwrap(),
        [wire::PAINT, 1, 5, 115, 127, 85, 53, 0x123456]
    );
    for (at, value) in [(1, leaf[1] + 1), (3, u64::MAX), (7, 0x1000000)] {
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
    world.surface_capacity(16);
    world.surface_limit(0, 8).unwrap();
    world.surface_limit(1, 8).unwrap();
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
fn surface(w: &mut World, slot: usize, rank: u64, upstream: u64) -> u64 {
    let epoch = view(w, slot)[1];
    w.prepare_surface(slot, 7, epoch, rank).unwrap();
    w.create_surface(slot, rank as usize, upstream)
}

#[test]
fn allowance_subdivision_is_backed_and_rank_cannot_cross_sibling_range() {
    let mut w = world();
    assert_eq!(w.surface_limit(0, 9), Err(abi::NO_MEMORY));
    place(&mut w, 0, 0, 0);
    place(&mut w, 1, 200, 0);
    let epoch = view(&mut w, 0)[1];
    for rank in 0..8 {
        assert_eq!(w.prepare_surface(0, 7, epoch, rank), Ok(rank));
        w.create_surface(0, rank as usize, rank + 100);
    }
    assert_eq!(w.prepare_surface(0, 7, epoch, 0), Err(abi::NO_MEMORY));
    assert_eq!(w.surface_limit(0, 7), Err(abi::BUSY));
    let other_epoch = view(&mut w, 1)[1];
    assert_eq!(w.prepare_surface(1, 7, other_epoch, 0), Ok(8));
    assert_eq!(w.prepare_surface(1, 7, other_epoch, 8), Err(abi::DENIED));
    let other = w.create_surface(1, 0, 300);
    assert_eq!(other, 1); // Independent caller-local namespace.
    assert_eq!(
        w.surface(1, 7, [wire::PAINT, other_epoch, 100, 0, 0, 1, 1, 0]),
        Err(abi::BAD_HANDLE)
    );
    w.revoke(0).unwrap();
    assert_eq!(drain(&mut w), 8);
    assert_eq!(
        w.request(1, 7, [wire::SURFACE_INFO, 0, 0, 0, 0, 0, 0, 0]),
        [0, 8, 1, 0, 0, 0, 0, 0]
    );
    assert_eq!(w.surface_limit(1, 9), Err(abi::BUSY));
}
#[test]
fn released_and_rebound_names_never_resurrect_and_sibling_activity_is_invisible() {
    let mut a = world();
    let mut b = world();
    for w in [&mut a, &mut b] {
        place(w, 0, 0, 0);
        place(w, 1, 200, 0);
    }
    let epoch = view(&mut a, 0)[1];
    let first = surface(&mut a, 0, 0, 100);
    assert_eq!(surface(&mut b, 0, 0, 900), first);
    for _ in 0..8 {
        surface(&mut b, 1, 0, 800);
        b.revoke(1).unwrap();
        drain(&mut b);
        b.surface_limit(1, 8).unwrap();
        place(&mut b, 1, 200, 0);
    }
    for w in [&mut a, &mut b] {
        w.surface(0, 7, [wire::ERASE, epoch, first, 0, 0, 0, 0, 0])
            .unwrap();
    }
    let next = surface(&mut a, 0, 0, 101);
    assert_eq!(surface(&mut b, 0, 0, 901), next);
    assert_ne!(next, first);
    assert_eq!(
        a.surface(0, 7, [wire::PAINT, epoch, first, 0, 0, 1, 1, 0]),
        Err(abi::BAD_HANDLE)
    );
    a.revoke(0).unwrap();
    drain(&mut a);
    a.surface_limit(0, 8).unwrap();
    place(&mut a, 0, 0, 0);
    let current = view(&mut a, 0)[1];
    assert_eq!(
        a.surface(0, 7, [wire::PAINT, current, next, 0, 0, 1, 1, 0]),
        Err(abi::BAD_HANDLE)
    );
    let fresh = surface(&mut a, 0, 0, 102);
    assert!(fresh > next);
    a.surface(0, 7, [wire::ERASE, current, fresh, 0, 0, 0, 0, 0])
        .unwrap();
    a.children[0].next_surface = u64::MAX;
    assert_eq!(a.prepare_surface(0, 7, current, 0), Err(abi::NO_MEMORY));
}
#[test]
fn three_levels_use_local_handles_and_conserve_stacking_ranges() {
    let mut outer = world();
    place(&mut outer, 0, 0, 0);
    let epoch = view(&mut outer, 0)[1];
    let mut middle = world();
    middle.surface_capacity(8);
    middle.surface_limit(0, 4).unwrap();
    middle.surface_limit(1, 4).unwrap();
    place(&mut middle, 1, 0, 0);
    let mut inner = world();
    inner.surface_capacity(4);
    inner.surface_limit(0, 2).unwrap();
    inner.surface_limit(1, 2).unwrap();
    place(&mut inner, 1, 0, 0);
    // Re-read each local epoch; no ancestor epoch or rank appears at the leaf.
    let leaf_epoch = view(&mut inner, 1)[1];
    let rank = inner.prepare_surface(1, 7, leaf_epoch, 1).unwrap();
    assert_eq!(rank, 3);
    let middle_epoch = view(&mut middle, 1)[1];
    let rank = middle.prepare_surface(1, 7, middle_epoch, rank).unwrap();
    assert_eq!(rank, 7);
    assert_eq!(outer.prepare_surface(0, 7, epoch, rank), Ok(7));
    // Even a deep rightmost descendant remains below the outside sibling's range.
    place(&mut outer, 1, 0, 0);
    let outside = view(&mut outer, 1)[1];
    assert_eq!(outer.prepare_surface(1, 7, outside, 0), Ok(8));
}

fn drain(world: &mut World) -> usize {
    let mut count = 0;
    while world.take_retired_surface().is_some() {
        count += 1;
    }
    count
}
