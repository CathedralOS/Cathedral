use super::*;
use cathedral_platform_contracts::input as keys;
fn world() -> World<2> {
    let mut w = World::empty();
    w.inherit(Grant {
        epoch: 1,
        width: 400,
        height: 200,
        keys: wire::VALID_KEY_MASK,
    });
    w.surface_capacity(8);
    for slot in 0..2 {
        w.place(
            slot,
            Rect {
                x: slot as u64 * 200,
                y: 0,
                width: 200,
                height: 200,
            },
            wire::VALID_KEY_MASK,
        )
        .unwrap();
        w.surface_limit(slot, 4).unwrap();
    }
    w
}
fn poll(w: &mut World<2>, slot: usize) -> wire::Packet {
    w.request(slot, 1, [wire::POLL, 0, 0, 0, 0, 0, 0, 0])
}
fn drain(w: &mut World<2>, slot: usize) {
    while wire::poll_event(poll(w, slot))[2] != wire::NONE {}
}

#[test]
fn hidden_presentation_keeps_selected_input_and_capture_without_new_hit_testing() {
    let mut w = world();
    let before = poll(&mut w, 0);
    w.pointer_stream(1, 20, 20, 1).unwrap();
    drain(&mut w, 0);
    w.take_notification(0);
    w.take_notification(1);
    w.visible(0, false).unwrap();
    assert!(w.take_notification(0));
    assert!(!w.take_notification(1));
    let hidden = poll(&mut w, 0);
    assert!(!wire::poll_frames(hidden));
    assert_eq!(wire::poll_view(hidden), wire::poll_view(before));
    assert_eq!(hidden[4] >> wire::SURFACE_BITS, 4);
    assert!(wire::poll_frames(poll(&mut w, 1)));
    w.key_stream(1, keys::A as u64, keys::PRESS as u64).unwrap();
    assert_eq!(wire::poll_event(poll(&mut w, 0))[2], wire::KEY);
    w.pointer_stream(1, 30, 20, 0).unwrap();
    assert_eq!(wire::poll_event(poll(&mut w, 0))[2], wire::POINTER);
    w.pointer_stream(1, 30, 20, 1).unwrap();
    // A fresh hit cannot enter a hidden presentation. The focus switch may
    // cancel old keys, but never delivers the new pointer press there.
    loop {
        let event = wire::poll_event(poll(&mut w, 0));
        assert_ne!(event[2], wire::POINTER);
        if event[2] == wire::NONE {
            break;
        }
    }
    w.visible(0, true).unwrap();
    assert!(wire::poll_frames(poll(&mut w, 0)));
}

#[test]
fn display_demand_composes_without_rebinding_or_forgetting_local_visibility() {
    let mut outer = world();
    let mut inner = World::<2>::empty();
    inner.inherit_poll(poll(&mut outer, 0));
    for slot in 0..2 {
        inner
            .place(
                slot,
                Rect {
                    x: slot as u64 * 100,
                    y: 0,
                    width: 100,
                    height: 100,
                },
                keys::KEY_MASK,
            )
            .unwrap();
    }
    let before = wire::poll_view(poll(&mut inner, 0));
    inner.visible(1, false).unwrap();
    outer.visible(0, false).unwrap();
    inner.inherit_poll(poll(&mut outer, 0));
    let hidden = poll(&mut inner, 0);
    assert_eq!(wire::poll_view(hidden), before);
    assert!(!wire::poll_frames(hidden));
    inner.visible(0, false).unwrap();
    inner.visible(0, true).unwrap();
    assert!(!wire::poll_frames(poll(&mut inner, 0))); // Child cannot raise ancestor demand.
    outer.visible(0, true).unwrap();
    inner.inherit_poll(poll(&mut outer, 0));
    assert!(wire::poll_frames(poll(&mut inner, 0)));
    assert!(!wire::poll_frames(poll(&mut inner, 1)));
    assert_eq!(wire::poll_view(poll(&mut inner, 0)), before);
    inner.inherit_poll(wire::error(abi::DENIED));
    assert!(!wire::poll_frames(poll(&mut inner, 0)));
}

#[test]
fn explicit_input_deactivation_is_independent_of_rendering_demand() {
    let mut w = world();
    w.pointer_stream(1, 20, 20, 1).unwrap();
    w.pointer_stream(1, 20, 20, 0).unwrap();
    drain(&mut w, 0);
    w.deactivate_input(0).unwrap();
    let reply = poll(&mut w, 0);
    assert!(wire::poll_frames(reply));
    assert_eq!(wire::poll_event(reply)[2], wire::CANCEL);
    w.key_stream(1, keys::A as u64, keys::PRESS as u64).unwrap();
    assert_eq!(wire::poll_event(poll(&mut w, 0))[2], wire::NONE);
    w.render_demand(false);
    assert!(!wire::poll_frames(poll(&mut w, 0)));
    assert!(!wire::poll_frames(poll(&mut w, 1)));
    assert_eq!(wire::NO_FRAMES & wire::VALID_KEY_MASK, 0);
    assert_eq!(wire::NO_FRAMES >> wire::SURFACE_BITS, 0);
}

#[test]
fn capture_frame_demand_does_not_reveal_or_rebind_hidden_content() {
    let mut w = world();
    let epoch = poll(&mut w, 0)[1];
    w.visible(0, false).unwrap();
    w.render_demand(false);
    w.take_notification(0);
    let ticket = w.request_frame(0, 1, epoch).unwrap();
    assert!(w.take_notification(0));
    let p = poll(&mut w, 0);
    assert_eq!(p[1], epoch);
    assert!(wire::poll_frames(p));
    assert_ne!(p[4] & wire::FRAME_REQUESTED, 0);
    assert!(!wire::poll_frames(poll(&mut w, 1)));
    assert_eq!(w.request_frame(0, 1, epoch), Err(abi::BUSY));
    w.pointer_stream(1, 20, 20, 1).unwrap();
    assert_eq!(wire::poll_event(poll(&mut w, 0))[2], wire::NONE);
    assert_eq!(
        w.request(0, 1, [wire::FRAME_DEMAND, epoch, 0, 0, 0, 0, 0, 0]),
        [0, epoch, ticket, 0, 0, 0, 0, 0]
    );
    assert_eq!(
        w.request(0, 1, [wire::FRAME_READY, epoch, ticket + 1, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    assert_eq!(
        w.request(0, 1, [wire::FRAME_READY, epoch, ticket, 0, 0, 0, 0, 0]),
        [0; 8]
    );
    assert!(w.frame_ready(0, 1, epoch, ticket));
    let p = poll(&mut w, 0);
    assert!(wire::poll_frames(p));
    assert_eq!(p[4] & wire::FRAME_REQUESTED, 0);
    // Capture demand persists until the provider pins/renders or cancels work.
    w.release_frame(0, 1, epoch, ticket).unwrap();
    assert!(!wire::poll_frames(poll(&mut w, 0)));
    let next = w.request_frame(0, 1, epoch).unwrap();
    assert!(next > ticket);
    assert_eq!(w.release_frame(0, 1, epoch, ticket), Err(abi::DENIED));
    assert!(!w.frame_ready(0, 1, epoch, next));
    assert_eq!(
        wire::FRAME_REQUESTED & (wire::VALID_KEY_MASK | wire::NO_FRAMES),
        0
    );
    assert_eq!(wire::FRAME_REQUESTED >> wire::SURFACE_BITS, 0);
}
#[test]
fn frame_requests_end_on_binding_replacement_and_cannot_acknowledge_a_new_child() {
    let mut w = world();
    let epoch = poll(&mut w, 0)[1];
    let old = w.request_frame(0, 1, epoch).unwrap();
    w.visible(0, false).unwrap();
    w.place(
        0,
        Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 100,
        },
        wire::VALID_KEY_MASK,
    )
    .unwrap();
    let changed = poll(&mut w, 0);
    assert_ne!(changed[1], epoch);
    assert!(!wire::poll_frames(changed));
    assert!(!w.frame_ready(0, 1, epoch, old));
    assert_eq!(w.release_frame(0, 1, epoch, old), Err(abi::DENIED));
    let next = w.request_frame(0, 1, changed[1]).unwrap();
    assert!(next > old);
    w.disconnected(0);
    assert!(!w.frame_ready(0, 1, changed[1], next));
    assert_eq!(w.request_frame(0, 1, changed[1]), Err(abi::DENIED));
    assert_eq!(w.request_frame(usize::MAX, 1, changed[1]), Err(abi::DENIED));
}

#[test]
fn changed_surface_allowance_invalidates_pending_frame_without_unhiding() {
    let mut w = world();
    let epoch = poll(&mut w, 0)[1];
    w.visible(0, false).unwrap();
    let ticket = w.request_frame(0, 1, epoch).unwrap();
    w.surface_capacity(7);
    assert_eq!(
        w.request(0, 1, [wire::FRAME_READY, epoch, ticket, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    assert!(!wire::poll_frames(poll(&mut w, 0)));
    w.surface_limit(0, 2).unwrap();
    let next = w.request_frame(0, 1, epoch).unwrap();
    w.surface_limit(0, 1).unwrap();
    assert!(!w.frame_ready(0, 1, epoch, next));
    assert!(!wire::poll_frames(poll(&mut w, 0)));
}
