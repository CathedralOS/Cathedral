use super::tests::{collect, snapshot};
use super::*;
use alloc::vec::Vec;
use redb::backends::InMemoryBackend;
/// Exercise redb's actual poison/reopen path, not just an error conversion.
#[test]
fn physical_capacity_before_commit_preserves_heads_and_reopens_for_cleanup() {
    use redb::StorageBackend;
    use std::sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    };
    #[derive(Clone, Debug)]
    struct Bounded {
        bytes: Arc<InMemoryBackend>,
        limit: Arc<AtomicU64>,
    }
    impl StorageBackend for Bounded {
        fn len(&self) -> core::result::Result<u64, redb::io::Error> {
            self.bytes.len()
        }
        fn read(&self, offset: u64, out: &mut [u8]) -> core::result::Result<(), redb::io::Error> {
            self.bytes.read(offset, out)
        }
        fn write(&self, offset: u64, data: &[u8]) -> core::result::Result<(), redb::io::Error> {
            self.bytes.write(offset, data)
        }
        fn sync_data(&self) -> core::result::Result<(), redb::io::Error> {
            self.bytes.sync_data()
        }
        fn set_len(&self, len: u64) -> core::result::Result<(), redb::io::Error> {
            if len > self.limit.load(Ordering::Relaxed) {
                return Err(redb::io::Error::other(
                    cathedral_storage_engine::CAPACITY_REFUSAL,
                ));
            }
            self.bytes.set_len(len)
        }
    }
    let backend = Bounded {
        bytes: Arc::new(InMemoryBackend::new()),
        limit: Arc::new(AtomicU64::new(u64::MAX)),
    };
    let open = || {
        Database::builder()
            .set_cache_size(64 * 1024)
            .create_with_backend(backend.clone())
            .unwrap()
    };
    let mut db = open();
    let anchor = Store::open(&db)
        .unwrap()
        .publish(0, 0, 0, b"saved draft", 0)
        .unwrap();
    // Let the current extent fill. The next requested growth cannot touch disk.
    backend
        .limit
        .store(backend.len().unwrap(), Ordering::Relaxed);
    let mut seed = 1u64;
    let mut objects = Vec::new();
    let before = loop {
        let before = snapshot(&db);
        let data: Vec<u8> = (0..64 * 1024)
            .map(|_| {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                seed as u8
            })
            .collect();
        match Store::open(&db).unwrap().publish(0, 0, 0, &data, 0) {
            Ok(object) => objects.push(object),
            Err(Error::ReopenCapacity) => break before,
            other => panic!("unexpected capacity outcome: {other:?}"),
        }
        assert!(objects.len() < 30);
    };
    // An in-place retry is forbidden; only a new engine can recover its journal.
    assert!(Store::open(&db).is_err());
    drop(db);
    db = open();
    assert_eq!(snapshot(&db), before);
    let store = Store::open(&db).unwrap();
    assert_eq!(store.head(0, anchor.id).unwrap(), anchor.head);
    for object in objects {
        store.delete(0, object.id, object.head.revision).unwrap();
    }
    collect(&store, 0);
    store
        .publish(0, anchor.id, anchor.head.revision, b"retry saved", 0)
        .unwrap();
    assert_eq!(
        engine::<(), _>(Err(redb::io::Error::other("disk IO failure"))),
        Err(Error::Engine)
    );
}
