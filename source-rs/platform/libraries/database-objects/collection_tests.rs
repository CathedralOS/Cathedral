use super::*;
use alloc::{vec, vec::Vec};
use redb::{ReadableDatabase, ReadableTable, StorageBackend};
use std::{
    io,
    sync::{Arc, Mutex},
};
#[derive(Clone, Debug)]
struct Disk(Arc<Mutex<Vec<u8>>>);
const CAPACITY: usize = 1_056_768;
impl StorageBackend for Disk {
    fn len(&self) -> io::Result<u64> {
        Ok(self.0.lock().unwrap().len() as u64)
    }
    fn set_len(&self, len: u64) -> io::Result<()> {
        if len > CAPACITY as u64 {
            return Err(io::Error::other("fixed capacity"));
        }
        self.0.lock().unwrap().resize(len as usize, 0);
        Ok(())
    }
    fn read(&self, offset: u64, out: &mut [u8]) -> io::Result<()> {
        out.copy_from_slice(&self.0.lock().unwrap()[offset as usize..offset as usize + out.len()]);
        Ok(())
    }
    fn write(&self, offset: u64, bytes: &[u8]) -> io::Result<()> {
        self.0.lock().unwrap()[offset as usize..offset as usize + bytes.len()]
            .copy_from_slice(bytes);
        Ok(())
    }
    fn sync_data(&self) -> io::Result<()> {
        Ok(())
    }
}
fn open(disk: Disk) -> Database {
    Database::builder()
        .set_cache_size(32768)
        .create_with_backend(disk)
        .unwrap()
}
fn collect(store: &Store<'_>, root: u8) {
    for _ in 0..100 {
        if !store.collect(root).unwrap().more {
            return;
        }
    }
    panic!("collection failed to converge");
}
fn accounting(db: &Database) {
    let tx = db.begin_read().unwrap();
    let table = tx.open_table(TABLE).unwrap();
    let mut charged = 0;
    let mut actual = 0;
    for row in table.iter().unwrap() {
        let (key, value) = row.unwrap();
        if key.value()[0] != 0 {
            continue;
        }
        actual += key.value().len() + value.value().len();
        if key.value() == [0, b'q'] {
            charged = u64::from_le_bytes(value.value().try_into().unwrap());
        }
    }
    assert_eq!(charged, actual as u64);
    assert!(charged <= ROOT_QUOTA);
}
#[test]
fn every_collection_boundary_cold_opens_with_live_pins_and_exact_accounting() {
    let disk = Disk(Arc::new(Mutex::new(Vec::new())));
    let db = open(disk.clone());
    let store = Store::open(&db).unwrap();
    for revision in 1..=40u64 {
        store
            .publish(
                0,
                u64::from(revision > 1),
                revision - 1,
                &revision.to_le_bytes(),
                0,
            )
            .unwrap();
        if revision == 1 {
            store.pin(0, 1, 1, 0).unwrap();
        }
    }
    // A second object shares nothing with the pinned history but must survive it.
    let other = store.publish(0, 0, 0, b"other object", 0).unwrap();
    assert_eq!(store.pin(0, 1, 40, 0), Err(Error::Conflict));
    assert_eq!(store.pin(1, 1, 1, 0), Err(Error::Conflict));
    assert_eq!(store.release(0, 1, 2, 0), Err(Error::Conflict));
    assert_eq!(store.release(0, other.id, 1, 0), Err(Error::Conflict));
    let mut passes = 0;
    loop {
        let result = store.collect(0).unwrap();
        passes += 1;
        assert!(result.removed <= 8);
        // Copy before clean close: the original engine remains live and dirty.
        let recovered = open(Disk(Arc::new(Mutex::new(disk.0.lock().unwrap().clone()))));
        let reader = Store::open(&recovered).unwrap();
        let mut out = [0; 8];
        reader.read_pinned(0, 0, 1, 1, 0, &mut out).unwrap();
        assert_eq!(out, 1u64.to_le_bytes());
        reader.read(0, 1, 40, 0, &mut out).unwrap();
        assert_eq!(out, 40u64.to_le_bytes());
        let mut text = [0; 12];
        reader.read(0, other.id, 1, 0, &mut text).unwrap();
        assert_eq!(&text, b"other object");
        accounting(&recovered);
        // Every still-discoverable event remains pinnable between passes.
        for revision in 2..40 {
            match reader.pin(0, 1, revision, 1) {
                Ok(_) => {
                    reader.read_pinned(0, 1, 1, revision, 0, &mut out).unwrap();
                    assert_eq!(out, revision.to_le_bytes());
                    reader.release(0, 1, revision, 1).unwrap();
                }
                Err(Error::Conflict) => {} // Already retired by this pass.
                other => panic!("retained event has invalid closure: {other:?}"),
            }
        }
        if !result.more {
            break;
        }
    }
    assert!(passes > 1);
    // A pinned revision outlives its object's deletion until released.
    store.delete(0, 1, 40).unwrap();
    collect(&store, 0);
    let mut out = [0; 8];
    store.read_pinned(0, 0, 1, 1, 0, &mut out).unwrap();
    assert_eq!(out, 1u64.to_le_bytes());
    store.release(0, 1, 1, 0).unwrap();
    collect(&store, 0);
    assert_eq!(store.pin(0, 1, 1, 0), Err(Error::Conflict));
    assert_eq!(
        store.read_pinned(0, 0, 1, 1, 0, &mut [0; 8]),
        Err(Error::Conflict)
    );
    accounting(&db);
}
#[test]
fn collected_payload_pages_are_reused_with_unrelated_data_in_a_fixed_extent() {
    let disk = Disk(Arc::new(Mutex::new(Vec::new())));
    let db = open(disk.clone());
    let tx = db.begin_write().unwrap();
    {
        let mut table = tx
            .open_table(TableDefinition::<u64, &[u8]>::new("unrelated-resident"))
            .unwrap();
        for key in 0..100 {
            table.insert(key, [77u8; 4096].as_slice()).unwrap();
        }
    }
    tx.commit().unwrap();
    let store = Store::open(&db).unwrap();
    let mut data = vec![0; 8192];
    for revision in 1..=600u64 {
        for (index, byte) in data.iter_mut().enumerate() {
            *byte = ((index * 17 + index / 4096 * 71 + revision as usize * 13) % 251) as u8;
        }
        store
            .publish(0, u64::from(revision > 1), revision - 1, &data, 0)
            .unwrap();
        collect(&store, 0);
    }
    assert!(disk.len().unwrap() <= CAPACITY as u64);
    accounting(&db);
    let recovered = open(Disk(Arc::new(Mutex::new(disk.0.lock().unwrap().clone()))));
    let mut out = vec![0; 8192];
    Store::open(&recovered)
        .unwrap()
        .read(0, 1, 600, 0, &mut out)
        .unwrap();
    assert_eq!(out, data);
    let tx = recovered.begin_read().unwrap();
    let table = tx
        .open_table(TableDefinition::<u64, &[u8]>::new("unrelated-resident"))
        .unwrap();
    for key in 0..100 {
        assert!(
            table
                .get(key)
                .unwrap()
                .unwrap()
                .value()
                .iter()
                .all(|b| *b == 77)
        );
    }
}
