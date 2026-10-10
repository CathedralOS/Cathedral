use super::*;
#[test]
fn direct_crud_and_abort_preserve_root_revision_rules() {
    let mut store = Store::open(Disk::blank()).unwrap();
    let mut client = crate::protocol::Client::default();
    let mut call = |op, object, expected, payload: &[u8]| {
        client.request(
            &mut store,
            1,
            1,
            &wire::request(op, object, expected, payload).unwrap(),
            |_| {},
        )
    };
    assert_eq!(call(wire::LIST, 0, 0, b"").unwrap().length, 0);
    assert_eq!(call(wire::CREATE, 1, 0, b"first").unwrap().generation, 1);
    assert_eq!(call(wire::CREATE, 1, 1, b"duplicate"), Err(wire::EXISTS));
    call(wire::BEGIN, 0, 1, b"").unwrap();
    call(wire::STAGE_REPLACE, 1, 0, b"discard").unwrap();
    call(wire::ABORT, 0, 0, b"").unwrap();
    assert_eq!(call(wire::READ, 1, 0, b"").unwrap().payload(), b"first");
    assert_eq!(call(wire::REPLACE, 1, 1, b"second").unwrap().generation, 2);
    assert_eq!(call(wire::DELETE, 1, 2, b"").unwrap().generation, 3);
    assert_eq!(call(wire::READ, 1, 0, b""), Err(wire::NOT_FOUND));
    assert_eq!(call(wire::CREATE, 1, 2, b"stale"), Err(abi::WOULD_BLOCK));
    call(wire::CREATE, 1, 3, b"reborn").unwrap();
    call(wire::BEGIN, 0, 4, b"").unwrap();
    assert_eq!(call(wire::COMMIT, 0, 0, b""), Err(abi::INVALID_ARGUMENT));
    assert_eq!(call(wire::COMMIT, 0, 0, b""), Err(abi::BAD_HANDLE));
    assert_eq!(call(wire::READ, 1, 0, b"").unwrap().generation, 4);
}
#[test]
fn pending_transactions_are_bounded_and_die_with_the_connection() {
    use crate::protocol::Client;
    let mut store = Store::open(Disk::blank()).unwrap();
    let mut client = Client::default();
    let mut call = |incarnation, op, object, expected, payload: &[u8]| {
        client.request(
            &mut store,
            0,
            incarnation,
            &wire::request(op, object, expected, payload).unwrap(),
            |_| {},
        )
    };
    call(1, wire::BEGIN, 0, 0, b"").unwrap();
    call(1, wire::STAGE_REPLACE, 1, 0, b"uncommitted").unwrap();
    assert_eq!(call(2, wire::COMMIT, 0, 0, b""), Err(abi::BAD_HANDLE));
    assert_eq!(call(2, wire::READ, 1, 0, b"").unwrap().payload(), b"");
    call(2, wire::BEGIN, 0, 0, b"").unwrap();
    call(2, wire::STAGE_CREATE, 2, 0, b"a").unwrap();
    assert_eq!(
        call(2, wire::STAGE_CREATE, 2, 0, b"duplicate"),
        Err(abi::INVALID_ARGUMENT)
    );
    call(2, wire::STAGE_CREATE, 3, 0, b"b").unwrap();
    assert_eq!(
        call(2, wire::STAGE_CREATE, 4, 0, b"overflow"),
        Err(abi::NO_MEMORY)
    );
    call(2, wire::COMMIT, 0, 0, b"").unwrap();
    assert_eq!(call(2, wire::LIST, 0, 0, b"").unwrap().length, 24);
}

#[test]
fn competing_commits_conflict_but_other_roots_do_not() {
    let mut store = Store::open(Disk::blank()).unwrap();
    let mut client = crate::protocol::Client::default();
    let request = |op, object, generation, payload: &[u8]| {
        wire::request(op, object, generation, payload).unwrap()
    };
    client
        .request(&mut store, 0, 10, &request(wire::BEGIN, 0, 0, b""), |_| {})
        .unwrap();
    client
        .request(
            &mut store,
            0,
            10,
            &request(wire::STAGE_CREATE, 2, 0, b"pending"),
            |_| {},
        )
        .unwrap();
    store
        .transact(
            1,
            0,
            &[Change::value(wire::CREATE, 1, b"unrelated").unwrap()],
            |_| {},
        )
        .unwrap();
    client
        .request(&mut store, 0, 10, &request(wire::COMMIT, 0, 0, b""), |_| {})
        .unwrap();
    client
        .request(&mut store, 0, 10, &request(wire::BEGIN, 0, 1, b""), |_| {})
        .unwrap();
    client
        .request(
            &mut store,
            0,
            10,
            &request(wire::STAGE_DELETE, 2, 0, b""),
            |_| {},
        )
        .unwrap();
    replace(&mut store, 1, 1, b"competitor").unwrap();
    assert_eq!(
        client.request(&mut store, 0, 10, &request(wire::COMMIT, 0, 0, b""), |_| {}),
        Err(abi::WOULD_BLOCK)
    );
    assert_eq!(store.read(0, 2).unwrap().payload(), b"pending");
}

#[test]
fn malformed_packets_and_invalid_mutations_have_no_disk_effects() {
    let mut store = Store::open(Disk::blank()).unwrap();
    let mut client = crate::protocol::Client::default();
    let baseline = store.disk.operations;
    for op in 0..=12 {
        for object in [0, 1, 4, 5, u64::MAX] {
            let mut packet = wire::request(op, object, 0, b"").unwrap();
            packet[63] = 1; // Noncanonical unused bytes must fail before staging or I/O.
            assert_eq!(
                client.request(&mut store, 0, 1, &packet, |_| {}),
                Err(abi::INVALID_ARGUMENT)
            );
        }
    }
    for len in 0..64 {
        assert_eq!(
            client.request(&mut store, 0, 1, &[0; 64][..len], |_| {}),
            Err(abi::INVALID_ARGUMENT)
        );
    }
    assert_eq!(store.disk.operations, baseline);
    assert_eq!(store.read(0, 1).unwrap(), Record::default());
}
