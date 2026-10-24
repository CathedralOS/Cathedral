use super::tests::{drain, forward, next, world};
use super::*;
use cathedral_platform_contracts::input::description::{self, Description};

fn description() -> Description {
    Description::new(description::POINTER | description::KEYS, "Virtual input").unwrap()
}
fn queried(w: &mut World<2>, slot: usize, name: u64) -> wire::Packet {
    let epoch = w.request(slot, 1, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0])[1];
    let row = w.request(slot, 1, [wire::INPUT_DIRECTORY, epoch, 0, 0, 0, 0, 0, 0]);
    w.request(
        slot,
        1,
        [wire::INPUT_DESCRIPTION, epoch, row[2], name, 0, 0, 0, 0],
    )
}

#[test]
fn descriptions_precede_input_once_and_do_not_expose_unrouted_registrations() {
    let mut w = world();
    w.bind_input_stream(19, 19).unwrap();
    w.describe_input_stream(19, description()).unwrap();
    assert_eq!(queried(&mut w, 0, 1)[0], abi::DENIED);
    w.pointer_stream(19, 20, 20, 0).unwrap();
    assert_eq!(queried(&mut w, 1, 1)[0], abi::DENIED);
    let row = queried(&mut w, 0, 1);
    assert_eq!(&row[4..7], &description().words());
    let event = next(&mut w, 0);
    assert_eq!(event[2], wire::DESCRIBED);
    assert_eq!(&event[3..6], &description().words());
    assert_eq!((event[6], event[7]), (1, 1));
    assert_eq!(next(&mut w, 0)[2], wire::POINTER);
    w.pointer_stream(19, 30, 20, 0).unwrap();
    assert_eq!(next(&mut w, 0)[2], wire::POINTER);
    assert_eq!(next(&mut w, 0)[2], wire::NONE);
    assert_eq!(w.describe_input_stream(19, description()), Ok(()));
    assert_eq!(
        w.describe_input_stream(19, Description::new(description::KEYS, "Changed").unwrap()),
        Err(abi::BUSY)
    );
    // First-time late publication also refuses, even when ordinary events drained.
    w.pointer_stream(73, 20, 20, 0).unwrap();
    drain(&mut w, 0);
    assert_eq!(w.describe_input_stream(73, description()), Err(abi::BUSY));
    w.revoke(0).unwrap();
    assert_eq!(w.describe_input_stream(73, description()), Err(abi::BUSY));
}

#[test]
fn descriptor_survives_pressure_and_retirement_without_an_extra_queue_slot() {
    for retire in [false, true] {
        let mut w = world();
        w.bind_input_stream(19, 19).unwrap();
        w.describe_input_stream(19, description()).unwrap();
        for i in 0..18 {
            w.pointer_stream(19, 20, 20, i % 2).unwrap();
        }
        if retire {
            w.retire_input_stream(19).unwrap();
        }
        let row = queried(&mut w, 0, 1);
        assert_eq!(&row[4..7], &description().words());
        assert_eq!(next(&mut w, 0)[2], wire::DESCRIBED);
        assert_eq!(
            next(&mut w, 0)[2],
            if retire { wire::RETIRED } else { wire::CANCEL }
        );
        if retire {
            assert_eq!(queried(&mut w, 0, 1)[0], abi::DENIED);
        } else {
            assert_eq!(&queried(&mut w, 0, 1)[4..7], &description().words());
        }
    }
}

#[test]
fn nested_description_survives_cancellation_and_uses_receiver_local_names() {
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
    inner.bind_input_stream(8, 8).unwrap();
    inner.pointer_stream(8, 20, 20, 0).unwrap();
    drain(&mut inner, 0); // Occupy local alias one without an upstream source.
    outer.bind_input_stream(73, 73).unwrap();
    outer.describe_input_stream(73, description()).unwrap();
    outer.pointer_stream(73, 20, 20, 0).unwrap();
    forward(&mut outer, &mut inner);
    let events = drain(&mut inner, 0);
    assert_eq!(events.len(), 2);
    assert_eq!((events[0][2], events[0][6]), (wire::DESCRIBED, 2));
    assert_eq!(&queried(&mut inner, 0, 2)[4..7], &description().words());
    outer.cancel_stream(73).unwrap();
    forward(&mut outer, &mut inner);
    assert_eq!(next(&mut inner, 0)[2], wire::CANCEL);
    outer.pointer_stream(73, 20, 20, 0).unwrap();
    forward(&mut outer, &mut inner);
    assert_eq!(next(&mut inner, 0)[2], wire::POINTER);
    assert_eq!(&queried(&mut inner, 0, 2)[4..7], &description().words());
    assert_eq!(queried(&mut inner, 1, 2)[0], abi::DENIED);
}

#[test]
fn producer_description_checks_grant_generation_and_own_ordinal() {
    let mut w = world();
    for slot in 0..2 {
        w.request(slot, 1, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
    }
    let [a, b, controls] = description().words();
    let mut packet = [wire::PRODUCER_DESCRIBE, 1, 1, a, b, controls, 0, 0];
    assert_eq!(w.request(0, 1, packet)[0], abi::DENIED);
    w.grant_input_producer(0, 1, 0, 0, 0).unwrap();
    assert_eq!(w.request(0, 1, packet), [0; 8]);
    assert_eq!(w.request(1, 1, packet)[0], abi::DENIED);
    packet[1] = 2;
    assert_eq!(w.request(0, 1, packet)[0], abi::DENIED);
    packet[1] = 1;
    packet[2] = 2;
    assert_eq!(w.request(0, 1, packet)[0], abi::DENIED);
    packet[2] = 1;
    packet[5] = 4;
    assert_eq!(w.request(0, 1, packet)[0], abi::INVALID_ARGUMENT);
    packet[5] = controls;
    w.revoke_input_producer(0);
    assert_eq!(w.request(0, 1, packet)[0], abi::DENIED);
}
