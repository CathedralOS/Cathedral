use super::*;
use crate::scene::{Rect, Scene};
use cathedral_contracts::composition::{self as c, Node};
fn world() -> ([Client; 3], Tree, Focus) {
    let mut clients = core::array::from_fn(|i| {
        let mut c = Client::empty();
        let bytes = c::encode(
            &[Node([c::ROOT, c::RECT, 0, 0, 256, 256, 0])],
            &[0; c::IMAGE_PIXELS],
        )
        .unwrap();
        c.submit(i as u64 + 10, 1, &bytes, &mut Scene::empty())
            .unwrap();
        c
    });
    let mut tree = Tree::default();
    for i in 0..2 {
        tree.place_root(
            i,
            Rect {
                x: i as u64 * 300,
                y: 50,
                w: 256,
                h: 200,
            },
        )
        .unwrap();
    }
    let grant = tree.grant(1, 2, &clients).unwrap();
    tree.place(
        1,
        grant,
        Rect {
            x: 100,
            y: 80,
            w: 200,
            h: 160,
        },
        &clients,
    )
    .unwrap();
    tree.resolve(&mut clients);
    (clients, tree, Focus::default())
}
#[test]
fn focus_attenuates_keys_delegation_and_subtree_authority() {
    let (clients, tree, mut f) = world();
    let left = f
        .root_grant(0, wire::KEY_MASK, false, &tree, &clients)
        .unwrap();
    let host = f.root_grant(1, 0x19, true, &tree, &clients).unwrap(); // Up,Right,Enter
    assert_eq!(
        f.delegate((0, left), 2, 0x10, false, &tree, &clients),
        Err(abi::DENIED)
    );
    assert_eq!(
        f.delegate((1, host), 2, wire::KEY_MASK, false, &tree, &clients),
        Err(abi::DENIED)
    );
    let child = f
        .delegate((1, host), 2, 0x10, false, &tree, &clients)
        .unwrap();
    f.click(Some(0), &tree, &clients);
    assert_eq!(f.select(1, host, 2, &tree, &clients), Err(abi::DENIED));
    f.click(Some(1), &tree, &clients);
    assert_eq!(f.select(2, child, 2, &tree, &clients), Err(abi::DENIED));
    f.select(1, host, 2, &tree, &clients).unwrap();
    assert_eq!(f.select(2, child, 0, &tree, &clients), Err(abi::DENIED));
    f.key(
        input::Event {
            key: input::UP,
            state: input::PRESS,
        },
        &tree,
        &clients,
    );
    assert_eq!(f.next(2), None);
    f.key(
        input::Event {
            key: input::ENTER,
            state: input::PRESS,
        },
        &tree,
        &clients,
    );
    assert_eq!(
        f.next(2),
        Some(wire::key_event(input::ENTER, input::PRESS, child))
    );
    f.revoke(1, host, &tree, &clients).unwrap();
    assert_eq!(f.state()[1], 0);
    // Hosting revocation and layout invalidation can both visit the same branch
    // before a sleeping child gets its cancellation reply.
    f.revoke_branch(2, &tree);
    assert_eq!(f.next(2), Some(wire::focus_lost(child)));
    assert_eq!(f.next(2), None);
    assert_eq!(f.select(2, child, 2, &tree, &clients), Err(abi::DENIED));
}
#[test]
fn focus_change_cancels_old_queue_and_never_retargets_held_keys() {
    let (clients, tree, mut f) = world();
    f.root_grant(0, wire::KEY_MASK, false, &tree, &clients)
        .unwrap();
    let right = f
        .root_grant(1, wire::KEY_MASK, true, &tree, &clients)
        .unwrap();
    f.click(Some(0), &tree, &clients);
    f.key(
        input::Event {
            key: input::ENTER,
            state: input::PRESS,
        },
        &tree,
        &clients,
    );
    f.click(Some(1), &tree, &clients);
    assert_eq!(f.next(0).unwrap()[2], wire::FOCUS_LOST);
    assert_eq!(f.next(0), None);
    f.key(
        input::Event {
            key: input::ENTER,
            state: input::REPEAT,
        },
        &tree,
        &clients,
    );
    f.key(
        input::Event {
            key: input::ENTER,
            state: input::RELEASE,
        },
        &tree,
        &clients,
    );
    assert_eq!(f.next(1), None);
    f.key(
        input::Event {
            key: input::ENTER,
            state: input::PRESS,
        },
        &tree,
        &clients,
    );
    assert_eq!(
        f.next(1),
        Some(wire::key_event(input::ENTER, input::PRESS, right))
    );
}
#[test]
fn host_incarnation_and_placement_revoke_descendant_authority() {
    let (mut clients, mut tree, mut f) = world();
    assert_eq!(
        clients[2].scope,
        Rect {
            x: 400,
            y: 130,
            w: 156,
            h: 120
        }
    );
    let host = f
        .root_grant(1, wire::KEY_MASK, true, &tree, &clients)
        .unwrap();
    let child = f
        .delegate((1, host), 2, 0x10, false, &tree, &clients)
        .unwrap();
    f.click(Some(2), &tree, &clients);
    tree.lost(1);
    f.revoke_branch(1, &tree);
    clients[1].incarnation = 99;
    tree.resolve(&mut clients);
    assert!(!clients[2].scope.valid());
    assert_eq!(f.select(2, child, 2, &tree, &clients), Err(abi::DENIED));
    assert_eq!(
        f.delegate((1, host), 2, 0x10, false, &tree, &clients),
        Err(abi::DENIED)
    );
    let new = tree.grant(1, 2, &clients).unwrap();
    assert_eq!(
        tree.place(
            0,
            new,
            Rect {
                x: 0,
                y: 0,
                w: 20,
                h: 20
            },
            &clients
        ),
        Err(abi::DENIED)
    );
    assert_eq!(
        tree.place(
            1,
            new,
            Rect {
                x: u64::MAX,
                y: 0,
                w: 20,
                h: 20
            },
            &clients
        ),
        Err(abi::INVALID_ARGUMENT)
    );
}
#[test]
fn explicit_regions_target_empty_pixels_and_exclude_decoration() {
    let mut scene = Scene::empty();
    let bytes = c::encode(
        &[
            Node([c::ROOT, c::RECT, 0, 0, 100, 100, 0]),
            Node([c::ROOT, c::GROUP, 10, 10, 20, 20, 0]),
            Node([1, c::INPUT, 5, 5, 40, 40, 0]),
            Node([c::ROOT, c::INPUT, 150, 150, 20, 20, 0]),
        ],
        &[0; c::IMAGE_PIXELS],
    )
    .unwrap();
    scene.decode(&bytes).unwrap();
    assert!(!scene.hit(0, 0));
    assert!(scene.hit(15, 15));
    assert!(!scene.hit(30, 30));
    assert!(scene.hit(155, 155));
    assert_eq!(scene.pixel(155, 155), None);
}
#[test]
fn focus_overflow_and_source_loss_emit_cancel_without_stuck_repeats() {
    let (clients, tree, mut f) = world();
    f.root_grant(0, wire::KEY_MASK, false, &tree, &clients)
        .unwrap();
    f.click(Some(0), &tree, &clients);
    for _ in 0..9 {
        f.key(
            input::Event {
                key: input::ENTER,
                state: input::PRESS,
            },
            &tree,
            &clients,
        );
    }
    assert_eq!(f.next(0).unwrap()[2], wire::FOCUS_LOST);
    assert_eq!(f.next(0), None);
    f.key(
        input::Event {
            key: input::ENTER,
            state: input::REPEAT,
        },
        &tree,
        &clients,
    );
    assert_eq!(f.next(0), None);
    f.key(
        input::Event {
            key: input::ENTER,
            state: input::RELEASE,
        },
        &tree,
        &clients,
    );
    f.key(
        input::Event {
            key: input::ENTER,
            state: input::PRESS,
        },
        &tree,
        &clients,
    );
    assert!(f.next(0).is_some());
    f.source_lost();
    assert_eq!(f.next(0).unwrap()[2], wire::FOCUS_LOST);
}
