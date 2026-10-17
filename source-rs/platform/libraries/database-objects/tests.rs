use super::*;
use alloc::{vec, vec::Vec};
use redb::{ReadableDatabase, ReadableTable, backends::InMemoryBackend};
fn database() -> Database {
    Database::builder()
        .create_with_backend(InMemoryBackend::new())
        .unwrap()
}
fn snapshot(db: &Database) -> Vec<(Vec<u8>, Vec<u8>)> {
    let tx = db.begin_read().unwrap();
    tx.open_table(TABLE)
        .unwrap()
        .iter()
        .unwrap()
        .map(|entry| {
            let (k, v) = entry.unwrap();
            (k.value().to_vec(), v.value().to_vec())
        })
        .collect()
}
fn content() -> Vec<u8> {
    (0..8192)
        .map(|i| ((i * 17 + i / 4096 * 71) % 251) as u8)
        .collect()
}

#[test]
fn refusals_abort_every_record_and_other_roots_stay_private() {
    let db = database();
    let store = Store::open(&db).unwrap();
    let bytes = content();
    let head = store.publish(0, 0, &bytes).unwrap();
    assert_eq!(store.read(1, 0, 0, &mut []).unwrap(), Head::default());
    let before = snapshot(&db);
    assert_eq!(store.publish(0, 0, &bytes), Err(Error::Conflict));
    assert_eq!(
        store.publish(0, 1, &vec![0; MAX_BYTES + 1]),
        Err(Error::TooLarge)
    );
    // Refuse after staging multiple new chunks, not merely before opening a tx.
    let large: Vec<_> = (0..MAX_BYTES).map(|i| (200 + i / 4096) as u8).collect();
    assert_eq!(store.publish(0, 1, &large), Err(Error::Capacity));
    assert_eq!(snapshot(&db), before);
    let mut read = vec![0; bytes.len()];
    assert_eq!(store.read(0, 1, 0, &mut read).unwrap(), head);
    assert_eq!(read, bytes);
    let mut replacement = bytes.clone();
    replacement[41] ^= 128;
    assert_eq!(store.publish(0, 1, &replacement).unwrap().revision, 2);
    assert_eq!(store.read(0, 1, 0, &mut read), Err(Error::Conflict));
    store.read(0, 2, 0, &mut read).unwrap();
    assert_eq!(read, replacement);
    store.publish(1, 0, b"separate").unwrap();
    let rows = snapshot(&db);
    assert_eq!(
        rows.iter()
            .filter(|(k, _)| k[0] == 0 && k[1] == b'c')
            .count(),
        3
    );
    assert_eq!(
        rows.iter()
            .filter(|(k, _)| k[0] == 0 && k[1] == b'e')
            .count(),
        2
    );
    // Quota is exactly retained serialized keys+values, including its own record.
    for root in [0, 1] {
        let measured: usize = rows
            .iter()
            .filter(|(k, _)| k[0] == root)
            .map(|(k, v)| k.len() + v.len())
            .sum();
        let quota = rows.iter().find(|(k, _)| k == &[root, b'q']).unwrap();
        assert_eq!(
            u64::from_le_bytes(quota.1.as_slice().try_into().unwrap()),
            measured as u64
        );
    }
}

#[test]
fn inline_boundaries_empty_objects_and_checked_ranges() {
    let db = database();
    let store = Store::open(&db).unwrap();
    for (root, len) in [0, 1, 128, 129, 4096, 4097, MAX_BYTES]
        .into_iter()
        .enumerate()
    {
        let data: Vec<_> = (0..len).map(|i| (i % 251) as u8).collect();
        let head = store.publish(root as u8, 0, &data).unwrap();
        let mut out = vec![0; len];
        assert_eq!(store.read(root as u8, 1, 0, &mut out).unwrap(), head);
        assert_eq!(out, data);
        assert_eq!(
            store.read(root as u8, 1, usize::MAX, &mut [0]),
            Err(Error::Invalid)
        );
        assert_eq!(
            store.read(root as u8, 1, len, &mut [0]),
            Err(Error::Invalid)
        );
        if len > 4096 {
            let mut cross = [0; 2];
            store.read(root as u8, 1, 4095, &mut cross).unwrap();
            assert_eq!(cross, data[4095..4097]);
        }
    }
}

#[test]
fn corruption_of_event_manifest_or_payload_is_detected() {
    for kind in [b'e', b'm', b'c'] {
        let db = database();
        let store = Store::open(&db).unwrap();
        store.publish(0, 0, &content()).unwrap();
        let (key, mut bytes) = snapshot(&db)
            .into_iter()
            .find(|(k, _)| k[1] == kind)
            .unwrap();
        bytes[0] ^= 1;
        let tx = db.begin_write().unwrap();
        tx.open_table(TABLE)
            .unwrap()
            .insert(key.as_slice(), bytes.as_slice())
            .unwrap();
        tx.commit().unwrap();
        assert_eq!(store.read(0, 1, 0, &mut [0; 8192]), Err(Error::Corrupt));
    }
}

#[test]
fn writer_revalidates_revision_after_preparation() {
    let db = database();
    let store = Store::open(&db).unwrap();
    let mut competed = false;
    let result = store.publish_observed(0, 0, b"stale prepared bytes", |stage| {
        if matches!(stage, Stage::Prepared) {
            store.publish(0, 0, b"winner").unwrap();
            competed = true;
        }
    });
    assert!(competed);
    assert_eq!(result, Err(Error::Conflict));
    let mut bytes = [0; 6];
    store.read(0, 1, 0, &mut bytes).unwrap();
    assert_eq!(&bytes, b"winner");
}
