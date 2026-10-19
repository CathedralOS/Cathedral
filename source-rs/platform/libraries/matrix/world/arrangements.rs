//! Alternative compositions use only the platform API, with no stock contracts.
use super::*;
use cathedral_platform_contracts::input;

#[test]
fn three_children_use_owner_geometry_without_a_launcher_slot() {
    let mut world = World::<3>::empty();
    world.inherit(Grant {
        epoch: 1,
        width: 300,
        height: 600,
        keys: wire::KEY_MASK,
    });
    world.surface_capacity(12);
    let rects: [Rect; 3] = core::array::from_fn(|i| Rect {
        x: 0,
        y: i as u64 * 200,
        width: 300,
        height: 200,
    });
    let mut views = [[0; 8]; 3];
    for slot in 0..3 {
        world.surface_limit(slot, 4).unwrap();
        world.place(slot, rects[slot], wire::KEY_MASK).unwrap();
        views[slot] = world.request(slot, 7, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            world.prepare_surface(slot, 7, views[slot][1], 0),
            Ok(slot as u64 * 4)
        );
    }
    world.pointer(20, 420, 1);
    world.pointer(20, 420, 0);
    world.key(input::ENTER as u64, input::PRESS as u64);
    assert_eq!(
        world.request(0, 7, [wire::NEXT, views[0][1], 0, 0, 0, 0, 0, 0])[2],
        wire::NONE
    );
    let event = world.request(2, 7, [wire::NEXT, views[2][1], 0, 0, 0, 0, 0, 0]);
    assert_eq!(&event[2..6], &[wire::POINTER, 20, 20, 1]);
    // Any owner-selected child may control geometry, including the last one.
    world.grant_layout(2, 7);
    let swapped = [rects[1], rects[0], rects[2]];
    assert!(matches!(
        world.layout(0, 7, views[0][1], swapped),
        Err(abi::DENIED)
    ));
    world.layout(2, 7, views[2][1], swapped).unwrap();
    let retained = world.request(1, 7, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
    let snapshot = world.placement(2, 7).unwrap();
    world.disconnected(2);
    world.restore(snapshot, 8, true).unwrap();
    assert_eq!(
        world.request(1, 7, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]),
        retained
    );
    assert!(world.placement(2, 8).is_some());
}

#[test]
fn surface_ranges_do_not_move_when_another_child_changes_allowance() {
    let mut world = World::<3>::empty();
    world.inherit(Grant {
        epoch: 1,
        width: 300,
        height: 600,
        keys: 0,
    });
    world.surface_capacity(12);
    for slot in 0..3 {
        world.surface_limit(slot, 4).unwrap();
        world
            .place(
                slot,
                Rect {
                    x: 0,
                    y: slot as u64 * 100,
                    width: 300,
                    height: 100,
                },
                0,
            )
            .unwrap();
    }
    let view = world.request(2, 7, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(world.prepare_surface(2, 7, view[1], 0), Ok(8));
    world.create_surface(2, 0, 100);
    world.surface_limit(0, 0).unwrap();
    world.surface_limit(1, 8).unwrap();
    assert_eq!(world.prepare_surface(2, 7, view[1], 1), Ok(9));
    assert_eq!(world.surface_limit(0, 1), Err(abi::NO_MEMORY));
    assert_eq!(world.request(2, 7, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]), view);
}

#[test]
fn owner_stacking_is_independent_of_slot_and_allocation_order() {
    let mut world = World::<3>::empty();
    world.inherit(Grant {
        epoch: 1,
        width: 300,
        height: 200,
        keys: 0,
    });
    world.surface_capacity(12);
    // Slot two is the bottom layer, allocated after the top layer in slot zero.
    world.surface_range(0, 8, 4).unwrap();
    world.surface_range(2, 0, 4).unwrap();
    world.surface_range(1, 4, 4).unwrap();
    for slot in 0..3 {
        world
            .place(
                slot,
                Rect {
                    x: 0,
                    y: 0,
                    width: 300,
                    height: 200,
                },
                0,
            )
            .unwrap();
    }
    let view = world.request(0, 7, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(world.prepare_surface(0, 7, view[1], 0), Ok(8));
    world.create_surface(0, 0, 9);
    world.pointer(20, 20, 1);
    assert_eq!(
        world.request(0, 7, [wire::NEXT, view[1], 0, 0, 0, 0, 0, 0])[2],
        wire::POINTER
    );
    let bottom = world.request(2, 7, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        world.request(2, 7, [wire::NEXT, bottom[1], 0, 0, 0, 0, 0, 0])[2],
        wire::NONE
    );
    world.pointer(20, 20, 0);
    assert_eq!(world.surface_range(1, 7, 4), Err(abi::NO_MEMORY));
    world.surface_range(2, 0, 0).unwrap();
    assert_eq!(world.surface_range(0, 0, 4), Err(abi::BUSY));
    world.grant_layout(0, 7);
    let saved = world.placement(0, 7).unwrap();
    world.disconnected(0);
    world.restore(saved, 8, true).unwrap();
    let view = world.request(0, 8, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(world.prepare_surface(0, 8, view[1], 0), Ok(8));
}

#[test]
fn headless_four_child_authority_needs_no_display_or_desktop_policy() {
    let mut scopes = storage::Scopes::<4>::empty();
    scopes.grant(3, 8, 0, 42, 1).unwrap();
    let epoch = scopes.revision(3, 8).unwrap();
    assert_eq!(scopes.authorize(3, 8, epoch), Ok((0, 42)));
    assert_eq!(scopes.binding(0, 8), Err(abi::DENIED));
    scopes.revoke(3);
    assert_eq!(scopes.authorize(3, 8, epoch), Err(abi::DENIED));
    let mut launches = launches::Launches::<4>::default();
    let receipt = launches.record(3, 8, 123).unwrap();
    assert_eq!(launches.finished(123, wire::LAUNCH_EXITED), Some(3));
    assert_eq!(launches.query(3, 8, receipt)[1], wire::LAUNCH_EXITED);
    let mut management = management::Management::<4>::default();
    management.grant(3, 8, 2).unwrap();
    assert_eq!(management.target(3, 8), Ok(2));
    assert_eq!(management.target(0, 8), Err(abi::DENIED));
    // A headless world need not even reserve a visual child array.
    let mut no_display = World::<0>::empty();
    assert_eq!(
        no_display.request(0, 8, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0])[0],
        abi::DENIED
    );
}
