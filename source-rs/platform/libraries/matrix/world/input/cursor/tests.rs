use super::*;
fn setup() -> World<2> {
    let mut w = World::empty();
    w.inherit(Grant {
        epoch: 1,
        width: 120,
        height: 80,
        keys: wire::VALID_KEY_MASK,
    });
    for slot in 0..2 {
        w.place(
            slot,
            Rect {
                x: slot as u64 * 60,
                y: 0,
                width: 50,
                height: 80,
            },
            wire::VALID_KEY_MASK,
        )
        .unwrap();
        w.request(slot, 7 + slot as u64, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
    }
    w
}
fn epoch(w: &mut World<2>, slot: usize) -> u64 {
    w.request(slot, 7 + slot as u64, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0])[1]
}
fn preference(w: &mut World<2>, slot: usize, channel: u64, mode: u64) -> u64 {
    let e = epoch(w, slot);
    w.request(
        slot,
        7 + slot as u64,
        [wire::CURSOR_PREFERENCE, e, channel, mode, 0, 0, 0, 0],
    )[0]
}
#[test]
fn hover_and_capture_not_keyboard_focus_select_each_pointer_preference() {
    let mut w = setup();
    w.bind_input_stream(1, 1).unwrap();
    w.bind_input_stream(2, 2).unwrap();
    w.pointer_stream(1, 10, 10, 0).unwrap();
    w.pointer_stream(2, 70, 10, 0).unwrap();
    assert_eq!(preference(&mut w, 0, 1, wire::CURSOR_HIDDEN), 0);
    assert_eq!(w.cursor(0).unwrap().mode, wire::CURSOR_HIDDEN);
    assert_eq!(w.cursor(1).unwrap().mode, wire::CURSOR_DEFAULT);
    w.select_input_group(1, Some(1)).unwrap();
    assert_eq!(w.cursor(0).unwrap().mode, wire::CURSOR_HIDDEN);
    w.pointer_stream(1, 10, 10, 1).unwrap();
    w.pointer_stream(1, 70, 10, 1).unwrap();
    assert_eq!(w.cursor(0).unwrap().mode, wire::CURSOR_HIDDEN);
    w.pointer_stream(1, 70, 10, 0).unwrap();
    assert_eq!(w.cursor(0).unwrap().mode, wire::CURSOR_DEFAULT);
    // A late preference for the former receiver cannot change the current target.
    assert_eq!(preference(&mut w, 0, 1, wire::CURSOR_HIDDEN), 0);
    assert_eq!(w.cursor(0).unwrap().mode, wire::CURSOR_DEFAULT);
    assert_eq!(preference(&mut w, 0, 2, wire::CURSOR_HIDDEN), abi::DENIED);
}
#[test]
fn geometry_visibility_death_and_stream_cancel_restore_without_motion() {
    let mut w = setup();
    w.pointer_stream(1, 10, 10, 0).unwrap();
    assert_eq!(preference(&mut w, 0, 1, wire::CURSOR_HIDDEN), 0);
    w.visible(0, false).unwrap();
    assert_eq!(w.cursor(0).unwrap().mode, wire::CURSOR_DEFAULT);
    w.visible(0, true).unwrap();
    assert_eq!(w.cursor(0).unwrap().mode, wire::CURSOR_HIDDEN);
    w.place(
        0,
        Rect {
            x: 20,
            y: 0,
            width: 50,
            height: 80,
        },
        wire::VALID_KEY_MASK,
    )
    .unwrap();
    assert_eq!(w.cursor(0).unwrap().mode, wire::CURSOR_DEFAULT);
    w.pointer_stream(1, 25, 10, 0).unwrap();
    let cycle = w.cursor(0).unwrap().cycle;
    w.cancel_stream(1).unwrap();
    assert!(w.cursor(0).is_none());
    w.pointer_stream(1, 25, 10, 0).unwrap();
    assert!(w.cursor(0).unwrap().cycle > cycle);
    assert_eq!(w.cursor(0).unwrap().mode, wire::CURSOR_DEFAULT);
    assert_eq!(preference(&mut w, 0, 1, wire::CURSOR_HIDDEN), 0);
    let old = epoch(&mut w, 0);
    w.disconnected(0);
    assert_eq!(w.cursor(0).unwrap().mode, wire::CURSOR_DEFAULT);
    w.place(
        0,
        Rect {
            x: 0,
            y: 0,
            width: 50,
            height: 80,
        },
        wire::VALID_KEY_MASK,
    )
    .unwrap();
    assert_eq!(
        w.request(0, 99, [wire::CURSOR_PREFERENCE, old, 1, 1, 0, 0, 0, 0])[0],
        abi::DENIED
    );
    w.inherit(Grant::default());
    assert!(w.cursor(0).is_none());
}
#[test]
fn keys_unknown_aliases_and_malformed_modes_cannot_hide_a_cursor() {
    let mut w = setup();
    w.bind_input_stream(1, 1).unwrap();
    w.select_input_group(1, Some(0)).unwrap();
    w.key_stream(1, cathedral_platform_contracts::input::A as u64, 1)
        .unwrap();
    assert_eq!(preference(&mut w, 0, 1, wire::CURSOR_HIDDEN), abi::DENIED);
    w.pointer_stream(1, 10, 10, 0).unwrap();
    assert_eq!(preference(&mut w, 0, 1, 2), abi::DENIED);
    assert_eq!(preference(&mut w, 0, 0, 1), abi::DENIED);
    assert_eq!(preference(&mut w, 0, u64::MAX, 1), abi::DENIED);
    assert_eq!(w.cursor(0).unwrap().mode, wire::CURSOR_DEFAULT);
}

#[test]
fn exhausted_cancellation_cycle_cannot_be_revived() {
    let mut w = setup();
    w.pointer_stream(1, 10, 10, 0).unwrap();
    w.input.streams[0].cursor_cycle = u64::MAX;
    w.cancel_stream(1).unwrap();
    assert_eq!(w.input.streams[0].cursor_cycle, 0);
    w.cancel_stream(1).unwrap();
    assert_eq!(w.pointer_stream(1, 10, 10, 0), Err(abi::NO_MEMORY));
    assert!(w.cursor(0).is_none());
}

#[test]
fn recursive_preferences_translate_aliases_and_cancel_without_reusing_old_policy() {
    use super::super::tests::{drain, forward, world};
    let mut outer = world();
    let mut inner = world();
    forward(&mut outer, &mut inner);
    inner
        .place(
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
    inner.request(0, 1, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
    outer.pointer_stream(99, 20, 20, 0).unwrap();
    forward(&mut outer, &mut inner);
    let event = drain(&mut inner, 0)
        .into_iter()
        .find(|e| e[2] == wire::POINTER)
        .unwrap();
    assert_ne!(event[6], 99);
    assert_eq!(
        inner.request(
            0,
            1,
            [
                wire::CURSOR_PREFERENCE,
                event[1],
                event[6],
                wire::CURSOR_HIDDEN,
                0,
                0,
                0,
                0
            ]
        )[0],
        0
    );
    let c = inner.cursor(0).unwrap();
    assert!(c.inherited);
    assert_ne!(c.channel, 99);
    let epoch = outer.request(0, 1, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0])[1];
    assert_eq!(
        outer.request(
            0,
            1,
            [
                wire::CURSOR_PREFERENCE,
                epoch,
                c.channel as u64,
                c.mode,
                0,
                0,
                0,
                0
            ]
        )[0],
        0
    );
    assert_eq!(outer.cursor(0).unwrap().mode, wire::CURSOR_HIDDEN);
    outer.cancel_stream(99).unwrap();
    forward(&mut outer, &mut inner);
    assert!(inner.cursor(0).is_none());
    assert_eq!(
        inner.request(
            0,
            1,
            [
                wire::CURSOR_PREFERENCE,
                event[1],
                event[6],
                wire::CURSOR_HIDDEN,
                0,
                0,
                0,
                0
            ]
        )[0],
        abi::DENIED
    );
    outer.pointer_stream(99, 20, 20, 0).unwrap();
    forward(&mut outer, &mut inner);
    let fresh = inner.cursor(0).unwrap();
    assert!(fresh.cycle > c.cycle);
    assert_eq!(fresh.mode, wire::CURSOR_DEFAULT);
    assert_eq!(outer.cursor(0).unwrap().mode, wire::CURSOR_DEFAULT);
}
