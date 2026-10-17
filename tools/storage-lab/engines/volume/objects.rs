//! Independent oracle: the host layout lab constructs the expected retained
//! record set. Compare every byte/key, including history and absence of aborted
//! chunks; don't let the guest's reader implementation verify itself.
use super::*;
use crate::chunks::{Layout, Policy, Store};
use redb::ReadableTable;
use std::collections::BTreeMap;
pub fn verify(path: &Path, offset: u64, bytes: u64, revision: u64) -> Result<()> {
    if !(1..=2).contains(&revision) {
        return Err("expected revision 1 or 2".into());
    }
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
        let bytes: Vec<_> = (0..8192)
            .map(|i| ((i * 17 + i / 4096 * 71 + root as usize * 43) % 251) as u8)
            .collect();
        let mut version = store.put(&bytes);
        for n in 1..=revision {
            if n == 2 {
                version = store.patch(&version, 41, &[bytes[41] ^ 128]).unwrap();
            }
            let records = store.records(&version);
            let mut head = n.to_le_bytes().to_vec();
            head.extend_from_slice(&records[b"head".as_slice()]);
            head.extend_from_slice(&8192u64.to_le_bytes());
            for (key, value) in records {
                if key == b"head" {
                    continue;
                }
                let mut rooted = vec![root];
                rooted.extend_from_slice(&key);
                expected.insert(rooted, value);
            }
            let mut event = vec![root, b'e'];
            event.extend_from_slice(&n.to_be_bytes());
            expected.insert(event, head.clone());
            expected.insert(vec![root, b'h'], head);
        }
        let charge: usize = expected
            .iter()
            .filter(|(k, _)| k[0] == root)
            .map(|(k, v)| k.len() + v.len())
            .sum::<usize>()
            + 2
            + 8;
        expected.insert(vec![root, b'q'], (charge as u64).to_le_bytes().to_vec());
    }
    if actual != expected {
        return Err(
            "guest private records differ from independent host content/history/quota oracle"
                .into(),
        );
    }
    println!(
        "VERIFIED: both private roots, real content, revision {revision}, exact retained history and no aborted records ({} records)",
        actual.len()
    );
    Ok(())
}
