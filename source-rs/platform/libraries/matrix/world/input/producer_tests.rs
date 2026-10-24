use super::tests::{drain, next, world};
use super::*;
use cathedral_platform_contracts::input as keys;

fn connect(w: &mut World<2>, slot: usize) {
    assert_eq!(w.request(slot, 1, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0])[0], 0);
}
fn info(w: &mut World<2>, slot: usize) -> wire::Packet {
    w.request(slot, 1, [wire::PRODUCER_INFO, 0, 0, 0, 0, 0, 0, 0])
}
fn pointer(w: &mut World<2>, slot: usize, generation: u64, x: u64) -> wire::Packet {
    w.request(
        slot,
        1,
        [wire::PRODUCER_POINTER, generation, 1, x, 20, 1, 0, 0],
    )
}

#[test]
fn explicit_grants_separate_identity_quota_and_focus_authority() {
    let mut w = world();
    connect(&mut w, 0);
    connect(&mut w, 1);
    assert_eq!(info(&mut w, 0), wire::error(abi::DENIED));
    w.grant_input_producer(0, 1, 0, 2, 1).unwrap();
    w.grant_input_producer(1, 1, wire::PRODUCER_FOCUS, 2, 1)
        .unwrap();
    let first = info(&mut w, 0);
    let second = info(&mut w, 1);
    assert_eq!(first[1], second[1]); // No sibling grant counter leaks into this token.
    w.revoke_input_producer(1);
    w.grant_input_producer(1, 1, wire::PRODUCER_FOCUS, 2, 1)
        .unwrap();
    let second = info(&mut w, 1);
    assert_eq!(&first[2..], &[400, 200, wire::VALID_KEY_MASK, 1, 0, 0]);
    assert_ne!(first[1], second[1]);
    assert_eq!(pointer(&mut w, 0, second[1], 220)[0], abi::DENIED);
    assert_eq!(
        w.request(
            0,
            1,
            [wire::PRODUCER_POINTER, first[1], 2, 220, 20, 1, 0, 0]
        )[0],
        abi::DENIED
    );
    assert_eq!(
        w.request(0, 1, [wire::INPUT_SELECT, 2, 2, 0, 0, 0, 0, 0])[0],
        abi::DENIED
    );
    assert_eq!(pointer(&mut w, 0, first[1], 220), [0; 8]);
    assert_eq!(pointer(&mut w, 1, second[1], 20), [0; 8]);
    drain(&mut w, 0);
    drain(&mut w, 1);
    for (slot, token, key) in [(0, first[1], keys::A), (1, second[1], keys::Z)] {
        assert_eq!(
            w.request(
                slot,
                1,
                [
                    wire::PRODUCER_KEY,
                    token,
                    1,
                    key as u64,
                    keys::PRESS as u64,
                    0,
                    0,
                    0
                ]
            ),
            [0; 8]
        );
    }
    // Producer zero may deliver pointer events but cannot select a keyboard
    // target. Producer one's independently permitted click selects its own group.
    assert_eq!(next(&mut w, 0)[3], keys::Z as u64);
    assert!(drain(&mut w, 1).is_empty());
}

#[test]
fn admission_reserves_capacity_before_inherited_sources_arrive() {
    let mut w = world();
    connect(&mut w, 0);
    assert_eq!(w.grant_input_producer(0, 3, 0, 2, 1), Err(abi::NO_MEMORY));
    assert!(w.input.streams.iter().all(|s| !s.id.valid()));
    w.grant_input_producer(0, 2, 0, 2, 1).unwrap();
    assert_eq!(w.grant_input_producer(0, 1, 0, 0, 0), Err(abi::BUSY));
    w.input.bind(Name::upstream(1), Name::upstream(1)).unwrap();
    w.input.bind(Name::upstream(2), Name::upstream(1)).unwrap();
    assert!(w.input.streams.iter().all(|s| s.id.valid()));
}

#[test]
fn release_retires_held_input_before_capacity_can_be_regranted() {
    let mut w = world();
    connect(&mut w, 0);
    w.grant_input_producer(0, 4, wire::PRODUCER_FOCUS, 0, 0)
        .unwrap();
    let old = info(&mut w, 0)[1];
    assert_eq!(pointer(&mut w, 0, old, 220), [0; 8]);
    let held = next(&mut w, 1);
    assert_eq!(
        w.request(0, 1, [wire::PRODUCER_RELEASE, old, 0, 0, 0, 0, 0, 0]),
        [0; 8]
    );
    assert_eq!(pointer(&mut w, 0, old, 220)[0], abi::DENIED);
    assert_eq!(w.grant_input_producer(0, 4, 0, 0, 0), Err(abi::NO_MEMORY));
    let retired = next(&mut w, 1);
    assert_eq!((retired[2], retired[6]), (wire::RETIRED, held[6]));
    assert!(!w.input_ready()); // Group boundary still pending.
    assert_eq!(next(&mut w, 1)[2], wire::GROUP_RETIRED);
    assert!(w.input_ready());
    w.grant_input_producer(0, 4, 0, 0, 0).unwrap();
    assert!(info(&mut w, 0)[1] > old);
    assert_eq!(pointer(&mut w, 0, old, 220)[0], abi::DENIED);
}

#[test]
fn disconnect_resize_and_parent_replacement_end_grants() {
    for mode in 0..4 {
        let mut w = world();
        connect(&mut w, 0);
        w.grant_input_producer(0, 1, wire::PRODUCER_FOCUS, 0, 0)
            .unwrap();
        let token = info(&mut w, 0)[1];
        pointer(&mut w, 0, token, 220);
        next(&mut w, 1);
        match mode {
            0 => w.disconnected(0),
            1 => w
                .place(
                    0,
                    Rect {
                        x: 0,
                        y: 0,
                        width: 100,
                        height: 100,
                    },
                    wire::VALID_KEY_MASK,
                )
                .unwrap(),
            2 => {
                w.request(0, 2, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
            }
            _ => {
                let mut grant = w.parent;
                grant.epoch += 1;
                w.inherit(grant);
            }
        }
        assert_eq!(info(&mut w, 0)[0], abi::DENIED);
        if mode != 3 {
            assert_eq!(next(&mut w, 1)[2], wire::RETIRED);
            assert!(!w.input_ready());
            assert_eq!(next(&mut w, 1)[2], wire::GROUP_RETIRED);
        }
        assert!(w.input_ready());
        assert!(w.input.streams.iter().all(|s| !s.id.valid()));
    }
}

#[test]
fn grant_counter_exhaustion_and_invalid_shape_cannot_publish_partial_authority() {
    let mut w = world();
    assert_eq!(w.grant_input_producer(0, 1, 0, 0, 0), Err(abi::DENIED));
    connect(&mut w, 0);
    for (count, rights, reserved) in [(0, 0, 0), (5, 0, 0), (1, 2, 0), (1, 0, 5)] {
        assert_eq!(
            w.grant_input_producer(0, count, rights, reserved, 0),
            Err(abi::DENIED)
        );
    }
    w.next_producer[0] = u64::MAX;
    assert_eq!(w.grant_input_producer(0, 1, 0, 0, 0), Err(abi::NO_MEMORY));
    assert!(w.input.groups.iter().all(|g| !g.id.valid()));
    assert_eq!(info(&mut w, 0)[0], abi::DENIED);
}
