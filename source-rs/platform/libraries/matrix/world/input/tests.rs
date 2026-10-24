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
    }
    w
}
fn click(w: &mut World<2>, channel: u32, x: u64) {
    w.pointer_stream(channel, x, 20, 1).unwrap();
    w.pointer_stream(channel, x, 20, 0).unwrap();
}
fn next(w: &mut World<2>, slot: usize) -> wire::Packet {
    wire::poll_event(w.request(slot, 1, [wire::POLL, 0, 0, 0, 0, 0, 0, 0]))
}
fn drain(w: &mut World<2>, slot: usize) -> std::vec::Vec<wire::Packet> {
    let mut events = std::vec::Vec::new();
    loop {
        let event = next(w, slot);
        if event[2] == wire::NONE {
            return events;
        }
        events.push(event);
    }
}
#[test]
fn separate_sources_select_and_type_without_stealing_focus() {
    let mut w = world();
    click(&mut w, 19, 20);
    click(&mut w, 73, 220);
    drain(&mut w, 0);
    drain(&mut w, 1);
    w.key_stream(19, keys::A as u64, keys::PRESS as u64)
        .unwrap();
    w.key_stream(73, keys::Z as u64, keys::PRESS as u64)
        .unwrap();
    let a = next(&mut w, 0);
    let b = next(&mut w, 1);
    assert_eq!(&a[2..5], &[wire::KEY, keys::A as u64, keys::PRESS as u64]);
    assert_eq!(&b[2..5], &[wire::KEY, keys::Z as u64, keys::PRESS as u64]);
    assert_eq!(a[6], 1); // Child-local names, not ancestor IDs 19 / 73.
    assert_eq!(b[6], 1);
    assert!(drain(&mut w, 0).is_empty());
    assert!(drain(&mut w, 1).is_empty());
}
#[test]
fn cancel_one_stream_preserves_selection_and_other_streams_hold() {
    let mut w = world();
    click(&mut w, 19, 20);
    click(&mut w, 73, 20);
    drain(&mut w, 0);
    for channel in [19, 73] {
        w.key_stream(channel, keys::A as u64, keys::PRESS as u64)
            .unwrap();
    }
    w.cancel_stream(19).unwrap();
    let events = drain(&mut w, 0);
    assert!(events.iter().any(|e| e[2] == wire::CANCEL && e[6] == 1));
    assert!(events.iter().any(|e| e[2] == wire::KEY && e[6] == 2));
    w.key_stream(19, keys::A as u64, keys::REPEAT as u64)
        .unwrap();
    assert!(drain(&mut w, 0).is_empty());
    w.key_stream(73, keys::A as u64, keys::RELEASE as u64)
        .unwrap();
    assert_eq!(next(&mut w, 0)[6], 2);
    w.key_stream(19, keys::A as u64, keys::RELEASE as u64)
        .unwrap();
    w.key_stream(19, keys::Z as u64, keys::PRESS as u64)
        .unwrap();
    assert_eq!(next(&mut w, 0)[3], keys::Z as u64); // No second selecting click.
}
#[test]
fn overflow_and_motion_coalescing_do_not_cross_streams() {
    let mut w = world();
    click(&mut w, 19, 20);
    click(&mut w, 73, 20);
    drain(&mut w, 0);
    w.key_stream(73, keys::Z as u64, keys::PRESS as u64)
        .unwrap();
    for _ in 0..9 {
        w.key_stream(19, keys::A as u64, keys::PRESS as u64)
            .unwrap();
        w.key_stream(19, keys::A as u64, keys::RELEASE as u64)
            .unwrap();
    }
    let events = drain(&mut w, 0);
    assert!(events.iter().any(|e| e[2] == wire::CANCEL && e[6] == 1));
    assert!(
        events
            .iter()
            .any(|e| e[2] == wire::KEY && e[3] == keys::Z as u64 && e[6] == 2)
    );
    for x in 30..100 {
        w.pointer_stream(19, x, 20, 0).unwrap();
        w.pointer_stream(73, 190 - x, 20, 0).unwrap();
    }
    let events = drain(&mut w, 0);
    assert_eq!(events.len(), 4);
    assert!(events.iter().any(|e| e[3] == 99 && e[6] == 1));
    assert!(events.iter().any(|e| e[3] == 91 && e[6] == 2));
}
#[test]
fn stream_capacity_and_child_injection_are_bounded() {
    let mut w = world();
    assert_eq!(w.pointer_stream(0, 20, 20, 0), Err(abi::INVALID_ARGUMENT));
    for channel in 1..=wire::INPUT_STREAMS as u32 {
        w.pointer_stream(channel, 20, 20, 0).unwrap();
    }
    assert_eq!(w.pointer_stream(99, 20, 20, 0), Err(abi::NO_MEMORY));
    assert_eq!(
        w.request(0, 1, [wire::POINTER_SOURCE, 20, 20, 1, 0, 0, 0, 0])[0],
        abi::DENIED
    );
    assert_eq!(
        w.request(
            0,
            1,
            [
                wire::KEY_SOURCE,
                keys::A as u64,
                keys::PRESS as u64,
                0,
                0,
                0,
                0,
                0
            ]
        )[0],
        abi::DENIED
    );
}
#[test]
fn overflow_interrupts_same_source_in_other_children_only() {
    let mut w = world();
    click(&mut w, 19, 20);
    click(&mut w, 73, 220);
    drain(&mut w, 0);
    drain(&mut w, 1);
    w.key_stream(19, keys::A as u64, keys::PRESS as u64)
        .unwrap();
    assert_eq!(next(&mut w, 0)[2], wire::KEY);
    // Source 19 hovers child 1 while its keyboard selection stays on child 0.
    // Keep source 73's independent key pending in that hovered child.
    w.pointer_stream(19, 220, 20, 0).unwrap();
    w.key_stream(73, keys::Z as u64, keys::PRESS as u64)
        .unwrap();
    // Fill the focused child's queue with non-coalescible key transitions.
    for _ in 0..9 {
        w.key_stream(19, keys::Z as u64, keys::PRESS as u64)
            .unwrap();
        w.key_stream(19, keys::Z as u64, keys::RELEASE as u64)
            .unwrap();
    }
    let focus = drain(&mut w, 0);
    let hover = drain(&mut w, 1);
    assert!(focus.iter().any(|e| e[2] == wire::CANCEL));
    assert!(hover.iter().any(|e| e[2] == wire::CANCEL && e[6] == 2));
    assert!(hover.iter().any(|e| e[2] == wire::KEY && e[6] == 1));
    w.key_stream(19, keys::A as u64, keys::REPEAT as u64)
        .unwrap();
    assert!(drain(&mut w, 0).is_empty());
    w.key_stream(73, keys::Z as u64, keys::RELEASE as u64)
        .unwrap();
    assert_eq!(next(&mut w, 1)[4], keys::RELEASE as u64);
}
#[test]
fn nested_forwarding_preserves_streams_and_remembered_selection() {
    let mut outer = world();
    let mut inner = World::<2>::empty();
    let view = outer.request(0, 1, [wire::POLL, 0, 0, 0, 0, 0, 0, 0]);
    inner.inherit_poll(view);
    for slot in 0..2 {
        inner
            .place(
                slot,
                Rect {
                    x: slot as u64 * 100,
                    y: 0,
                    width: 100,
                    height: 200,
                },
                wire::VALID_KEY_MASK,
            )
            .unwrap();
    }
    click(&mut outer, 19, 20);
    click(&mut outer, 73, 120);
    forward(&mut outer, &mut inner);
    drain(&mut inner, 0);
    drain(&mut inner, 1);
    outer.cancel_stream(19).unwrap();
    forward(&mut outer, &mut inner);
    drain(&mut inner, 0);
    drain(&mut inner, 1);
    outer
        .key_stream(19, keys::A as u64, keys::PRESS as u64)
        .unwrap();
    outer
        .key_stream(73, keys::Z as u64, keys::PRESS as u64)
        .unwrap();
    forward(&mut outer, &mut inner);
    assert_eq!(next(&mut inner, 0)[3], keys::A as u64);
    assert_eq!(next(&mut inner, 1)[3], keys::Z as u64);
}
fn forward(outer: &mut World<2>, inner: &mut World<2>) {
    loop {
        let reply = outer.request(0, 1, [wire::POLL, 0, 0, 0, 0, 0, 0, 0]);
        assert!(inner.inherit_poll(reply).valid());
        if reply[5] & wire::MORE == 0 {
            break;
        }
    }
}

#[test]
fn upstream_cancellation_does_not_wait_for_releases_the_sender_suppresses() {
    let mut outer = world();
    let mut inner = world();
    forward(&mut outer, &mut inner);
    inner
        .place(
            0,
            Rect {
                x: 0,
                y: 0,
                width: 100,
                height: 200,
            },
            wire::VALID_KEY_MASK,
        )
        .unwrap();
    click(&mut outer, 19, 20);
    forward(&mut outer, &mut inner);
    drain(&mut inner, 0);
    outer
        .key_stream(19, keys::A as u64, keys::PRESS as u64)
        .unwrap();
    outer.pointer_stream(19, 20, 20, 1).unwrap();
    forward(&mut outer, &mut inner);
    drain(&mut inner, 0);

    outer.cancel_stream(19).unwrap();
    forward(&mut outer, &mut inner);
    assert_eq!(next(&mut inner, 0)[2], wire::CANCEL);
    outer
        .key_stream(19, keys::A as u64, keys::RELEASE as u64)
        .unwrap();
    outer.pointer_stream(19, 20, 20, 0).unwrap();
    assert!(drain(&mut outer, 0).is_empty());

    // Both are fresh transitions. The intermediate router must not suppress
    // them because the preceding releases were intentionally never forwarded.
    outer
        .key_stream(19, keys::A as u64, keys::PRESS as u64)
        .unwrap();
    outer.pointer_stream(19, 20, 20, 1).unwrap();
    forward(&mut outer, &mut inner);
    let events = drain(&mut inner, 0);
    assert_eq!(events.len(), 2);
    assert_eq!(
        &events[0][2..5],
        &[wire::KEY, keys::A as u64, keys::PRESS as u64]
    );
    assert_eq!(&events[1][2..6], &[wire::POINTER, 20, 20, 1]);
}

#[test]
fn busy_stream_cannot_starve_another_pending_stream() {
    let mut w = world();
    click(&mut w, 19, 20);
    click(&mut w, 73, 20);
    drain(&mut w, 0);
    w.pointer_stream(19, 30, 20, 0).unwrap();
    w.key_stream(73, keys::A as u64, keys::PRESS as u64)
        .unwrap();
    let mut delivered = false;
    for _ in 0..2 {
        delivered |= next(&mut w, 0)[6] == 2;
        w.pointer_stream(19, 40, 20, 0).unwrap();
    }
    assert!(delivered);
}

#[test]
fn removing_capture_cannot_consume_the_next_fresh_click() {
    let mut w = world();
    w.pointer_stream(19, 20, 20, 1).unwrap();
    w.pointer_stream(73, 220, 20, 1).unwrap();
    drain(&mut w, 0);
    drain(&mut w, 1);
    w.revoke(0).unwrap();
    w.place(
        0,
        Rect {
            x: 0,
            y: 0,
            width: 200,
            height: 200,
        },
        wire::VALID_KEY_MASK,
    )
    .unwrap();
    w.pointer_stream(19, 220, 20, 1).unwrap(); // Old hold cannot select a sibling.
    assert!(drain(&mut w, 1).is_empty());
    w.pointer_stream(19, 220, 20, 0).unwrap(); // Ends suppression, not a new event.
    assert!(drain(&mut w, 1).is_empty());
    w.pointer_stream(19, 20, 20, 1).unwrap();
    assert_eq!(&next(&mut w, 0)[2..6], &[wire::POINTER, 20, 20, 1]);
    w.key_stream(19, keys::A as u64, keys::PRESS as u64)
        .unwrap();
    assert_eq!(next(&mut w, 0)[2], wire::KEY);
    w.pointer_stream(73, 20, 20, 0).unwrap(); // Other stream's capture stayed intact.
    assert_eq!(&next(&mut w, 1)[2..6], &[wire::POINTER, 0, 20, 0]);
}
