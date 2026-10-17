//! Independent oracle for the default desktop image after its interactive test:
//! construct every expected record without importing the guest store, then
//! compare the whole retained set. Notes' root holds two objects at its top
//! whose first revisions share one manifest; the ungranted root holds nothing.
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
    // Note one is saved once; note two is copied from it, then edited and saved.
    // Collection runs before each write, so the copy's first event is still
    // present: nothing retired it after the final publication.
    let original = store.put(b"NTS1hello cathedral");
    let edited = store.put(b"NTS1hello cathedral two");
    let mut manifests = Vec::new();
    for version in [&original, &edited] {
        for (k, v) in store.records(version) {
            if k == b"head" {
                manifests.push(v);
                continue;
            }
            expected.insert(key(k[0], &k[1..]), v);
        }
    }
    let first = colors::head(1, &manifests[0], 4 + "hello cathedral".len(), 0);
    let copied = colors::head(1, &manifests[0], 4 + "hello cathedral".len(), 0);
    let second = colors::head(2, &manifests[1], 4 + "hello cathedral two".len(), 0);
    expected.insert(key(b'h', &1u64.to_be_bytes()), first.clone());
    expected.insert(key(b'h', &2u64.to_be_bytes()), second.clone());
    expected.insert(edge(0, 1), Vec::new());
    expected.insert(edge(0, 2), Vec::new());
    expected.insert(event(1, 1), first);
    expected.insert(event(2, 1), copied);
    expected.insert(event(2, 2), second);
    expected.insert(key(b'n', &[]), 3u64.to_le_bytes().to_vec());
    let charge = expected.iter().map(|(k, v)| k.len() + v.len()).sum::<usize>() + 10;
    expected.insert(key(b'q', &[]), (charge as u64).to_le_bytes().to_vec());
    if actual != expected {
        difference::report(&actual, &expected);
        return Err("desktop records differ from the independent object oracle".into());
    }
    println!(
        "VERIFIED: two notes sharing one manifest, exact heads/history/edges/quota, empty second root ({} records)",
        actual.len()
    );
    Ok(())
}
