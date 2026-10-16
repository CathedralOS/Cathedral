use crate::chunks::{Policy, Store, Work};
use crate::workloads::fixture;

fn store(chunk_bytes: usize, inline_limit: usize) -> Store {
    Store::new(Policy {
        chunk_bytes,
        inline_limit,
    })
}

#[test]
fn patches_match_byte_model_at_every_small_boundary() {
    for inline_limit in [0, 12] {
        for len in 0..=25 {
            let input = fixture(len, 713);
            for offset in 0..=len {
                for count in 0..=len - offset {
                    let mut s = store(8, inline_limit);
                    let before = s.put(&input);
                    let patch = fixture(count, 991);
                    let after = s.patch(&before, offset, &patch).unwrap();
                    let mut expected = input.clone();
                    expected[offset..offset + count].copy_from_slice(&patch);
                    s.retain(&[before.clone(), after.clone()]);
                    assert_eq!(s.read(&before), input);
                    assert_eq!(s.read(&after), expected);
                }
            }
        }
    }
}

#[test]
fn invalid_ranges_do_no_work() {
    let mut s = store(8, 0);
    let v = s.put(b"abcdefghi");
    s.work = Work::default();
    assert!(s.patch(&v, 9, b"x").is_err());
    assert!(s.patch(&v, 10, b"").is_err());
    assert!(s.patch(&v, usize::MAX, b"x").is_err());
    assert_eq!(s.work, Work::default());
    assert_eq!(s.read(&v), b"abcdefghi");
}

#[test]
fn unchanged_content_allocates_nothing() {
    let mut s = store(8, 0);
    let a = s.put(b"abcdefghijklmnopq");
    s.work = Work::default();
    let b = s.put(b"abcdefghijklmnopq");
    let c = s.patch(&b, 3, b"def").unwrap();
    assert_eq!(a, b);
    assert_eq!(b, c);
    assert_eq!(s.work.new_payload, 0);
    assert_eq!(s.work.new_manifest, 0);
    let live = s.retain(&[a, b, c]);
    assert_eq!((live.payload, live.chunks, live.versions), (17, 3, 1));
}

#[test]
fn old_snapshot_pins_only_its_changed_chunks() {
    let mut s = store(8, 0);
    let before = s.put(b"abcdefghijklmnopq");
    s.work = Work::default();
    let after = s.patch(&before, 7, b"XY").unwrap();
    assert_eq!((s.work.hashed_payload, s.work.new_payload), (16, 16));
    assert_eq!(s.work.new_manifest, 17 + 3 * 32);
    let both = s.retain(&[before.clone(), after.clone()]);
    assert_eq!((both.payload, both.chunks, both.versions), (33, 5, 2));
    assert_eq!(s.read(&before), b"abcdefghijklmnopq");
    let latest = s.retain(std::slice::from_ref(&after));
    assert_eq!((latest.payload, latest.chunks, latest.versions), (17, 3, 1));
    assert_eq!(s.read(&after), b"abcdefgXYjklmnopq");
    assert_eq!(s.retain(&[]), Default::default());
}

#[test]
fn inline_threshold_empty_objects_and_short_tails_have_exact_costs() {
    for len in [0, 1, 12, 13, 16, 17] {
        let mut s = store(8, 12);
        let bytes = fixture(len, 94);
        let v = s.put(&bytes);
        let live = s.retain(std::slice::from_ref(&v));
        if len <= 12 {
            assert_eq!(
                (live.payload, live.manifests, live.chunks),
                (0, 17 + len, 0)
            );
        } else {
            assert_eq!(
                (live.payload, live.manifests, live.chunks),
                (len, 17 + len.div_ceil(8) * 32, len.div_ceil(8))
            );
        }
        assert_eq!(s.read(&v), bytes);
    }
}

#[test]
fn shifted_input_loses_fixed_chunk_reuse() {
    let mut s = store(8, 0);
    let bytes = fixture(128, 713);
    let old = s.put(&bytes);
    let mut shifted = vec![19];
    shifted.extend_from_slice(&bytes);
    s.work = Work::default();
    let new = s.put(&shifted);
    assert_eq!(s.work.new_payload, shifted.len());
    s.retain(&[old.clone(), new.clone()]);
    assert_eq!(s.read(&old), bytes);
    assert_eq!(s.read(&new), shifted);
}

#[test]
fn repeated_chunks_can_diverge_without_changing_other_references() {
    let mut s = store(8, 0);
    let before = s.put(b"abcdefghabcdefghabcdefgh");
    assert_eq!(s.work.new_payload, 8);
    let after = s.patch(&before, 9, b"Z").unwrap();
    let both = s.retain(&[before.clone(), after.clone()]);
    assert_eq!((both.payload, both.chunks), (16, 2));
    assert_eq!(s.read(&before), b"abcdefghabcdefghabcdefgh");
    assert_eq!(s.read(&after), b"abcdefghaZcdefghabcdefgh");
    s.retain(std::slice::from_ref(&after));
    assert_eq!(s.read(&after), b"abcdefghaZcdefghabcdefgh");
}
