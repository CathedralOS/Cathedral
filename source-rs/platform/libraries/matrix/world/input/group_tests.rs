use super::tests::{click, drain, forward, next, world};
use super::*;
use cathedral_platform_contracts::input as keys;

#[test]
fn keyboard_and_pointers_share_selection_without_sharing_capture() {
    let mut w = world();
    for name in [19, 23, 73] {
        w.bind_input_stream(name, 99).unwrap();
    }
    w.bind_input_stream(127, 777).unwrap();
    click(&mut w, 127, 220);
    w.pointer_stream(19, 20, 20, 1).unwrap(); // Keep the first pointer captured.
    w.key_stream(23, keys::A as u64, keys::PRESS as u64)
        .unwrap();
    let first = drain(&mut w, 0);
    assert_eq!(first.len(), 2);
    assert_eq!(first[0][7], first[1][7]); // Distinct channels in the same local group.
    assert_ne!(first[0][6], first[1][6]);
    drain(&mut w, 1);

    click(&mut w, 73, 220); // Shared keyboard selection moves, capture does not.
    w.pointer_stream(19, 250, 30, 1).unwrap();
    let old = drain(&mut w, 0);
    assert!(old.iter().any(|e| e[2] == wire::CANCEL_KEYS));
    assert!(!old.iter().any(|e| e[2] == wire::CANCEL));
    assert!(old.iter().any(|e| e[2..6] == [wire::POINTER, 199, 30, 1]));
    w.key_stream(23, keys::A as u64, keys::REPEAT as u64)
        .unwrap();
    w.key_stream(23, keys::A as u64, keys::RELEASE as u64)
        .unwrap();
    assert!(drain(&mut w, 1).iter().all(|e| e[2] != wire::KEY));
    w.key_stream(23, keys::A as u64, keys::PRESS as u64)
        .unwrap();
    assert_eq!(next(&mut w, 1)[2], wire::KEY);
    w.pointer_stream(19, 250, 30, 0).unwrap();
    assert_eq!(&next(&mut w, 0)[2..6], &[wire::POINTER, 199, 30, 0]);
    w.select_input_group(99, Some(0)).unwrap();
    drain(&mut w, 1);
    w.key_stream(127, keys::Z as u64, keys::PRESS as u64)
        .unwrap();
    assert_eq!(next(&mut w, 1)[3], keys::Z as u64); // Other group kept its selection.
}

#[test]
fn nested_group_delivery_preserves_click_before_key_across_streams() {
    let mut outer = world();
    for name in [19, 23, 73] {
        outer.bind_input_stream(name, 99).unwrap();
    }
    let mut inner = world();
    forward(&mut outer, &mut inner);
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
    // Prime queue slot order so the selecting pointer's slot comes after the
    // keyboard's. A round-robin stream drain alone would deliver the key first.
    click(&mut outer, 19, 20);
    outer
        .key_stream(23, keys::A as u64, keys::PRESS as u64)
        .unwrap();
    outer
        .key_stream(23, keys::A as u64, keys::RELEASE as u64)
        .unwrap();
    forward(&mut outer, &mut inner);
    drain(&mut inner, 0);
    drain(&mut inner, 1);
    click(&mut outer, 73, 120);
    outer
        .key_stream(23, keys::Z as u64, keys::PRESS as u64)
        .unwrap();
    forward(&mut outer, &mut inner);
    assert!(drain(&mut inner, 0).iter().all(|e| e[2] != wire::KEY));
    assert!(
        drain(&mut inner, 1)
            .iter()
            .any(|e| e[2..5] == [wire::KEY, keys::Z as u64, keys::PRESS as u64])
    );
}

#[test]
fn shared_group_motion_cannot_coalesce_across_another_channels_key() {
    let mut w = world();
    w.bind_input_stream(19, 99).unwrap();
    w.bind_input_stream(23, 99).unwrap();
    click(&mut w, 19, 20);
    drain(&mut w, 0);
    w.pointer_stream(19, 30, 20, 0).unwrap();
    w.pointer_stream(19, 40, 20, 0).unwrap();
    w.key_stream(23, keys::A as u64, keys::PRESS as u64)
        .unwrap();
    w.pointer_stream(19, 50, 20, 0).unwrap();
    let events = drain(&mut w, 0);
    assert_eq!(
        events
            .iter()
            .map(|e| (e[2], e[3]))
            .collect::<std::vec::Vec<_>>(),
        [
            (wire::POINTER, 30),
            (wire::POINTER, 40),
            (wire::KEY, keys::A as u64),
            (wire::POINTER, 50)
        ]
    );
}

#[test]
fn keyboard_cancellation_keeps_full_pointer_queue_and_filters_keys_after_cancel() {
    let mut w = world();
    for i in 0..16 {
        w.pointer_stream(19, 20, 20, (i % 2 == 0) as u64).unwrap();
    }
    w.select_input_group(19, Some(1)).unwrap();
    let events = drain(&mut w, 0);
    assert_eq!(events.len(), 17); // Reserved boundary cannot discard sixteen edges.
    assert!(events[..16].iter().all(|e| e[2] == wire::POINTER));
    assert_eq!(events[16][2], wire::CANCEL_KEYS);
    click(&mut w, 19, 20);
    drain(&mut w, 0);
    w.cancel_stream(19).unwrap();
    w.key_stream(19, keys::A as u64, keys::PRESS as u64)
        .unwrap();
    w.select_input_group(19, Some(1)).unwrap();
    let events = drain(&mut w, 0);
    assert_eq!(
        events.iter().map(|e| e[2]).collect::<std::vec::Vec<_>>(),
        [wire::CANCEL, wire::CANCEL_KEYS]
    );
}

#[test]
fn associations_are_explicit_bounded_and_do_not_silently_rebind() {
    let mut w = world();
    assert_eq!(w.bind_input_stream(0, 99), Err(abi::INVALID_ARGUMENT));
    assert_eq!(w.bind_input_stream(19, 0), Err(abi::INVALID_ARGUMENT));
    assert_eq!(w.select_input_group(99, Some(0)), Err(abi::DENIED));
    for id in 1..=wire::INPUT_STREAMS as u32 {
        w.bind_input_stream(id, 99).unwrap();
    }
    assert_eq!(w.bind_input_stream(5, 99), Err(abi::NO_MEMORY));
    assert_eq!(w.bind_input_stream(1, 100), Err(abi::BUSY));
    w.bind_input_stream(1, 99).unwrap();
    w.select_input_group(99, Some(0)).unwrap();
    assert_eq!(w.select_input_group(99, Some(2)), Err(abi::DENIED));
    w.key_stream(2, keys::A as u64, keys::PRESS as u64).unwrap();
    assert_eq!(next(&mut w, 0)[2], wire::KEY);
}

#[test]
fn ancestor_selection_can_change_while_a_nested_drag_continues() {
    let mut outer = world();
    for name in [19, 23, 73] {
        outer.bind_input_stream(name, 99).unwrap();
    }
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
    outer.pointer_stream(19, 20, 20, 1).unwrap();
    outer
        .key_stream(23, keys::A as u64, keys::PRESS as u64)
        .unwrap();
    forward(&mut outer, &mut inner);
    drain(&mut inner, 0);

    click(&mut outer, 73, 220); // Select another outer child in the same group.
    forward(&mut outer, &mut inner);
    let cancelled = drain(&mut inner, 0);
    assert!(cancelled.iter().any(|e| e[2] == wire::CANCEL_KEYS));
    assert!(!cancelled.iter().any(|e| e[2] == wire::CANCEL));
    outer.pointer_stream(19, 350, 30, 1).unwrap();
    forward(&mut outer, &mut inner);
    assert_eq!(&next(&mut inner, 0)[2..6], &[wire::POINTER, 99, 30, 1]);

    outer
        .key_stream(23, keys::A as u64, keys::RELEASE as u64)
        .unwrap();
    outer.select_input_group(99, Some(0)).unwrap();
    outer
        .key_stream(23, keys::A as u64, keys::PRESS as u64)
        .unwrap();
    forward(&mut outer, &mut inner);
    assert_eq!(next(&mut inner, 0)[2], wire::KEY); // Remembered inner selection.
    outer.pointer_stream(19, 350, 30, 0).unwrap();
    forward(&mut outer, &mut inner);
    assert_eq!(&next(&mut inner, 0)[2..6], &[wire::POINTER, 99, 30, 0]);
}
