//! Owned immutable preparations; expected revisions are checked inside the writer.
//! A deliberately small record protocol, not Cathedral's DBFS format or authority API.
use super::{Result, device::Device};
use redb::{Database, ReadTransaction, ReadableDatabase, ReadableTable, TableDefinition};
use std::{
    sync::{Arc, Barrier},
    time::Instant,
};

pub const TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("objects");

pub struct Prepared {
    object: u64,
    expected: u64,
    bytes: Vec<u8>,
    digest: [u8; 32],
}

impl Prepared {
    pub fn payload_len(&self) -> usize {
        self.bytes.len()
    }

    pub fn new(object: u64, expected: u64, bytes: Vec<u8>) -> Self {
        // Hashing and ownership acquisition precede any database writer lock.
        let digest = *blake3::hash(&bytes).as_bytes();
        Self {
            object,
            expected,
            bytes,
            digest,
        }
    }
}

pub fn open(backend: impl redb::StorageBackend) -> Result<Database> {
    Ok(Database::builder()
        .set_cache_size(1024 * 1024)
        .create_with_backend(backend)?)
}

pub fn initialize(db: &Database) -> Result<()> {
    let mut tx = db.begin_write()?;
    tx.set_durability(redb::Durability::Immediate)?;
    tx.open_table(TABLE)?;
    tx.commit()?;
    Ok(())
}

pub struct Published {
    pub accepted: bool,
    pub writer_wait_us: u128,
    pub writer_held_us: u128,
}

fn key(kind: u8, object: u64) -> Vec<u8> {
    let mut key = vec![kind];
    key.extend_from_slice(&object.to_be_bytes());
    key
}

pub fn publish(db: &Database, changes: &[Prepared], retire: bool) -> Result<Published> {
    let waiting = Instant::now();
    let mut tx = db.begin_write()?;
    let writer_wait_us = waiting.elapsed().as_micros();
    let holding = Instant::now();
    tx.set_durability(redb::Durability::Immediate)?;
    {
        let mut table = tx.open_table(TABLE)?;
        let mut seen = std::collections::BTreeSet::new();
        // Validate the whole batch before writing any of it.
        for item in changes {
            assert!(seen.insert(item.object), "duplicate object in publication");
            let current = table.get(key(b'h', item.object).as_slice())?;
            let revision = current.as_ref().map_or(0, |v| revision(v.value()));
            if revision != item.expected {
                return Ok(Published {
                    accepted: false,
                    writer_wait_us,
                    writer_held_us: holding.elapsed().as_micros(),
                });
            }
        }
        for item in changes {
            let head = key(b'h', item.object);
            if retire {
                let old = table.get(head.as_slice())?.map(|v| v.value().to_vec());
                if let Some(old) = old {
                    let mut content = vec![b'c'];
                    content.extend_from_slice(&old[8..]);
                    // Reclamation experiment uses one object and unique payloads;
                    // this is not a shared-content collector.
                    table.remove(content.as_slice())?;
                    let mut event = key(b'e', item.object);
                    event.extend_from_slice(&old[..8]);
                    table.remove(event.as_slice())?;
                }
            }
            let mut content = vec![b'c'];
            content.extend_from_slice(&item.digest);
            table.insert(content.as_slice(), item.bytes.as_slice())?;
            let mut value = (item.expected + 1).to_be_bytes().to_vec();
            value.extend_from_slice(&item.digest);
            table.insert(head.as_slice(), value.as_slice())?;
            let mut event = key(b'e', item.object);
            event.extend_from_slice(&value[..8]);
            table.insert(event.as_slice(), value.as_slice())?;
        }
    }
    tx.commit()?;
    Ok(Published {
        accepted: true,
        writer_wait_us,
        writer_held_us: holding.elapsed().as_micros(),
    })
}

fn revision(value: &[u8]) -> u64 {
    assert_eq!(value.len(), 40);
    u64::from_be_bytes(value[..8].try_into().unwrap())
}

pub fn read(tx: &ReadTransaction, object: u64) -> Result<(u64, Vec<u8>)> {
    let table = tx.open_table(TABLE)?;
    let head = table
        .get(key(b'h', object).as_slice())?
        .ok_or("missing head")?;
    let mut content = vec![b'c'];
    content.extend_from_slice(&head.value()[8..]);
    let bytes = table
        .get(content.as_slice())?
        .ok_or("missing content")?
        .value()
        .to_vec();
    assert_eq!(
        blake3::hash(&bytes).as_bytes().as_slice(),
        &head.value()[8..]
    );
    let mut event = key(b'e', object);
    event.extend_from_slice(&head.value()[..8]);
    assert_eq!(
        table.get(event.as_slice())?.ok_or("missing event")?.value(),
        head.value()
    );
    Ok((revision(head.value()), bytes))
}

pub fn records(db: &Database) -> Result<std::collections::BTreeMap<Vec<u8>, Vec<u8>>> {
    let tx = db.begin_read()?;
    let table = tx.open_table(TABLE)?;
    table
        .iter()?
        .map(|entry| {
            let (k, v) = entry?;
            Ok((k.value().to_vec(), v.value().to_vec()))
        })
        .collect()
}

pub fn check_conflict() -> Result<()> {
    let db = Arc::new(open(Device::new(Vec::new()))?);
    initialize(&db)?;
    let barrier = Arc::new(Barrier::new(2));
    let mut workers = Vec::new();
    for value in [1, 2] {
        let db = db.clone();
        let barrier = barrier.clone();
        workers.push(std::thread::spawn(move || -> Result<bool> {
            let prepared = Prepared::new(1, 0, vec![value; 4096]);
            barrier.wait();
            Ok(publish(&db, &[prepared], false)?.accepted)
        }));
    }
    let accepted = workers
        .into_iter()
        .map(|w| w.join().unwrap())
        .collect::<Result<Vec<_>>>()?;
    assert_eq!(accepted.into_iter().filter(|v| *v).count(), 1);
    let (revision, bytes) = read(&db.begin_read()?, 1)?;
    assert_eq!(revision, 1);
    assert!(bytes == vec![1; 4096] || bytes == vec![2; 4096]);
    assert_eq!(
        records(&db)?.len(),
        3,
        "rejected writer left records behind"
    );
    let before = records(&db)?;
    let changes = [
        Prepared::new(2, 0, vec![3; 4096]),
        Prepared::new(1, 0, vec![4; 4096]), // Late conflict must reject both.
    ];
    assert!(!publish(&db, &changes, false)?.accepted);
    assert_eq!(records(&db)?, before);
    println!("VERIFIED: exactly one competing expected-revision writer commits");
    Ok(())
}
