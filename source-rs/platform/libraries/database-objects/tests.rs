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
fn count(db: &Database, root: u8, kind: u8) -> usize {
    snapshot(db)
        .iter()
        .filter(|(k, _)| k[0] == root && k[1] == kind)
        .count()
}
fn content() -> Vec<u8> {
    (0..8192)
        .map(|i| ((i * 17 + i / 4096 * 71) % 251) as u8)
        .collect()
}
fn collect(store: &Store<'_>, root: u8) {
    for _ in 0..100 {
        if !store.collect(root).unwrap().more {
            return;
        }
    }
    panic!("collection failed to converge");
}
/// Quota is exactly retained serialized keys+values, including its own record.
fn accounted(db: &Database, root: u8) {
    let rows = snapshot(db);
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
    assert!(measured as u64 <= ROOT_QUOTA);
}

#[test]
fn refusals_abort_every_record_and_other_roots_stay_private() {
    let db = database();
    let store = Store::open(&db).unwrap();
    let bytes = content();
    let first = store.publish(0, 0, 0, &bytes, 0).unwrap();
    assert_eq!(first.id, 1);
    assert_eq!(first.head.revision, 1);
    assert_eq!(store.head(1, 1), Err(Error::Missing));
    assert_eq!(store.list(1, 0, 0, &mut [0; 4]).unwrap(), 0);
    let before = snapshot(&db);
    assert_eq!(store.publish(0, 1, 0, &bytes, 0), Err(Error::Conflict));
    assert_eq!(store.publish(0, 0, 1, &bytes, 0), Err(Error::Invalid));
    assert_eq!(store.publish(0, 2, 0, &bytes, 0), Err(Error::Missing));
    assert_eq!(
        store.publish(0, 1, 1, &vec![0; MAX_BYTES + 1], 0),
        Err(Error::TooLarge)
    );
    // Refuse after staging multiple new chunks, not merely before opening a tx.
    let large: Vec<_> = (0..MAX_BYTES).map(|i| (200 + i / 4096) as u8).collect();
    assert_eq!(store.publish(0, 1, 1, &large, 0), Err(Error::Capacity));
    assert_eq!(snapshot(&db), before);
    let mut read = vec![0; bytes.len()];
    assert_eq!(store.read(0, 1, 1, 0, &mut read).unwrap(), first.head);
    assert_eq!(read, bytes);
    let mut replacement = bytes.clone();
    replacement[41] ^= 128;
    assert_eq!(
        store
            .publish(0, 1, 1, &replacement, 0)
            .unwrap()
            .head
            .revision,
        2
    );
    assert_eq!(store.read(0, 1, 1, 0, &mut read), Err(Error::Conflict));
    store.read(0, 1, 0, 0, &mut read).unwrap();
    assert_eq!(read, replacement);
    store.publish(1, 0, 0, b"separate", 0).unwrap();
    assert_eq!(count(&db, 0, b'c'), 3); // One changed chunk, one shared.
    assert_eq!(count(&db, 0, b'e'), 2);
    accounted(&db, 0);
    accounted(&db, 1);
}

#[test]
fn a_copy_shares_every_chunk_and_costs_only_its_head_and_event() {
    let db = database();
    let store = Store::open(&db).unwrap();
    let bytes = content();
    let source = store.publish(0, 0, 0, &bytes, 0).unwrap();
    let before = snapshot(&db);
    let copy = store.copy(0, source.id, 1, 0).unwrap();
    assert_eq!(copy.id, 2);
    assert_eq!(copy.head.revision, 1);
    assert_eq!(copy.head.manifest, source.head.manifest);
    let after = snapshot(&db);
    let added: Vec<_> = after.iter().filter(|row| !before.contains(row)).collect();
    // Three small records plus the rewritten number/quota ledgers; no content.
    assert_eq!(added.len(), 5);
    assert!(
        added
            .iter()
            .all(|(k, _)| matches!(k[1], b'h' | b'e' | b't' | b'n' | b'q'))
    );
    assert_eq!(count(&db, 0, b'c'), 2);
    assert_eq!(count(&db, 0, b'm'), 1);
    let mut read = vec![0; bytes.len()];
    store.read(0, copy.id, 1, 0, &mut read).unwrap();
    assert_eq!(read, bytes);
    assert_eq!(store.copy(0, source.id, 2, 0), Err(Error::Conflict));
    assert_eq!(store.copy(0, 9, 1, 0), Err(Error::Missing));
    // Editing the copy leaves the original untouched and still shares one chunk.
    let mut edited = bytes.clone();
    edited[8000] ^= 1;
    store.publish(0, copy.id, 1, &edited, 0).unwrap();
    store.read(0, source.id, 1, 0, &mut read).unwrap();
    assert_eq!(read, bytes);
    assert_eq!(count(&db, 0, b'c'), 3);
    accounted(&db, 0);
    let mut ids = [0; 4];
    assert_eq!(store.list(0, 0, 0, &mut ids).unwrap(), 2);
    assert_eq!(ids[..2], [1, 2]);
    assert_eq!(store.list(0, 0, 1, &mut ids).unwrap(), 1);
    assert_eq!(ids[0], 2);
}

#[test]
fn deleted_objects_release_only_unshared_content_and_numbers_are_never_reused() {
    let db = database();
    let store = Store::open(&db).unwrap();
    let bytes = content();
    let a = store.publish(0, 0, 0, &bytes, 0).unwrap();
    let b = store.copy(0, a.id, 1, 0).unwrap();
    let c = store.publish(0, 0, 0, b"tiny", 0).unwrap();
    assert_eq!((a.id, b.id, c.id), (1, 2, 3));
    assert_eq!(store.delete(0, a.id, 2), Err(Error::Conflict));
    store.delete(0, a.id, 1).unwrap();
    assert_eq!(store.head(0, a.id), Err(Error::Missing));
    assert_eq!(store.read(0, a.id, 0, 0, &mut []), Err(Error::Missing));
    collect(&store, 0);
    // The copy still holds the content; only the deleted object's event went.
    assert_eq!(count(&db, 0, b'c'), 2);
    assert_eq!(count(&db, 0, b'e'), 2);
    let mut read = vec![0; bytes.len()];
    store.read(0, b.id, 1, 0, &mut read).unwrap();
    assert_eq!(read, bytes);
    store.delete(0, b.id, 1).unwrap();
    collect(&store, 0);
    assert_eq!(count(&db, 0, b'c'), 0);
    assert_eq!(count(&db, 0, b'm'), 1);
    let d = store.publish(0, 0, 0, b"later", 0).unwrap();
    assert_eq!(d.id, 4);
    let mut ids = [0; 4];
    assert_eq!(store.list(0, 0, 0, &mut ids).unwrap(), 2);
    assert_eq!(ids[..2], [3, 4]);
    accounted(&db, 0);
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
        let object = store.publish(root as u8, 0, 0, &data, 0).unwrap();
        let mut out = vec![0; len];
        assert_eq!(
            store.read(root as u8, object.id, 1, 0, &mut out).unwrap(),
            object.head
        );
        assert_eq!(out, data);
        assert_eq!(
            store.read(root as u8, object.id, 1, usize::MAX, &mut [0]),
            Err(Error::Invalid)
        );
        assert_eq!(
            store.read(root as u8, object.id, 1, len, &mut [0]),
            Err(Error::Invalid)
        );
        if len > 4096 {
            let mut cross = [0; 2];
            store
                .read(root as u8, object.id, 1, 4095, &mut cross)
                .unwrap();
            assert_eq!(cross, data[4095..4097]);
        }
    }
}

#[test]
fn corruption_of_event_manifest_or_payload_is_detected() {
    for kind in [b'e', b'm', b'c'] {
        let db = database();
        let store = Store::open(&db).unwrap();
        store.publish(0, 0, 0, &content(), 0).unwrap();
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
        assert_eq!(store.read(0, 1, 1, 0, &mut [0; 8192]), Err(Error::Corrupt));
    }
}

#[test]
fn writer_revalidates_revision_after_preparation() {
    let db = database();
    let store = Store::open(&db).unwrap();
    store.publish(0, 0, 0, b"first", 0).unwrap();
    let mut competed = false;
    let result = store.publish_observed(0, 1, 1, b"stale prepared bytes", 0, |stage| {
        if matches!(stage, Stage::Prepared) {
            store.publish(0, 1, 1, b"winner", 0).unwrap();
            competed = true;
        }
    });
    assert!(competed);
    assert_eq!(result, Err(Error::Conflict));
    let mut bytes = [0; 6];
    store.read(0, 1, 2, 0, &mut bytes).unwrap();
    assert_eq!(&bytes, b"winner");
}

#[test]
fn subtrees_are_listed_scoped_copied_whole_and_deleted_leaf_first() {
    let db = database();
    let store = Store::open(&db).unwrap();
    let bytes = content();
    // world(1) / { note(2), folder(3) / { inner(4) } }, plus a sibling world(5).
    let world = store.publish(0, 0, 0, b"", 0).unwrap();
    let note = store.publish(0, 0, 0, &bytes, world.id).unwrap();
    let folder = store.publish(0, 0, 0, b"", world.id).unwrap();
    let inner = store.copy(0, note.id, 1, folder.id).unwrap();
    let other = store.publish(0, 0, 0, b"elsewhere", 0).unwrap();
    assert_eq!(
        (world.id, note.id, folder.id, inner.id, other.id),
        (1, 2, 3, 4, 5)
    );
    assert_eq!(store.head(0, inner.id).unwrap().parent, folder.id);
    assert_eq!(store.publish(0, 0, 0, b"", 9), Err(Error::Missing));
    assert_eq!(
        store.publish(0, note.id, 1, b"moved?", folder.id),
        Err(Error::Invalid)
    );
    let mut ids = [0; 4];
    assert_eq!(store.list(0, 0, 0, &mut ids).unwrap(), 2);
    assert_eq!(ids[..2], [world.id, other.id]);
    assert_eq!(store.list(0, world.id, 0, &mut ids).unwrap(), 2);
    assert_eq!(ids[..2], [note.id, folder.id]);
    assert_eq!(store.list(0, world.id, note.id, &mut ids).unwrap(), 1);
    assert_eq!(ids[0], folder.id);
    assert_eq!(store.list(0, folder.id, 0, &mut ids).unwrap(), 1);
    assert_eq!(store.list(0, 9, 0, &mut ids), Err(Error::Missing));
    for (object, scope, expected) in [
        (inner.id, world.id, true),
        (inner.id, folder.id, true),
        (note.id, folder.id, false),
        (other.id, world.id, false),
        (world.id, world.id, true),
        (0, world.id, false),
        (other.id, 0, true),
    ] {
        assert_eq!(store.within(0, object, scope).unwrap(), expected);
    }
    let before = snapshot(&db);
    let copied = store.copy_tree(0, world.id, 1, 0).unwrap();
    assert_eq!(copied.id, 6);
    assert_eq!(copied.head.parent, 0);
    let added = snapshot(&db)
        .iter()
        .filter(|row| !before.contains(row))
        .count();
    assert_eq!(added, 4 * 3 + 2); // Four objects, three records each, two ledgers.
    assert_eq!(count(&db, 0, b'c'), 2);
    assert_eq!(count(&db, 0, b'm'), 3); // Empty, the note text and "elsewhere".
    assert_eq!(store.list(0, copied.id, 0, &mut ids).unwrap(), 2);
    let [copied_note, copied_folder] = [ids[0], ids[1]];
    assert_eq!(store.list(0, copied_folder, 0, &mut ids).unwrap(), 1);
    assert_eq!(store.head(0, ids[0]).unwrap().manifest, note.head.manifest);
    assert_eq!(store.head(0, copied_note).unwrap().parent, copied.id);
    assert!(store.within(0, ids[0], copied.id).unwrap());
    assert!(!store.within(0, ids[0], world.id).unwrap());
    // Editing the original afterwards leaves the whole copied world alone.
    let mut edited = bytes.clone();
    edited[100] ^= 1;
    store.publish(0, note.id, 1, &edited, 0).unwrap();
    let mut read = vec![0; bytes.len()];
    store.read(0, copied_note, 1, 0, &mut read).unwrap();
    assert_eq!(read, bytes);
    assert_eq!(store.copy_tree(0, world.id, 2, 0), Err(Error::Conflict));
    assert_eq!(
        store.copy_tree(0, world.id, 1, folder.id),
        Err(Error::Invalid)
    );
    assert_eq!(
        store.copy_tree(0, world.id, 1, world.id),
        Err(Error::Invalid)
    );
    assert_eq!(store.delete(0, world.id, 1), Err(Error::NotEmpty));
    store.delete(0, inner.id, 1).unwrap();
    store.delete(0, folder.id, 1).unwrap();
    assert_eq!(store.list(0, world.id, 0, &mut ids).unwrap(), 1);
    accounted(&db, 0);
}

#[test]
fn a_tree_copy_is_refused_whole_beyond_the_object_limit() {
    let db = database();
    let store = Store::open(&db).unwrap();
    let world = store.publish(0, 0, 0, b"", 0).unwrap();
    for _ in 1..COPY_LIMIT {
        store.publish(0, 0, 0, b"", world.id).unwrap();
    }
    let before = snapshot(&db);
    assert_eq!(
        store.copy_tree(0, world.id, 1, 0).unwrap().id,
        COPY_LIMIT as u64 + 1
    );
    store.publish(0, 0, 0, b"", world.id).unwrap();
    let before_refusal = snapshot(&db);
    assert_eq!(store.copy_tree(0, world.id, 1, 0), Err(Error::Capacity));
    assert_eq!(snapshot(&db), before_refusal);
    assert!(before.len() < before_refusal.len());
    accounted(&db, 0);
}
