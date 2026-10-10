use super::*;
#[test]
fn acknowledged_versions_survive_reopen_and_stale_writers_cannot_overwrite() {
    let mut store = Store::open(Disk::blank()).unwrap();
    assert_eq!(store.read(0, 99), Err(abi::DENIED));
    assert_eq!(replace(&mut store, 99, 0, b"foreign"), Err(abi::DENIED));
    assert_eq!(
        replace(&mut store, 1, 0, &[0; 33]),
        Err(abi::INVALID_ARGUMENT)
    );
    for generation in 0..12 {
        let record = replace(&mut store, 1, generation, &[generation as u8; 32]).unwrap();
        store = Store::open(store.disk.power_loss()).unwrap();
        assert_eq!(store.read(0, 1).unwrap(), record);
        assert_eq!(
            replace(&mut store, 1, generation, b"stale"),
            Err(abi::WOULD_BLOCK)
        );
    }
}

#[test]
fn every_write_prefix_and_failed_flush_recovers_a_complete_committed_version() {
    let mut store = Store::open(Disk::blank()).unwrap();
    for generations in [0, 2] {
        for generation in 0..generations {
            replace(&mut store, 1, generation, b"old").unwrap();
        }
        let old = store.read(0, 1).unwrap();
        let baseline = store.disk.clone().power_loss();
        for operation in 1..=4 {
            for prefix in 0..=512 {
                let mut store = Store::open(baseline.clone()).unwrap();
                store.disk.fail = operation;
                store.disk.tear = prefix;
                assert!(replace(&mut store, 1, generations, b"new").is_err());
                assert_eq!(store.read(0, 1), Err(abi::IO_ERROR)); // Ambiguous instances cannot serve stale data.
                let recovered = Store::open(store.disk.power_loss())
                    .unwrap()
                    .read(0, 1)
                    .unwrap();
                assert!(
                    recovered == old
                        || (recovered.generation == generations + 1
                            && recovered.payload() == b"new")
                );
                if operation <= 2 {
                    assert_eq!(recovered, old);
                }
            }
        }
    }
}

#[test]
fn durable_commit_with_lost_reply_is_observable_without_duplicate_write() {
    let mut store = Store::open(Disk::blank()).unwrap();
    let committed = replace(&mut store, 1, 0, b"saved").unwrap();
    let mut recovered = Store::open(store.disk.power_loss()).unwrap();
    assert_eq!(recovered.read(0, 1).unwrap(), committed);
    assert_eq!(
        replace(&mut recovered, 1, 0, b"saved"),
        Err(abi::WOULD_BLOCK)
    );
}

#[test]
fn corrupt_or_unknown_media_is_never_silently_formatted() {
    let mut disk = Disk::blank();
    disk.cache[0][0] = 0x55;
    disk.stable = disk.cache;
    let before = disk.stable;
    assert!(Store::open(&mut disk).is_err());
    assert_eq!(disk.stable, before);
    let mut store = Store::open(Disk::blank()).unwrap();
    replace(&mut store, 1, 0, b"kept").unwrap();
    // Corrupt both commit records; no valid root remains.
    store.disk.stable[1][0] ^= 1;
    store.disk.stable[3][0] ^= 1;
    assert!(Store::open(store.disk.power_loss()).is_err());
    assert_eq!(format::crc(b"123456789"), 0xcbf43926);
}

#[test]
fn interrupted_initial_format_fails_closed_until_a_valid_baseline_exists() {
    for fail in 1..=4 {
        let mut disk = Disk::blank();
        disk.fail = fail;
        disk.tear = 256;
        assert!(Store::open(&mut disk).is_err());
        let disk = disk.power_loss();
        let reopened = Store::open(disk);
        // Before any acknowledgement or user data: incomplete format may fail
        // closed. A complete empty root is the only permitted recovered value.
        if let Ok(store) = reopened {
            assert_eq!(store.read(0, 1).unwrap(), Record::default());
        }
    }
}

#[test]
fn every_torn_transaction_preserves_both_records_and_the_other_root() {
    let mut store = Store::open(Disk::blank()).unwrap();
    store
        .transact(
            1,
            0,
            &[Change::value(wire::CREATE, 1, b"private").unwrap()],
            |_| {},
        )
        .unwrap();
    store
        .transact(
            0,
            0,
            &[
                Change::value(wire::REPLACE, 1, b"old-a").unwrap(),
                Change::value(wire::CREATE, 2, b"old-b").unwrap(),
            ],
            |_| {},
        )
        .unwrap();
    let old = store.catalog;
    let baseline = store.disk.power_loss();
    let changes = [
        Change::value(wire::REPLACE, 1, b"new-a").unwrap(),
        Change::value(wire::REPLACE, 2, b"new-b").unwrap(),
    ];
    let mut next = old;
    next.sequence += 1;
    next.roots[0] = old.roots[0].changed(1, &changes).unwrap();
    for operation in 1..=4 {
        for prefix in 0..=512 {
            let mut store = Store::open(baseline.clone()).unwrap();
            store.disk.fail = operation;
            store.disk.tear = prefix;
            assert!(store.transact(0, 1, &changes, |_| {}).is_err());
            assert!(store.needs_reopen());
            let recovered = Store::open(store.disk.power_loss()).unwrap();
            assert!(recovered.catalog == old || recovered.catalog == next);
            assert_eq!(recovered.catalog.roots[1], old.roots[1]);
        }
    }
}

#[test]
fn legacy_conversion_is_lazy_and_power_safe() {
    let mut disk = Disk::blank();
    let record = Record {
        generation: 7,
        length: 3,
        bytes: {
            let mut b = [0; 32];
            b[..3].copy_from_slice(&[1, 2, 3]);
            b
        },
    };
    let (body, commit) = legacy::encode(record);
    disk.stable[0] = body;
    disk.stable[1] = commit;
    let disk = disk.power_loss();
    for operation in 1..=4 {
        for prefix in 0..=512 {
            let mut store = Store::open(disk.clone()).unwrap();
            assert_eq!(store.read(0, 1).unwrap(), record);
            store.disk.fail = operation;
            store.disk.tear = prefix;
            let _ = store.transact(
                1,
                0,
                &[Change::value(wire::CREATE, 1, b"second").unwrap()],
                |_| {},
            );
            let store = Store::open(store.disk.power_loss()).unwrap();
            assert_eq!(store.read(0, 1).unwrap(), record);
        }
    }
}
