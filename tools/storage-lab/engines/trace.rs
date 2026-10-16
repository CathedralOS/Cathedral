//! Identical byte records for each engine; all immutable history is retained.
use crate::chunks::{Edit, Layout, Policy, Store};
use crate::workloads::fixture;
use std::collections::BTreeMap;

pub type Records = BTreeMap<Vec<u8>, Vec<u8>>;
pub type Batch = Vec<(Vec<u8>, Vec<u8>)>;
pub struct Trace {
    pub initial: Records,
    pub batches: Vec<Batch>,
    pub expected: Records,
}

pub fn make(layout: Layout, chunk_bytes: usize, scenario: &str) -> Trace {
    if scenario == "tiny" {
        let expected: Records = (0..4096u64)
            .map(|i| {
                let mut key = b"object/".to_vec();
                key.extend_from_slice(&i.to_be_bytes());
                (key, fixture(32, i + 1))
            })
            .collect();
        return Trace {
            initial: Records::new(),
            batches: vec![expected.clone().into_iter().collect()],
            expected,
        };
    }
    let mut store = Store::new(Policy {
        chunk_bytes,
        inline_limit: 128,
        layout,
    });
    let bytes = fixture(10 * 1024 * 1024, 73);
    let mut version = store.put(&bytes);
    let initial = store.records(&version);
    let mut expected = initial.clone();
    let patches: Vec<_> = (0..if scenario == "single" { 1 } else { 64 })
        .map(|i| (512 + i, fixture(1, i as u64 + 991)))
        .collect();
    let mut batches = Vec::new();
    if scenario == "batch" {
        let edits: Vec<_> = patches
            .iter()
            .map(|(offset, bytes)| Edit {
                offset: *offset,
                bytes,
            })
            .collect();
        version = store.patch_batch(&version, &edits).unwrap();
        publish(&store, &version, 1, &mut expected, &mut batches);
    } else {
        for (i, (offset, bytes)) in patches.iter().enumerate() {
            version = store.patch(&version, *offset, bytes).unwrap();
            publish(&store, &version, i as u64 + 1, &mut expected, &mut batches);
        }
    }
    let mut oracle = bytes;
    for (offset, bytes) in patches {
        oracle[offset..offset + bytes.len()].copy_from_slice(&bytes);
    }
    assert_eq!(store.read(&version), oracle);
    Trace {
        initial,
        batches,
        expected,
    }
}

fn publish(
    store: &Store,
    version: &crate::chunks::Version,
    sequence: u64,
    previous: &mut Records,
    batches: &mut Vec<Batch>,
) {
    let mut current = store.records(version);
    // State and its observation record are in the same engine transaction.
    let mut event = b"event/".to_vec();
    event.extend_from_slice(&sequence.to_be_bytes());
    current.insert(event, current[b"head".as_slice()].clone());
    let changes: Batch = current
        .into_iter()
        .filter(|(key, value)| previous.get(key) != Some(value))
        .collect();
    for (key, value) in &changes {
        previous.insert(key.clone(), value.clone());
    }
    batches.push(changes);
}
