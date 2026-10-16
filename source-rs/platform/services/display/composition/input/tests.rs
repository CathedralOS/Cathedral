use super::*;
use cathedral_platform_contracts::display::composition::{self as c, Node};
fn clients() -> [Client; 3] {
    core::array::from_fn(|index| {
        if index == 2 {
            return Client::empty();
        }
        let mut client = Client::empty();
        client.scope = Rect {
            x: 100 + index as u64 * 100,
            y: 100,
            w: 200,
            h: 200,
        };
        let bytes = c::encode(
            &[Node([c::ROOT, c::RECT, 0, 0, 200, 200, 0x123456])],
            &[0; c::IMAGE_PIXELS],
        )
        .unwrap();
        client
            .submit(index as u64 + 1, 1, &bytes, &mut Scene::empty())
            .unwrap();
        client
    })
}
use crate::composition::scene::Scene;
#[test]
fn hidden_scope_revokes_synthetic_lease_and_preserves_cancellation() {
    let mut clients = clients();
    let mut r = router(&clients);
    let token = r.lease(1, true, &clients).unwrap();
    r.drive(1, token, 40, 40, 1, &clients).unwrap();
    assert_eq!(r.next(1).kind, wire::BUTTONS);
    r.capture(1, 1, true).unwrap();
    clients[1].scope = Rect::default();
    assert_eq!(r.drive(1, token, 0, 0, 1, &clients), Err(abi::DENIED));
    r.scope_lost(1, &clients);
    r.scope_lost(1, &clients);
    assert_eq!(r.next(1).kind, wire::CANCEL);
    assert_eq!(r.next(1).kind, wire::NONE);
    assert_eq!(r.state(1)[4], 0);
    assert_eq!(r.state(1)[5], 0);
}
fn router(clients: &[Client; 3]) -> Router {
    let mut router = Router::new(1024, 768);
    router.enable(true, clients);
    router.device(0, 0, 0, false, clients);
    router
}
#[test]
fn overlap_capture_clamps_coordinates_and_releases() {
    let clients = clients();
    let mut r = router(&clients);
    r.device(110, 110, 1, false, &clients); // left only: world150,150
    assert_eq!(
        r.next(0),
        Event {
            pointer: 0,
            kind: wire::BUTTONS,
            x: 50,
            y: 50,
            buttons: 1
        }
    );
    assert_eq!(r.capture(1, 0, true), Err(abi::DENIED));
    r.capture(0, 0, true).unwrap();
    r.device(100, 0, 1, false, &clients); // overlap, still left
    assert_eq!(r.next(0).x, 150);
    assert_eq!(r.next(1).kind, wire::NONE);
    r.device(-200, 0, 1, false, &clients);
    assert_eq!(r.next(0).x, 0);
    r.device(200, 0, 0, false, &clients);
    assert_eq!(r.next(0).buttons, 0);
    assert_eq!(r.state(0)[4], 0);
    r.device(0, 0, 1, false, &clients);
    assert_eq!(r.next(1).x, 50);
    assert_eq!(r.next(0).kind, wire::NONE);
}
#[test]
fn hit_testing_obeys_nested_clips_and_actual_painted_content() {
    let mut clients = clients();
    let bytes = c::encode(
        &[
            Node([c::ROOT, c::GROUP, 10, 10, 20, 20, 0]),
            Node([0, c::GROUP, 10, 10, 40, 40, 0]),
            Node([1, c::RECT, 0, 0, 40, 40, 0xff0000]),
        ],
        &[0; c::IMAGE_PIXELS],
    )
    .unwrap();
    clients[1]
        .submit(2, 2, &bytes, &mut Scene::empty())
        .unwrap();
    let mut r = router(&clients);
    r.device(185, 85, 0, false, &clients); //225,125 in top's clipped 10x10
    assert_eq!(r.next(1).x, 25);
    r.device(10, 0, 0, false, &clients); //235,125 outside ancestor clip -> lower app
    assert_eq!(r.next(0).x, 135);
    assert_eq!(r.next(1).kind, wire::NONE);
}
#[test]
fn synthetic_grant_is_scoped_revocable_and_independent() {
    let mut clients = clients();
    let mut r = router(&clients);
    let epoch = r.lease(1, true, &clients).unwrap();
    assert_eq!(r.drive(0, epoch, 0, 0, 0, &clients), Err(abi::DENIED));
    r.drive(1, epoch, u64::MAX, u64::MAX, 1, &clients).unwrap();
    assert_eq!(
        r.next(1),
        Event {
            pointer: 1,
            kind: wire::BUTTONS,
            x: 199,
            y: 199,
            buttons: 1
        }
    );
    r.capture(1, 1, true).unwrap();
    r.device(110, 110, 1, false, &clients);
    r.capture(0, 0, true).unwrap();
    assert_eq!(r.state(0)[4], 1);
    assert_eq!(r.state(1)[4], 2);
    r.source_lost(&clients);
    assert_eq!(r.state(1)[4], 2);
    r.client_lost(1, &clients);
    clients[1].incarnation = 99;
    assert_eq!(r.drive(1, epoch, 0, 0, 0, &clients), Err(abi::DENIED));
    let replacement = r.lease(1, true, &clients).unwrap();
    assert_ne!(epoch, replacement);
    assert_eq!(r.drive(1, epoch, 0, 0, 0, &clients), Err(abi::DENIED));
}
#[test]
fn death_and_source_loss_cancel_instead_of_retargeting_a_held_press() {
    let mut clients = clients();
    let mut r = router(&clients);
    r.device(110, 110, 1, false, &clients);
    r.next(0);
    r.capture(0, 0, true).unwrap();
    r.source_lost(&clients);
    assert_eq!(r.next(0).kind, wire::CANCEL);
    r.device(100, 0, 1, false, &clients);
    assert_eq!(r.next(1).kind, wire::NONE);
    r.device(0, 0, 0, false, &clients);
    r.device(0, 0, 1, false, &clients);
    assert_eq!(r.next(1).kind, wire::BUTTONS);
    r.client_lost(1, &clients);
    clients[1].disconnect(2);
    r.device(0, 0, 1, false, &clients);
    assert_eq!(r.next(0).kind, wire::NONE);
    r.device(0, 0, 0, false, &clients);
    r.device(0, 0, 1, false, &clients);
    assert_eq!(r.next(0).kind, wire::BUTTONS);
}
#[test]
fn overflow_cancels_edges_but_preserves_other_pointer_progress() {
    let clients = clients();
    let mut r = router(&clients);
    let lease = r.lease(1, true, &clients).unwrap();
    r.drive(1, lease, 150, 50, 1, &clients).unwrap();
    r.capture(1, 1, true).unwrap();
    r.device(110, 110, 1, false, &clients);
    r.capture(0, 0, true).unwrap();
    for index in 0..8 {
        r.device(0, 0, if index % 2 == 0 { 3 } else { 1 }, false, &clients);
    }
    assert_eq!(r.state(0)[4], 0);
    assert_eq!(r.next(0).kind, wire::CANCEL);
    assert_eq!(r.state(1)[4], 2);
    assert_eq!(r.next(1).pointer, 1);
    r.device(0, 0, 1, false, &clients);
    assert_eq!(r.next(0).kind, wire::CANCEL);
    assert_eq!(r.next(0).kind, wire::NONE);
    r.device(0, 0, 0, false, &clients);
    r.device(0, 0, 1, false, &clients);
    assert_eq!(r.next(0).kind, wire::BUTTONS);
}
#[test]
fn motion_coalesces_and_cursor_is_clipped_to_lease() {
    let clients = clients();
    let mut r = router(&clients);
    for index in 0..100 {
        r.device(
            if index == 0 { 110 } else { 1 },
            if index == 0 { 110 } else { 0 },
            0,
            false,
            &clients,
        );
    }
    assert_eq!(r.queues[0].len, 1);
    assert_eq!(r.queues[1].len, 1);
    let epoch = r.lease(1, true, &clients).unwrap();
    r.drive(1, epoch, 199, 199, 0, &clients).unwrap();
    assert_eq!(r.pixel(399, 299, &clients), Some(0xffd166));
    assert_eq!(r.pixel(400, 300, &clients), None);
}

#[test]
fn revoked_queued_release_becomes_cancel_before_a_new_lease() {
    let clients = clients();
    let mut r = router(&clients);
    let old = r.lease(1, true, &clients).unwrap();
    r.drive(1, old, 50, 50, 1, &clients).unwrap();
    assert_eq!(r.next(1).buttons, 1); // App observed the press.
    r.drive(1, old, 50, 50, 0, &clients).unwrap(); // Release is still queued.
    r.lease(1, false, &clients).unwrap();
    let new = r.lease(1, true, &clients).unwrap();
    r.drive(1, new, 70, 70, 1, &clients).unwrap();
    assert_eq!(r.next(1).kind, wire::CANCEL);
    assert_eq!(
        r.next(1),
        Event {
            pointer: 1,
            kind: wire::BUTTONS,
            x: 70,
            y: 70,
            buttons: 1
        }
    );
    assert_eq!(r.next(1).kind, wire::NONE);
}

#[test]
fn synthetic_pointer_cannot_draw_or_target_over_a_covering_app() {
    let clients = clients();
    let mut r = router(&clients);
    let lease = r.lease(0, true, &clients).unwrap();
    r.drive(0, lease, 150, 50, 1, &clients).unwrap(); // World250,150: covered by app two.
    assert_eq!(r.next(0).kind, wire::NONE);
    assert_eq!(r.next(1).kind, wire::NONE);
    assert_eq!(r.pixel(250, 150, &clients), None);
    assert_eq!(r.capture(0, 1, true), Err(abi::DENIED));
}
