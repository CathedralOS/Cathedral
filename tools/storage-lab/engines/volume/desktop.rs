//! Independent oracle for the default desktop image after its interactive test:
//! construct every expected record without importing the guest store, then
//! compare the whole retained set. The notes root holds two worlds. The first
//! was made at startup and holds a note and its edited copy; the second is a
//! subtree copy of the first, whose second note was then edited once more.
//! The ungranted root holds nothing.
use super::*;
use crate::chunks::{Layout, Policy, Store};
use redb::ReadableTable;
use std::collections::BTreeMap;

const ROOT: u8 = 0;

fn key(kind: u8, suffix: &[u8]) -> Vec<u8> {
    let mut key = vec![ROOT, kind];
    key.extend_from_slice(suffix);
    key
}
fn head(revision: u64, manifest: &[u8], len: usize, parent: u64) -> Vec<u8> {
    let mut head = revision.to_le_bytes().to_vec();
    head.extend_from_slice(manifest);
    head.extend_from_slice(&(len as u64).to_le_bytes());
    head.extend_from_slice(&parent.to_le_bytes());
    head
}
fn event(object: u64, revision: u64) -> Vec<u8> {
    let mut suffix = object.to_be_bytes().to_vec();
    suffix.extend_from_slice(&revision.to_be_bytes());
    key(b'e', &suffix)
}
fn edge(parent: u64, child: u64) -> Vec<u8> {
    let mut suffix = parent.to_be_bytes().to_vec();
    suffix.extend_from_slice(&child.to_be_bytes());
    key(b't', &suffix)
}
pub fn verify(path: &Path, offset: u64, bytes: u64) -> Result<()> {
    let db = open(Region::new(Arc::new(Image::open(path)?), offset, bytes)?)?;
    let tx = db.begin_read()?;
    let table = tx.open_table(TableDefinition::<&[u8], &[u8]>::new("private-objects-v3"))?;
    let mut actual = BTreeMap::new();
    for row in table.iter()? {
        let (key, value) = row?;
        actual.insert(key.value().to_vec(), value.value().to_vec());
    }
    let mut store = Store::new(Policy {
        chunk_bytes: 4096,
        inline_limit: 128,
        layout: Layout::Flat,
    });
    let mut expected = BTreeMap::new();
    // Content is shared within the root: four distinct texts, however many
    // notes point at them. Collection runs before each write, so only events
    // that were still heads at the last write survive; the clone's edited note
    // was the last write, so its first event is still present.
    let empty = store.put(b"");
    let original = store.put(b"NTS1hello cathedral");
    let edited = store.put(b"NTS1hello cathedral two");
    let cloned = store.put(b"NTS1hello cathedral two clone");
    let mut manifests = Vec::new();
    for version in [&empty, &original, &edited, &cloned] {
        for (k, v) in store.records(version) {
            if k == b"head" {
                manifests.push(v);
                continue;
            }
            expected.insert(key(k[0], &k[1..]), v);
        }
    }
    let text = |manifest: usize, revision: u64, parent: u64| {
        let len = [0, 19, 23, 29][manifest];
        head(revision, &manifests[manifest], len, parent)
    };
    // Objects: world 1; its notes 2 (saved once) and 3 (copied, then edited);
    // world 4 (the clone); its notes 5 (copy of 2) and 6 (copy of 3, edited).
    let objects: [(u64, Vec<u8>, Vec<Vec<u8>>); 6] = [
        (1, text(0, 1, 0), vec![text(0, 1, 0)]),
        (2, text(1, 1, 1), vec![text(1, 1, 1)]),
        (3, text(2, 2, 1), vec![text(2, 2, 1)]),
        (4, text(0, 1, 0), vec![text(0, 1, 0)]),
        (5, text(1, 1, 4), vec![text(1, 1, 4)]),
        (6, text(3, 2, 4), vec![text(2, 1, 4), text(3, 2, 4)]),
    ];
    for (object, current, history) in objects {
        let parent = u64::from_le_bytes(current[48..].try_into().unwrap());
        expected.insert(key(b'h', &object.to_be_bytes()), current);
        expected.insert(edge(parent, object), Vec::new());
        for head in history {
            let revision = u64::from_le_bytes(head[..8].try_into().unwrap());
            expected.insert(event(object, revision), head);
        }
    }
    expected.insert(key(b'n', &[]), 7u64.to_le_bytes().to_vec());
    let charge = expected.iter().map(|(k, v)| k.len() + v.len()).sum::<usize>() + 10;
    expected.insert(key(b'q', &[]), (charge as u64).to_le_bytes().to_vec());
    if actual != expected {
        difference::report(&actual, &expected);
        return Err("desktop records differ from the independent object oracle".into());
    }
    println!(
        "VERIFIED: two worlds of two notes each sharing content, exact heads/history/edges/quota, empty second root ({} records)",
        actual.len()
    );
    Ok(())
}
