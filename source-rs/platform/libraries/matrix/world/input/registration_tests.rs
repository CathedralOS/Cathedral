use super::tests::{drain, forward, next, world};
use super::*;
use cathedral_platform_contracts::input as keys;

#[test]
fn inherited_and_local_names_never_share_selection_or_capture() {
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
    inner
        .place(
            1,
            Rect {
                x: 100,
                y: 0,
                width: 100,
                height: 200,
            },
            wire::VALID_KEY_MASK,
        )
        .unwrap();
    let group = inner.register_input_group().unwrap();
    let local = inner.register_input_stream(group).unwrap();
    assert_eq!((group, local), (1, 1));
    inner.pointer_stream(local, 120, 20, 1).unwrap();
    outer.pointer_stream(1, 20, 20, 1).unwrap();
    forward(&mut outer, &mut inner); // Upstream compact stream/group names also 1.
    drain(&mut inner, 0);
    drain(&mut inner, 1);
    inner
        .key_stream(local, keys::A as u64, keys::PRESS as u64)
        .unwrap();
    outer
        .key_stream(1, keys::Z as u64, keys::PRESS as u64)
        .unwrap();
    forward(&mut outer, &mut inner);
    assert_eq!(next(&mut inner, 0)[3], keys::Z as u64);
    assert_eq!(next(&mut inner, 1)[3], keys::A as u64);
    inner.retire_input_stream(local).unwrap();
    assert_eq!(next(&mut inner, 1)[2], wire::RETIRED);
    outer.pointer_stream(1, 150, 20, 1).unwrap();
    forward(&mut outer, &mut inner);
    assert_eq!(&next(&mut inner, 0)[2..6], &[wire::POINTER, 99, 20, 1]);
}

#[test]
fn retirement_waits_for_every_recipient_and_never_reuses_a_name() {
    let mut w = world();
    let group = w.register_input_group().unwrap();
    let mut streams = [0; wire::INPUT_STREAMS];
    for channel in &mut streams {
        *channel = w.register_input_stream(group).unwrap();
    }
    let old = streams[0];
    w.pointer_stream(old, 20, 20, 0).unwrap();
    let first = next(&mut w, 0);
    w.pointer_stream(old, 220, 20, 1).unwrap();
    let second = next(&mut w, 1);
    w.key_stream(old, keys::A as u64, keys::PRESS as u64)
        .unwrap();
    w.retire_input_stream(old).unwrap();
    w.cancel(); // Cancellation must not overwrite the retirement boundary.
    assert!(!w.input_ready());
    assert_eq!(w.register_input_stream(group), Err(abi::NO_MEMORY));
    assert_eq!(w.release_input_group(group), Err(abi::BUSY));
    assert_eq!(w.pointer_stream(old, 20, 20, 1), Err(abi::PEER_CLOSED));
    assert_eq!(
        next(&mut w, 0)[2..8],
        [wire::RETIRED, 0, 0, 0, first[6], first[7]]
    );
    assert_eq!(w.register_input_stream(group), Err(abi::NO_MEMORY));
    assert_eq!(
        next(&mut w, 1)[2..8],
        [wire::RETIRED, 0, 0, 0, second[6], second[7]]
    );
    assert!(w.input_ready());
    let new = w.register_input_stream(group).unwrap();
    assert!(new > *streams.last().unwrap());
    assert_eq!(w.bind_input_stream(old, group), Err(abi::PEER_CLOSED));
    w.pointer_stream(new, 20, 20, 1).unwrap();
    let fresh = next(&mut w, 0);
    assert_ne!(fresh[6], first[6]);
    assert_eq!(fresh[7], first[7]); // Retained group kept its identity.
}

#[test]
fn repeated_registrations_reclaim_slots_and_app_reception_state() {
    let mut w = world();
    let mut app = crate::reception::Pointers::default();
    let mut previous = (0, 0);
    for _ in 0..64 {
        let group = w.register_input_group().unwrap();
        let channel = w.register_input_stream(group).unwrap();
        w.pointer_stream(channel, 20, 20, 1).unwrap();
        let e = next(&mut w, 0);
        assert!(e[6] > previous.0 && e[7] > previous.1);
        assert!(app.update(e).unwrap().left_pressed());
        previous = (e[6], e[7]);
        w.retire_input_stream(channel).unwrap();
        app.update(next(&mut w, 0));
        w.release_input_group(group).unwrap();
        let end = next(&mut w, 0);
        assert_eq!(end[2], wire::GROUP_RETIRED);
        assert_eq!((end[6], end[7]), (0, e[7]));
        app.update(end);
        assert_eq!(
            w.key_stream(channel, keys::A as u64, keys::PRESS as u64),
            Err(abi::PEER_CLOSED)
        );
    }
}

#[test]
fn inherited_group_remembers_selection_between_channel_registrations() {
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
    let group = outer.register_input_group().unwrap();
    let old = outer.register_input_stream(group).unwrap();
    outer.pointer_stream(old, 20, 20, 1).unwrap();
    forward(&mut outer, &mut inner);
    let first = next(&mut inner, 0);
    outer.retire_input_stream(old).unwrap();
    forward(&mut outer, &mut inner);
    assert_eq!(next(&mut inner, 0)[2], wire::RETIRED);
    assert!(inner.input_ready());
    let new = outer.register_input_stream(group).unwrap();
    outer
        .key_stream(new, keys::A as u64, keys::PRESS as u64)
        .unwrap();
    forward(&mut outer, &mut inner);
    let key = next(&mut inner, 0);
    assert_eq!(&key[2..5], &[wire::KEY, keys::A as u64, keys::PRESS as u64]);
    assert_ne!(key[6], first[6]);
    assert_eq!(key[7], first[7]);
}

#[test]
fn repeated_nested_groups_end_explicitly_without_rebinding_the_view() {
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
    let mut previous = (0, 0);
    for _ in 0..64 {
        let group = outer.register_input_group().unwrap();
        let channel = outer.register_input_stream(group).unwrap();
        outer.pointer_stream(channel, 20, 20, 1).unwrap();
        forward(&mut outer, &mut inner);
        let event = next(&mut inner, 0);
        assert!(event[6] > previous.0 && event[7] > previous.1);
        previous = (event[6], event[7]);
        outer.retire_input_stream(channel).unwrap();
        forward(&mut outer, &mut inner);
        assert!(!inner.input_ready());
        assert_eq!(next(&mut inner, 0)[2], wire::RETIRED);
        assert!(inner.input_ready());
        outer.release_input_group(group).unwrap();
        assert!(!outer.input_ready());
        forward(&mut outer, &mut inner);
        assert!(!inner.input_ready());
        let end = next(&mut inner, 0);
        assert_eq!(end[2..8], [wire::GROUP_RETIRED, 0, 0, 0, 0, event[7]]);
        assert!(inner.input_ready() && outer.input_ready());
        assert!(inner.parent.valid());
    }
}

#[test]
fn owner_registration_does_not_make_ids_child_injection_authority() {
    let mut w = world();
    let group = w.register_input_group().unwrap();
    let channel = w.register_input_stream(group).unwrap();
    for command in [
        [wire::INPUT_GROUP_CREATE, 0, 0, 0, 0, 0, 0, 0],
        [wire::INPUT_CHANNEL_CREATE, group as u64, 0, 0, 0, 0, 0, 0],
        [wire::INPUT_POINTER, channel as u64, 20, 20, 1, 0, 0, 0],
        [
            wire::INPUT_KEY,
            channel as u64,
            keys::A as u64,
            keys::PRESS as u64,
            0,
            0,
            0,
            0,
        ],
        [wire::INPUT_SELECT, group as u64, 1, 0, 0, 0, 0, 0],
        [wire::INPUT_CHANNEL_RETIRE, channel as u64, 0, 0, 0, 0, 0, 0],
        [wire::INPUT_GROUP_RELEASE, group as u64, 0, 0, 0, 0, 0, 0],
    ] {
        assert_eq!(w.request(0, 1, command), wire::error(abi::DENIED));
    }
    assert!(drain(&mut w, 0).is_empty());
    assert_eq!(
        w.owner_command([wire::INPUT_POINTER, 999, 20, 20, 1, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    assert_eq!(
        w.owner_command([wire::INPUT_POINTER, channel as u64, 20, 20, 1, 0, 0, 0]),
        [0; 8]
    );
    assert_eq!(next(&mut w, 0)[2], wire::POINTER);
    w.input.last_local_stream = u32::MAX;
    w.input.last_local_group = u32::MAX;
    assert_eq!(w.register_input_stream(group), Err(abi::NO_MEMORY));
    assert_eq!(w.register_input_group(), Err(abi::NO_MEMORY));
}

#[test]
fn nested_retirement_backpressure_preserves_other_channels_and_the_grant() {
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
    let group = outer.register_input_group().unwrap();
    let mut channels = [0; wire::INPUT_STREAMS];
    for channel in &mut channels {
        *channel = outer.register_input_stream(group).unwrap();
        outer.pointer_stream(*channel, 20, 20, 1).unwrap();
    }
    forward(&mut outer, &mut inner);
    drain(&mut inner, 0);
    outer.retire_input_stream(channels[0]).unwrap();
    forward(&mut outer, &mut inner);
    assert!(!inner.input_ready());
    let new = outer.register_input_stream(group).unwrap();
    outer.pointer_stream(new, 20, 20, 1).unwrap();
    inner.cancel();
    let ended = drain(&mut inner, 0);
    assert!(ended.iter().any(|e| e[2] == wire::RETIRED));
    assert!(inner.input_ready());
    forward(&mut outer, &mut inner);
    assert_eq!(next(&mut inner, 0)[2], wire::POINTER);
    assert!(inner.parent.valid());
}

#[test]
fn removing_a_recipient_reclaims_retirement_without_waiting_for_dead_code() {
    let mut w = world();
    let group = w.register_input_group().unwrap();
    let channel = w.register_input_stream(group).unwrap();
    w.pointer_stream(channel, 20, 20, 1).unwrap();
    w.retire_input_stream(channel).unwrap();
    w.disconnected(0);
    assert!(w.input_ready());
    w.release_input_group(group).unwrap();
    assert_eq!(w.register_input_stream(group), Err(abi::DENIED));
}
