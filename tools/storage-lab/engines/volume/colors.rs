//! Independent oracle for the probe desktop image: each root holds one object
//! at its top, whose payload alternates colors across revisions. Collection
//! runs before every write, so exactly the last two revisions' events and
//! manifests remain.
use super::*;
use crate::chunks::{Layout, Policy, Store};
use redb::ReadableTable;
use std::collections::BTreeMap;

pub(super) fn records(table: &impl ReadableTable<&'static [u8], &'static [u8]>) -> Result<BTreeMap<Vec<u8>, Vec<u8>>> {
    let mut actual = BTreeMap::new();
    for row in table.iter()? {
        let (key, value) = row?;
        actual.insert(key.value().to_vec(), value.value().to_vec());
    }
    Ok(actual)
}
/// A head: revision, manifest digest, length and parent, little endian.
pub(super) fn head(revision: u64, manifest: &[u8], len: usize, parent: u64) -> Vec<u8> {
    let mut head = revision.to_le_bytes().to_vec();
    head.extend_from_slice(manifest);
    head.extend_from_slice(&(len as u64).to_le_bytes());
    head.extend_from_slice(&parent.to_le_bytes());
    head
}
/// One object numbered one at the top of the root, with `payload(revision)`
/// for the last two revisions retained and everything older collected.
pub(super) fn expect(expected: &mut BTreeMap<Vec<u8>, Vec<u8>>, root: u8, last: u64, payload: impl Fn(u64) -> Vec<u8>) {
    let mut store = Store::new(Policy {
        chunk_bytes: 4096,
        inline_limit: 128,
        layout: Layout::Flat,
    });
    let key = |kind: u8, suffix: &[u8]| {
        let mut key = vec![root, kind];
        key.extend_from_slice(suffix);
        key
    };
    for revision in last.saturating_sub(1).max(1)..=last {
        let bytes = payload(revision);
        let version = store.put(&bytes);
        let records = store.records(&version);
        let head = head(revision, &records[b"head".as_slice()], bytes.len(), 0);
        for (k, v) in records {
            if k == b"head" {
                continue;
            }
            if k[0] == b'c' {
                expected.insert(key(b'k', &k[1..]), Vec::new());
            }
            expected.insert(key(k[0], &k[1..]), v);
        }
        let mut suffix = 1u64.to_be_bytes().to_vec();
        suffix.extend_from_slice(&revision.to_be_bytes());
        expected.insert(key(b'e', &suffix), head.clone());
        if revision == last {
            expected.insert(key(b'h', &1u64.to_be_bytes()), head);
        }
    }
    let mut edge = 0u64.to_be_bytes().to_vec();
    edge.extend_from_slice(&1u64.to_be_bytes());
    expected.insert(key(b't', &edge), Vec::new());
    expected.insert(key(b'n', &[]), 2u64.to_le_bytes().to_vec());
    let charge = expected
        .iter()
        .filter(|(k, _)| k[0] == root)
        .map(|(k, v)| k.len() + v.len())
        .sum::<usize>()
        + 10;
    expected.insert(key(b'q', &[]), (charge as u64).to_le_bytes().to_vec());
}
pub fn verify(path: &Path, offset: u64, bytes: u64, last: [u64; 2]) -> Result<()> {
    let db = open(Region::new(Arc::new(Image::open(path)?), offset, bytes)?)?;
    let tx = db.begin_read()?;
    let table = tx.open_table(TableDefinition::<&[u8], &[u8]>::new("private-objects-v3"))?;
    let actual = records(&table)?;
    let mut expected = BTreeMap::new();
    for root in 0..2u8 {
        expect(&mut expected, root, last[root as usize], |revision| {
            vec![b'M', b'C', b'0', b'1', root, u8::from(revision % 2 == 0)]
        });
    }
    if actual != expected {
        difference::report(&actual, &expected);
        return Err("desktop records differ from the independent object oracle".into());
    }
    println!(
        "VERIFIED: two private color objects at revisions {:?}, exact heads/history/quota ({} records)",
        last,
        actual.len()
    );
    Ok(())
}
