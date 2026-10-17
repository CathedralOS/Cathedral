//! Independent desktop oracle: construct the two app snapshots without importing
//! the guest catalog codec, then compare the entire private database record set.
use super::*;
use crate::chunks::{Layout, Policy, Store};
use redb::ReadableTable;
use std::collections::BTreeMap;
pub fn verify(path: &Path, offset: u64, bytes: u64, recovery: bool, notes: bool) -> Result<()> {
    let db = open(Region::new(Arc::new(Image::open(path)?), offset, bytes)?)?;
    let tx = db.begin_read()?;
    let table = tx.open_table(TableDefinition::<&[u8], &[u8]>::new("private-objects-v1"))?;
    let mut actual = BTreeMap::new();
    for row in table.iter()? {
        let (key, value) = row?;
        actual.insert(key.value().to_vec(), value.value().to_vec());
    }
    let mut expected = BTreeMap::new();
    for root in [0u8, 1] {
        let mut store = Store::new(Policy {
            chunk_bytes: 4096,
            inline_limit: 128,
            layout: Layout::Flat,
        });
        let last: u64 = if (recovery || notes) && root == 0 { 4 } else { 2 };
        for revision in last - 1..=last {
            let mut snapshot = vec![0u8; 152];
            snapshot[..8].copy_from_slice(b"CTDBCAT1");
            snapshot[8..16].copy_from_slice(&revision.to_le_bytes());
            if notes {
                let full = b"hello cathedral notes abcdefghijklmnopqrstuvwxyz 0123456789abc";
                let text = if root == 0 { &full[..if revision == 3 { 60 } else { 35 }] }
                    else if revision == 1 { b"private sibling note".as_slice() }
                    else { b"private sibling note 2".as_slice() };
                let mut note = b"NTS1".to_vec();
                note.extend_from_slice(text);
                for (object, payload) in [&note[..note.len().min(32)], &note[note.len().min(32)..]].into_iter().enumerate() {
                    let offset = 16 + object * 34;
                    snapshot[offset] = 1;
                    snapshot[offset + 1] = payload.len() as u8;
                    snapshot[offset + 2..offset + 2 + payload.len()].copy_from_slice(payload);
                }
            } else {
                snapshot[50..58].copy_from_slice(&[1, 6, b'M', b'C', b'0', b'1', root, (revision % 2 == 0) as u8]);
            }
            let version = store.put(&snapshot);
            let records = store.records(&version);
            let mut head = revision.to_le_bytes().to_vec();
            head.extend_from_slice(&records[b"head".as_slice()]);
            head.extend_from_slice(&152u64.to_le_bytes());
            for (key, value) in records {
                if key == b"head" {
                    continue;
                }
                let mut rooted = vec![root];
                rooted.extend_from_slice(&key);
                expected.insert(rooted, value);
            }
            let mut event = vec![root, b'e'];
            event.extend_from_slice(&revision.to_be_bytes());
            expected.insert(event, head.clone());
            expected.insert(vec![root, b'h'], head);
        }
        let charge = expected
            .iter()
            .filter(|(k, _)| k[0] == root)
            .map(|(k, v)| k.len() + v.len())
            .sum::<usize>()
            + 10;
        expected.insert(vec![root, b'q'], (charge as u64).to_le_bytes().to_vec());
    }
    if actual != expected {
        return Err("desktop records differ from independent private catalog oracle".into());
    }
    println!(
        "VERIFIED: two private desktop catalogs, recovery={recovery}, notes={notes}, exact snapshots/content/history/quota ({} records)",
        actual.len()
    );
    Ok(())
}
