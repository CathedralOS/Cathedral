use crate::chunks::{Edit, Layout, Policy, Store, Work};
use crate::workloads::fixture;

fn store(chunk_bytes: usize, inline_limit: usize) -> Store {
    with_layout(chunk_bytes, inline_limit, Layout::Flat)
}

fn with_layout(chunk_bytes: usize, inline_limit: usize, layout: Layout) -> Store {
    Store::new(Policy {
        chunk_bytes,
        inline_limit,
        layout,
    })
}

#[test]
fn patches_match_byte_model_at_every_small_boundary() {
    for layout in [
        Layout::Flat,
        Layout::Tree { fanout: 2 },
        Layout::Tree { fanout: 3 },
    ] {
        for inline_limit in [0, 12] {
            for len in 0..=25 {
                let input = fixture(len, 713);
                for offset in 0..=len {
                    for count in 0..=len - offset {
                        let mut s = with_layout(8, inline_limit, layout);
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
}

#[test]
fn trees_match_flat_at_height_changes_short_tails_and_range_boundaries() {
    for fanout in [2, 3, 32, 128] {
        for chunks in [1, 2, 3, 4, 5, 8, 9, 10, 31, 32, 33, 127, 128, 129] {
            let bytes = fixture(chunks * 8 - 3, 731);
            let mut tree = with_layout(8, 0, Layout::Tree { fanout });
            let original = tree.put(&bytes);
            for offset in [0, 1, bytes.len() / 2, bytes.len() - 1, bytes.len()] {
                for count in [0, 1, 8, 17, bytes.len()] {
                    let count = count.min(bytes.len() - offset);
                    let (read, _) = tree.read_range(&original, offset, count).unwrap();
                    assert_eq!(read, bytes[offset..offset + count]);
                    let patch = fixture(count, 912);
                    let next = tree.patch(&original, offset, &patch).unwrap();
                    let mut expected = bytes.clone();
                    expected[offset..offset + count].copy_from_slice(&patch);
                    assert_eq!(tree.read(&next), expected);
                    // Incremental path copying produces the same canonical tree
                    // as building the entire independently computed byte model.
                    assert_eq!(next, tree.put(&expected));
                    assert_eq!(tree.read(&original), bytes);
                    tree.retain(std::slice::from_ref(&original));
                }
            }
        }
    }
}

#[test]
fn batches_validate_before_work_and_respect_overlapping_edit_order() {
    for layout in [
        Layout::Flat,
        Layout::Tree { fanout: 2 },
        Layout::Tree { fanout: 32 },
    ] {
        for inline_limit in [0, 128] {
            let mut s = with_layout(8, inline_limit, layout);
            let original = s.put(b"abcdefghijklmnopq");
            s.work = Work::default();
            assert!(
                s.patch_batch(
                    &original,
                    &[
                        Edit {
                            offset: 0,
                            bytes: b"good"
                        },
                        Edit {
                            offset: 17,
                            bytes: b"bad"
                        },
                    ]
                )
                .is_err()
            );
            assert!(
                s.patch_batch(
                    &original,
                    &[Edit {
                        offset: usize::MAX,
                        bytes: b"x"
                    }]
                )
                .is_err()
            );
            assert_eq!(s.work, Work::default());
            let next = s
                .patch_batch(
                    &original,
                    &[
                        Edit {
                            offset: 7,
                            bytes: b"XYZ",
                        },
                        Edit {
                            offset: 8,
                            bytes: b"1234",
                        },
                        Edit {
                            offset: 17,
                            bytes: b"",
                        },
                    ],
                )
                .unwrap();
            assert_eq!(s.read(&next), b"abcdefgX1234mnopq");
            assert_eq!(s.read(&original), b"abcdefghijklmnopq");
            assert_eq!(s.patch_batch(&next, &[]).unwrap(), next);
            let revert = s
                .patch_batch(
                    &original,
                    &[
                        Edit {
                            offset: 0,
                            bytes: b"z",
                        },
                        Edit {
                            offset: 0,
                            bytes: b"a",
                        },
                    ],
                )
                .unwrap();
            assert_eq!(revert, original);
            assert!(s.read_range(&next, usize::MAX, 1).is_err());
            assert!(s.read_range(&next, 18, 0).is_err());
            assert_eq!(s.read_range(&next, 17, 0).unwrap().1.metadata_bytes, 0);
        }
    }
}

#[test]
fn shared_tree_nodes_and_snapshot_payload_survive_collection() {
    for fanout in [2, 3, 32] {
        let layout = Layout::Tree { fanout };
        let mut s = with_layout(8, 0, layout);
        let mut expected = b"abcdefgh".repeat(140);
        let original = s.put(&expected);
        let mut roots = vec![original.clone()];
        let mut expected_versions = vec![expected.clone()];
        for offset in [0, 9, 256, 511, 900, 1119] {
            let next = s.patch(roots.last().unwrap(), offset, b"Z").unwrap();
            expected[offset] = b'Z';
            roots.push(next);
            expected_versions.push(expected.clone());
        }
        s.retain(&roots);
        for (root, bytes) in roots.iter().zip(expected_versions) {
            assert_eq!(s.read(root), bytes);
        }
        let latest = roots.last().unwrap();
        let retained = s.retain(std::slice::from_ref(latest));
        let mut fresh = with_layout(8, 0, layout);
        let rebuilt = fresh.put(&expected);
        assert_eq!(retained, fresh.retain(&[rebuilt]));
        assert_eq!(s.read(latest), expected);
        assert_eq!(s.retain(&[]), Default::default());
    }
}

#[test]
fn a_small_edit_and_read_touch_a_path_not_the_whole_manifest() {
    let bytes = fixture(10 * 1024 * 1024, 73);
    for (layout, expected_metadata, records) in [
        (Layout::Flat, 81937, 1),
        (Layout::Tree { fanout: 32 }, 2212, 4),
        (Layout::Tree { fanout: 128 }, 4799, 3),
    ] {
        let mut s = with_layout(4096, 0, layout);
        let original = s.put(&bytes);
        s.work = Work::default();
        let next = s.patch(&original, bytes.len() / 2, &[0xa5]).unwrap();
        assert_eq!(s.work.new_payload, 4096);
        assert_eq!(s.work.new_manifest, expected_metadata);
        assert_eq!(s.work.hashed_manifest, expected_metadata);
        let (read, cost) = s.read_range(&next, bytes.len() / 2, 1).unwrap();
        assert_eq!(read, [0xa5]);
        assert_eq!(
            (cost.metadata_bytes, cost.metadata_records),
            (expected_metadata, records)
        );
        assert_eq!((cost.payload_bytes, cost.payload_records), (4096, 1));
    }
}

#[test]
fn batching_rebuilds_each_shared_path_once() {
    for layout in [
        Layout::Flat,
        Layout::Tree { fanout: 2 },
        Layout::Tree { fanout: 32 },
    ] {
        let bytes = fixture(1024, 712);
        let mut batched = with_layout(8, 0, layout);
        let mut separate = with_layout(8, 0, layout);
        let old = batched.put(&bytes);
        let mut current = separate.put(&bytes);
        batched.work = Work::default();
        separate.work = Work::default();
        let edits = [
            Edit {
                offset: 1,
                bytes: b"X",
            },
            Edit {
                offset: 2,
                bytes: b"Y",
            },
            Edit {
                offset: 9,
                bytes: b"Z",
            },
        ];
        let next = batched.patch_batch(&old, &edits).unwrap();
        for edit in &edits {
            current = separate.patch(&current, edit.offset, edit.bytes).unwrap();
        }
        assert_eq!(batched.read(&next), separate.read(&current));
        assert_eq!(next, current);
        assert_eq!(batched.work.hashed_payload, 16);
        assert_eq!(separate.work.hashed_payload, 24);
        assert!(batched.work.hashed_manifest < separate.work.hashed_manifest);
        assert!(batched.work.new_manifest < separate.work.new_manifest);
    }
}

#[test]
fn range_costs_deduplicate_repeated_nodes_within_one_cold_read() {
    let mut s = with_layout(8, 0, Layout::Tree { fanout: 2 });
    let bytes = b"abcdefgh".repeat(8);
    let root = s.put(&bytes);
    let live = s.retain(std::slice::from_ref(&root));
    let (read, costs) = s.read_range(&root, 0, bytes.len()).unwrap();
    assert_eq!(read, bytes);
    // Eight identical chunks: one payload and one unique node at each of
    // three levels. Shared nodes are visited at different logical offsets.
    assert_eq!(live.tree_nodes, 3);
    assert_eq!(costs.metadata_bytes, live.manifests);
    assert_eq!(costs.metadata_records, live.tree_nodes + 1);
    assert_eq!((costs.payload_bytes, costs.payload_records), (8, 1));
    assert_eq!(s.read_range(&root, 7, 3).unwrap().0, b"hab");
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
