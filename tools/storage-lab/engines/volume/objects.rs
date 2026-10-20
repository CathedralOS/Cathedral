//! Independent oracle: the host layout lab constructs the expected retained
//! record set. Compare every byte/key, including history and absence of aborted
//! chunks; don't let the guest's reader implementation verify itself.
use super::*;
use crate::chunks::{Layout, Policy, Store};
use redb::ReadableTable;
use std::collections::BTreeMap;
pub fn verify(path: &Path, offset: u64, bytes: u64, revision: u64, retention: bool) -> Result<()> {
    if !(if retention {
        [9, 25].contains(&revision)
    } else {
        (1..=2).contains(&revision)
    }) {
        return Err("invalid fixture revision".into());
    }
    let db = open(Region::new(Arc::new(Image::open(path)?), offset, bytes)?)?;
    let tx = db.begin_read()?;
    let table = tx.open_table(TableDefinition::<&[u8], &[u8]>::new("private-objects-v3"))?;
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
        let mut versions = Vec::new();
        let mut heads = Vec::new();
        for n in 1..=revision {
            let mut bytes: Vec<_> = (0..8192)
                .map(|i| {
                    ((i * 17
                        + i / 4096 * 71
                        + root as usize * 43
                        + if retention && n >= 10 {
                            n as usize * 13
                        } else {
                            0
                        })
                        % 251) as u8
                })
                .collect();
            if retention && n > 1 && n < 10 {
                bytes[41] ^= n as u8;
            }
            if !retention && n == 2 {
                bytes[41] ^= 128;
            }
            let version = store.put(&bytes);
            let records = store.records(&version);
            heads.push(colors::head(n, &records[b"head".as_slice()], 8192, 0));
            versions.push(version);
        }
        // Retention explicitly collects. The small publication fixture stays
        // below automatic maintenance pressure and retains its old revision.
        let pinned = retention && revision == 9;
        let mut roots = vec![versions.last().unwrap().clone()];
        if pinned {
            roots.push(versions[0].clone());
        }
        if !retention {
            roots = versions.clone();
        }
        store.retain(&roots);
        for (key, value) in store.records(versions.last().unwrap()) {
            if key == b"head" {
                continue;
            }
            if key[0] == b'c' {
                let mut index = vec![root, b'k'];
                index.extend_from_slice(&key[1..]);
                expected.insert(index, Vec::new());
            }
            let mut rooted = vec![root];
            rooted.extend_from_slice(&key);
            expected.insert(rooted, value);
        }
        for (i, head) in heads.iter().enumerate() {
            let n = i as u64 + 1;
            if !retention || n == revision || (pinned && n == 1) {
                let mut event = vec![root, b'e'];
                event.extend_from_slice(&1u64.to_be_bytes());
                event.extend_from_slice(&n.to_be_bytes());
                expected.insert(event, head.clone());
            }
        }
        let mut head_key = vec![root, b'h'];
        head_key.extend_from_slice(&1u64.to_be_bytes());
        expected.insert(head_key, heads.last().unwrap().clone());
        let mut edge = vec![root, b't'];
        edge.extend_from_slice(&0u64.to_be_bytes());
        edge.extend_from_slice(&1u64.to_be_bytes());
        expected.insert(edge, Vec::new());
        if pinned {
            let mut pin = 1u64.to_be_bytes().to_vec();
            pin.extend_from_slice(&heads[0]);
            expected.insert(vec![root, b'p', 0], pin);
        }
        expected.insert(vec![root, b'n'], 2u64.to_le_bytes().to_vec());
        let charge = expected
            .iter()
            .filter(|(k, _)| k[0] == root)
            .map(|(k, v)| k.len() + v.len())
            .sum::<usize>()
            + 10;
        expected.insert(vec![root, b'q'], (charge as u64).to_le_bytes().to_vec());
    }
    if actual != expected {
        difference::report(&actual, &expected);
        return Err(
            "guest private records differ from independent content/history/quota oracle".into(),
        );
    }
    println!(
        "VERIFIED: both roots, content, revision {revision}, retention={retention}, exact live records ({})",
        actual.len()
    );
    Ok(())
}
