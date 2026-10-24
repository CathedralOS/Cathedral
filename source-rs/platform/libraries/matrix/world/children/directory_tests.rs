use super::*;

fn child() -> Child {
    let mut child = Child::empty();
    child.rect = Some(Rect {
        x: 0,
        y: 0,
        width: 20,
        height: 20,
    });
    child.keys = 1;
    child.refresh(
        Grant {
            epoch: 1,
            width: 20,
            height: 20,
            keys: 3,
        },
        false,
    );
    child
}
fn push(child: &mut Child, index: usize, source: u32) {
    child
        .push(
            index,
            Name::local(source),
            0,
            Name::local(99),
            [wire::POINTER, 1, 1, 0],
        )
        .unwrap();
}

#[test]
fn snapshot_revision_tracks_membership_not_events_and_reads_do_not_consume() {
    let mut c = child();
    let epoch = c.grant.epoch;
    assert_eq!(c.directory(epoch, 0, 0), [0, epoch, 1, 0, 0, 0, 0, 0]);
    push(&mut c, 0, 873);
    let first = c.directory(epoch, 0, 0);
    assert_eq!(first, [0, epoch, 2, 1, 1, wire::CHANNEL_LIVE, 1, 0]);
    push(&mut c, 0, 873);
    c.cancel_stream(0, Name::local(873));
    assert_eq!(c.directory(epoch, 2, 0), first);
    assert_eq!(c.next()[2], wire::CANCEL);
    assert_eq!(c.directory(epoch, 2, 0), first);
    push(&mut c, 1, 912);
    assert_eq!(c.directory(epoch, 2, 1), wire::error(abi::BUSY));
    let restart = c.directory(epoch, 0, 0);
    assert_eq!(restart[2], 3);
    let second = c.directory(epoch, 3, 1);
    assert_eq!(&second[3..], &[2, 1, wire::CHANNEL_LIVE, 1, 0]);
    assert_eq!(c.directory(epoch, 3, 2), [0, epoch, 3, 0, 0, 0, 0, 0]);
    assert_eq!(c.next()[6], 2);
}

#[test]
fn retirement_stays_visible_until_consumed_then_replacement_gets_a_new_name() {
    let mut c = child();
    let epoch = c.grant.epoch;
    push(&mut c, 0, 873);
    c.retire_stream(0, Name::local(873));
    let retired = c.directory(epoch, 0, 0);
    assert_eq!(retired, [0, epoch, 3, 1, 1, wire::CHANNEL_RETIRING, 1, 0]);
    c.retire_stream(0, Name::local(873));
    assert_eq!(c.directory(epoch, 3, 0), retired);
    assert_eq!(c.next()[2], wire::RETIRED);
    assert_eq!(c.directory(epoch, 3, 0), wire::error(abi::BUSY));
    assert_eq!(c.directory(epoch, 0, 0), [0, epoch, 4, 0, 0, 0, 0, 0]);
    push(&mut c, 0, 999);
    assert_eq!(c.directory(epoch, 0, 0)[3], 2);
}

#[test]
fn invalid_binding_and_exhausted_revision_fail_closed_until_new_view() {
    let mut c = child();
    let epoch = c.grant.epoch;
    for (view, revision, after) in [(epoch + 1, 0, 0), (epoch, 0, 1), (epoch, 1, u64::MAX)] {
        assert_eq!(c.directory(view, revision, after), wire::error(abi::DENIED));
    }
    c.directory_revision = u64::MAX;
    push(&mut c, 0, 17);
    assert_eq!(c.directory(epoch, 0, 0), wire::error(abi::NO_MEMORY));
    push(&mut c, 1, 18);
    assert_eq!(c.directory(epoch, 0, 0), wire::error(abi::NO_MEMORY));
    assert_eq!(c.next()[2], wire::POINTER);
    c.refresh(
        Grant {
            epoch: 1,
            width: 20,
            height: 20,
            keys: 3,
        },
        true,
    );
    assert_eq!(c.directory(epoch, 0, 0), wire::error(abi::DENIED));
    assert_eq!(c.directory(c.grant.epoch, 0, 0)[2..], [1, 0, 0, 0, 0, 0]);
}

#[test]
fn endpoint_directory_exposes_only_routed_local_bindings() {
    let mut w = World::<2>::empty();
    w.inherit(Grant {
        epoch: 1,
        width: 40,
        height: 20,
        keys: 3,
    });
    for slot in 0..2 {
        w.place(
            slot,
            Rect {
                x: slot as u64 * 20,
                y: 0,
                width: 20,
                height: 20,
            },
            1,
        )
        .unwrap();
    }
    let query = |w: &mut World<2>, slot, revision| {
        let epoch = w.request(slot, 1, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0])[1];
        w.request(
            slot,
            1,
            [wire::INPUT_DIRECTORY, epoch, revision, 0, 0, 0, 0, 0],
        )
    };
    let empty = query(&mut w, 1, 0);
    w.take_notification(0);
    w.take_notification(1);
    w.bind_input_stream(987, 987).unwrap();
    assert_eq!(query(&mut w, 1, 1), empty);
    assert!(!w.take_notification(1));
    w.pointer_stream(987, 1, 1, 1).unwrap();
    assert!(w.take_notification(0));
    assert!(!w.take_notification(1));
    assert_eq!(query(&mut w, 1, 1), empty);
    let first = query(&mut w, 0, 0);
    assert_eq!(&first[3..], &[1, 1, wire::CHANNEL_LIVE, 1, 0]);
    w.pointer_stream(1654, 21, 1, 1).unwrap();
    assert_eq!(query(&mut w, 0, first[2]), first);
    assert_eq!(query(&mut w, 1, 0)[3], 1);
    let epoch = first[1];
    assert_eq!(
        w.request(0, 1, [wire::INPUT_DIRECTORY, epoch, 0, 0, 1, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
    // A server-observed replacement endpoint cannot read the old binding.
    assert_eq!(
        w.request(0, 2, [wire::INPUT_DIRECTORY, epoch, 0, 0, 0, 0, 0, 0]),
        wire::error(abi::DENIED)
    );
}

#[test]
fn nested_directory_renames_inherited_and_local_channels_independently() {
    let mut outer = World::<1>::empty();
    let mut inner = World::<1>::empty();
    outer.inherit(Grant {
        epoch: 1,
        width: 20,
        height: 20,
        keys: 3,
    });
    let rect = Rect {
        x: 0,
        y: 0,
        width: 20,
        height: 20,
    };
    outer.place(0, rect, 3).unwrap();
    inner.inherit_poll(outer.request(0, 1, [wire::POLL, 0, 0, 0, 0, 0, 0, 0]));
    inner.place(0, rect, 1).unwrap();
    inner.pointer_stream(98, 1, 1, 0).unwrap();
    outer.pointer_stream(876, 1, 1, 0).unwrap();
    inner.inherit_poll(outer.request(0, 1, [wire::POLL, 0, 0, 0, 0, 0, 0, 0]));
    let view = inner.request(0, 1, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
    let local = inner.request(0, 1, [wire::INPUT_DIRECTORY, view[1], 0, 0, 0, 0, 0, 0]);
    let inherited = inner.request(
        0,
        1,
        [
            wire::INPUT_DIRECTORY,
            view[1],
            local[2],
            local[3],
            0,
            0,
            0,
            0,
        ],
    );
    assert_eq!(&local[3..], &[1, 1, wire::CHANNEL_LIVE, 1, 0]);
    assert_eq!(&inherited[3..], &[2, 2, wire::CHANNEL_LIVE, 1, 0]);
    let a = wire::poll_event(inner.request(0, 1, [wire::POLL, 0, 0, 0, 0, 0, 0, 0]));
    let b = wire::poll_event(inner.request(0, 1, [wire::POLL, 0, 0, 0, 0, 0, 0, 0]));
    assert_eq!((a[6], a[7]), (local[3], local[4]));
    assert_eq!((b[6], b[7]), (inherited[3], inherited[4]));
}
