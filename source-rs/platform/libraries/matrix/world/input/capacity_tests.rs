use super::tests::{drain, forward, next, world};
use super::*;
use cathedral_platform_contracts::input as keys;

fn limit(w: &mut World<2>, slot: usize, channels: u64, groups: u64) -> wire::Packet {
    let epoch = w.request(slot, 1, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0])[1];
    w.request(
        slot,
        1,
        [wire::INPUT_CAPACITY, epoch, channels, groups, 0, 0, 0, 0],
    )
}

#[test]
fn refusal_precedes_focus_capture_and_key_hold_changes() {
    let mut w = world();
    assert_eq!(limit(&mut w, 0, 1, 1)[0], 0);
    w.pointer_stream(1, 20, 20, 0).unwrap();
    drain(&mut w, 0);
    assert_eq!(w.pointer_stream(2, 20, 20, 1), Err(abi::NO_MEMORY));
    let second = w
        .input
        .streams
        .iter()
        .position(|s| s.id == Name::local(2))
        .unwrap();
    assert_eq!(w.input.streams[second].buttons, 0);
    assert!(w.input.streams[second].capture.is_none());
    assert!(
        w.input.groups[w.input.streams[second].group]
            .focus
            .is_none()
    );
    assert!(drain(&mut w, 0).is_empty());
    w.select_input_group(2, Some(0)).unwrap();
    assert_eq!(
        w.key_stream(2, keys::A as u64, keys::PRESS as u64),
        Err(abi::NO_MEMORY)
    );
    assert_eq!(w.input.streams[second].held, 0);
    assert_eq!(limit(&mut w, 0, 2, 2)[0], 0);
    w.pointer_stream(2, 20, 20, 1).unwrap();
    assert_eq!(next(&mut w, 0)[2], wire::POINTER);
    w.key_stream(2, keys::A as u64, keys::PRESS as u64).unwrap();
    assert_eq!(next(&mut w, 0)[2], wire::KEY);
}

#[test]
fn capacity_counts_live_names_and_retirement_not_just_queued_events() {
    let mut w = world();
    let group = w.register_input_group().unwrap();
    let channel = w.register_input_stream(group).unwrap();
    w.pointer_stream(channel, 20, 20, 0).unwrap();
    drain(&mut w, 0);
    assert_eq!(limit(&mut w, 0, 0, 0)[0], abi::BUSY);
    assert_eq!(w.children[0].input_capacity, [4, 4]);
    w.retire_input_stream(channel).unwrap();
    assert_eq!(limit(&mut w, 0, 0, 1)[0], abi::BUSY);
    assert_eq!(next(&mut w, 0)[2], wire::RETIRED);
    assert_eq!(limit(&mut w, 0, 0, 1)[0], 0);
    w.release_input_group(group).unwrap();
    assert_eq!(limit(&mut w, 0, 0, 0)[0], abi::BUSY);
    assert_eq!(next(&mut w, 0)[2], wire::GROUP_RETIRED);
    assert_eq!(limit(&mut w, 0, 0, 0)[0], 0);
}

#[test]
fn limits_are_connection_local_and_survive_view_changes() {
    let mut w = world();
    assert_eq!(limit(&mut w, 0, 1, 1)[0], 0);
    let stale = w.children[0].grant.epoch;
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
    assert_eq!(w.children[0].input_capacity, [1, 1]);
    assert_eq!(w.children[1].input_capacity, [4, 4]);
    assert_eq!(
        w.request(0, 1, [wire::INPUT_CAPACITY, stale, 4, 4, 0, 0, 0, 0])[0],
        abi::DENIED
    );
    assert_eq!(limit(&mut w, 0, 5, 1)[0], abi::DENIED);
    w.disconnected(0);
    assert_eq!(w.children[0].input_capacity, [4, 4]);
}

#[test]
fn group_capacity_is_independent_of_channel_capacity() {
    let mut w = world();
    assert_eq!(limit(&mut w, 0, 3, 1)[0], 0);
    w.bind_input_stream(1, 1).unwrap();
    w.bind_input_stream(2, 1).unwrap();
    w.pointer_stream(1, 20, 20, 0).unwrap();
    w.pointer_stream(2, 20, 20, 0).unwrap();
    assert_eq!(drain(&mut w, 0).len(), 2);
    assert_eq!(w.pointer_stream(3, 20, 20, 0), Err(abi::NO_MEMORY));
    assert_eq!(limit(&mut w, 0, 3, 2)[0], 0);
    w.pointer_stream(3, 20, 20, 0).unwrap();
}

#[test]
fn downstream_refusal_preserves_upstream_grant_and_suppresses_failed_hold() {
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
    assert_eq!(limit(&mut inner, 0, 1, 1)[0], 0);
    outer.pointer_stream(1, 20, 20, 0).unwrap();
    forward(&mut outer, &mut inner);
    drain(&mut inner, 0);
    outer.pointer_stream(2, 20, 20, 1).unwrap();
    forward(&mut outer, &mut inner);
    assert!(inner.parent.valid());
    assert!(drain(&mut inner, 0).is_empty());
    assert_eq!(limit(&mut inner, 0, 2, 2)[0], 0);
    outer.pointer_stream(2, 25, 20, 1).unwrap();
    forward(&mut outer, &mut inner);
    assert!(drain(&mut inner, 0).is_empty()); // No phantom new press after refusal.
    outer.pointer_stream(2, 25, 20, 0).unwrap();
    forward(&mut outer, &mut inner);
    outer.pointer_stream(2, 25, 20, 1).unwrap();
    forward(&mut outer, &mut inner);
    assert_eq!(next(&mut inner, 0)[2], wire::POINTER);
}

#[test]
fn inherited_headroom_includes_retiring_local_allocations() {
    let mut w = world();
    w.request(0, 1, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(w.inherited_input_headroom(2, 1), Ok([2, 3]));
    w.grant_input_producer(0, 2, 0, 2, 3).unwrap();
    let token = w.request(0, 1, [wire::PRODUCER_INFO, 0, 0, 0, 0, 0, 0, 0])[1];
    w.request(0, 1, [wire::PRODUCER_POINTER, token, 1, 220, 20, 0, 0, 0]);
    w.revoke_input_producer(0);
    assert_eq!(w.inherited_input_headroom(0, 0), Ok([3, 3])); // Unseen second channel is already freed.
    drain(&mut w, 1);
    w.input_ready();
    drain(&mut w, 1);
    assert_eq!(w.inherited_input_headroom(0, 0), Ok([4, 4]));
}

#[test]
fn hardware_sample_refusal_cannot_kill_adapter_or_replay_a_press() {
    let mut w = world();
    assert_eq!(limit(&mut w, 0, 0, 0)[0], 0);
    assert_eq!(w.source_sample(1, [wire::POINTER, 20, 20, 1]), Ok(()));
    assert_eq!(limit(&mut w, 0, 1, 1)[0], 0);
    w.source_sample(1, [wire::POINTER, 21, 20, 1]).unwrap();
    assert!(drain(&mut w, 0).is_empty());
    w.source_sample(1, [wire::POINTER, 21, 20, 0]).unwrap();
    w.source_sample(1, [wire::POINTER, 21, 20, 1]).unwrap();
    assert_eq!(next(&mut w, 0)[2], wire::POINTER);
}
