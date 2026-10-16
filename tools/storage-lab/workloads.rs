//! Workload entrance: tiny objects, sequential data, range edits, copies and
//! insertion. Verification and reachability scans are outside measured time.
use crate::chunks::{Edit, Live, Policy, Reads, Store, Version, Work};
use std::{fmt, time::Instant};

pub struct Row {
    name: &'static str,
    logical: usize,
    work: Work,
    live: Live,
    elapsed_us: u128,
    reads: Reads,
}

#[derive(Clone, Copy)]
enum Publication {
    RetainEach,
    Separate,
    Batch,
}

impl fmt::Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let w = self.work;
        let l = self.live;
        write!(
            f,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            self.name,
            self.logical,
            w.hashed_payload,
            w.hashed_manifest,
            w.new_payload,
            w.new_manifest,
            l.payload,
            l.manifests,
            l.chunks,
            l.versions,
            l.tree_nodes,
            self.reads.metadata_bytes,
            self.reads.metadata_records,
            self.reads.payload_bytes,
            self.reads.payload_records,
            self.elapsed_us
        )
    }
}

pub fn measure(policy: Policy) -> Vec<Row> {
    let mut rows = vec![tiny(policy)];
    let bytes = fixture(10 * 1024 * 1024, 73);
    let mut store = Store::new(policy);
    let start = Instant::now();
    let original = store.put(&bytes);
    rows.push(finish(
        "sequential_10mib",
        bytes.len(),
        &mut store,
        start,
        std::slice::from_ref(&original),
    ));
    assert_eq!(store.read(&original), bytes);

    store.work = Work::default();
    let start = Instant::now();
    // A copied object would add an identity/head record in a real metadata engine.
    // This row measures only its unchanged content representation.
    let copies = vec![original.clone(); 100];
    rows.push(finish(
        "copy_100_content_only",
        bytes.len() * copies.len(),
        &mut store,
        start,
        &copies,
    ));

    for (name, offset, length) in [
        ("patch_1_aligned", bytes.len() / 2, 1),
        ("patch_100_aligned", bytes.len() / 2, 100),
        ("patch_100_boundary", bytes.len() / 2 - 50, 100),
    ] {
        store.work = Work::default();
        let patch = vec![0xa5; length];
        let start = Instant::now();
        let next = store.patch(&original, offset, &patch).unwrap();
        rows.push(finish(
            name,
            length,
            &mut store,
            start,
            &[original.clone(), next.clone()],
        ));
        let mut expected = bytes.clone();
        expected[offset..offset + length].fill(0xa5);
        assert_eq!(store.read(&next), expected);
        assert_eq!(store.read(&original), bytes);
        store.retain(std::slice::from_ref(&original));
    }

    rows.push(edits(policy, &bytes, Publication::RetainEach, false));
    rows.push(edits(policy, &bytes, Publication::Separate, false));
    rows.push(edits(policy, &bytes, Publication::Batch, false));
    rows.push(edits(policy, &bytes, Publication::Separate, true));
    rows.push(edits(policy, &bytes, Publication::Batch, true));

    for (name, offset) in [
        ("read_100_aligned", bytes.len() / 2),
        ("read_100_boundary", bytes.len() / 2 - 50),
        ("read_100_tail", bytes.len() - 100),
    ] {
        store.work = Work::default();
        let start = Instant::now();
        let (readback, reads) = store.read_range(&original, offset, 100).unwrap();
        let mut row = finish(
            name,
            100,
            &mut store,
            start,
            std::slice::from_ref(&original),
        );
        row.reads = reads;
        assert_eq!(readback, bytes[offset..offset + 100]);
        rows.push(row);
    }

    let mut inserted = bytes.clone();
    inserted.insert(0, 0x42);
    store.work = Work::default();
    let start = Instant::now();
    let next = store.put(&inserted);
    rows.push(finish(
        "insert_one_prefix",
        1,
        &mut store,
        start,
        &[original.clone(), next.clone()],
    ));
    assert_eq!(store.read(&next), inserted);
    assert_eq!(store.read(&original), bytes);
    rows
}

fn tiny(policy: Policy) -> Row {
    let inputs: Vec<_> = (1..=4096).map(|seed| fixture(32, seed)).collect();
    let mut store = Store::new(policy);
    let start = Instant::now();
    let roots: Vec<_> = inputs.iter().map(|bytes| store.put(bytes)).collect();
    let row = finish("tiny_4096x32", inputs.len() * 32, &mut store, start, &roots);
    for (root, expected) in roots.iter().zip(inputs) {
        assert_eq!(store.read(root), expected);
    }
    row
}

fn edits(policy: Policy, bytes: &[u8], publication: Publication, clustered: bool) -> Row {
    let history = matches!(publication, Publication::RetainEach);
    let batched = matches!(publication, Publication::Batch);
    let mut store = Store::new(policy);
    let original = store.put(bytes);
    let mut current = original.clone();
    let mut roots = vec![original.clone()];
    let patches: Vec<_> = (0..64)
        .map(|i| {
            let (offset, length) = if clustered {
                (512 + i, 1)
            } else {
                ((i * 15485863 + 97) % (bytes.len() - 100), 100)
            };
            (offset, fixture(length, i as u64 + 991))
        })
        .collect();
    let batch: Vec<_> = patches
        .iter()
        .map(|(offset, bytes)| Edit {
            offset: *offset,
            bytes,
        })
        .collect();
    store.work = Work::default();
    let start = Instant::now();
    if batched {
        current = store.patch_batch(&current, &batch).unwrap();
    } else {
        for (offset, patch) in &patches {
            current = store.patch(&current, *offset, patch).unwrap();
            if history {
                roots.push(current.clone());
            }
        }
    }
    if !history {
        roots = vec![current.clone()];
    }
    let name = if clustered {
        if batched {
            "local_64_bytes_batched"
        } else {
            "local_64_bytes_separate"
        }
    } else if batched {
        "patch_64_batched"
    } else if history {
        "patch_64_keep_history"
    } else {
        "patch_64_latest_only"
    };
    let logical = patches.iter().map(|(_, bytes)| bytes.len()).sum();
    let row = finish(name, logical, &mut store, start, &roots);
    let mut expected = bytes.to_vec();
    if history {
        assert_eq!(store.read(&original), expected);
    }
    for (i, (offset, patch)) in patches.iter().enumerate() {
        expected[*offset..*offset + patch.len()].copy_from_slice(patch);
        if history {
            assert_eq!(store.read(&roots[i + 1]), expected);
        }
    }
    assert_eq!(store.read(&current), expected);
    row
}

fn finish(
    name: &'static str,
    logical: usize,
    store: &mut Store,
    start: Instant,
    roots: &[Version],
) -> Row {
    let elapsed_us = start.elapsed().as_micros();
    let live = store.retain(roots);
    Row {
        name,
        logical,
        work: store.work,
        live,
        elapsed_us,
        reads: Reads::default(),
    }
}

pub fn fixture(len: usize, mut state: u64) -> Vec<u8> {
    assert_ne!(state, 0);
    (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state as u8
        })
        .collect()
}
