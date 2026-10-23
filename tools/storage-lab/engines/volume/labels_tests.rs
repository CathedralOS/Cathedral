use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

fn fixture() -> Records {
    let programs = [
        (0, b"\x7fELFold launcher".as_slice()),
        (0, b""),
        (2, b"\x7fELFmatrix"),
        (3, b"\x7fELFold launcher"),
        (3, b"\x7fELFold notes"),
        (3, b"\x7fELFold colors"),
    ]
    .map(|(parent, bytes)| seed::Program {
        parent,
        bytes: bytes.to_vec(),
    });
    let mut records = seed::expected(&programs);
    // One existing user and its three programs, sharing the original template.
    for source in 3u64..=6 {
        let object = source + 4;
        let mut h = records[&key(b'h', &source.to_be_bytes())].clone();
        let parent = if source == 3 { 2u64 } else { 7 };
        h[48..].copy_from_slice(&parent.to_le_bytes());
        records.insert(key(b'h', &object.to_be_bytes()), h.clone());
        records.insert(event(object, 1), h);
        let mut edge = parent.to_be_bytes().to_vec();
        edge.extend_from_slice(&object.to_be_bytes());
        records.insert(key(b't', &edge), vec![]);
    }
    put(&mut records, 11, 1, 9, b"NTS1keep this draft");
    put(&mut records, 12, 1, 10, b"MC01\0\x01");
    let mut new_launcher = b"\x7fELFnew launcher".to_vec();
    new_launcher.extend_from_slice(&title(b"LAUNCHER"));
    put(&mut records, 4, 2, 3, &new_launcher);
    for (id, code, name) in [
        (5, b"\x7fELFnew notes".as_slice(), b"NOTES".as_slice()),
        (6, b"\x7fELFnew colors", b"COLORS"),
    ] {
        let mut bytes = code.to_vec();
        bytes.extend_from_slice(&title(name));
        put(&mut records, id, 2, 3, &bytes);
    }
    // A customized launcher with a convincing title must never match stock.
    let mut custom = b"\x7fELFcustom launcher".to_vec();
    custom.extend_from_slice(&title(b"LAUNCHER"));
    put(&mut records, 13, 1, 2, b"\x7fELFcustom user");
    put(&mut records, 14, 1, 13, &custom);
    records.insert(key(b'n', &[]), 15u64.to_le_bytes().to_vec());
    let charged = records
        .iter()
        .map(|(k, v)| k.len() + v.len())
        .sum::<usize>()
        + 10;
    records.insert(key(b'q', &[]), (charged as u64).to_le_bytes().to_vec());
    records
}

fn put(records: &mut Records, object: u64, revision: u64, parent: u64, data: &[u8]) {
    let mut store = Store::new(Policy {
        chunk_bytes: cathedral_database_objects::CHUNK_BYTES,
        inline_limit: 128,
        layout: Layout::Flat,
    });
    let version = store.put(data);
    let publication = store.records(&version);
    for (k, v) in &publication {
        if k != b"head" {
            records.insert(key(k[0], &k[1..]), v.clone());
            if k[0] == b'c' {
                records.insert(key(b'k', &k[1..]), vec![]);
            }
        }
    }
    let h = head(
        revision,
        &publication[b"head".as_slice()],
        data.len(),
        parent,
    );
    records.insert(key(b'h', &object.to_be_bytes()), h.clone());
    records.insert(event(object, revision), h);
    let mut edge = parent.to_be_bytes().to_vec();
    edge.extend_from_slice(&object.to_be_bytes());
    records.insert(key(b't', &edge), vec![]);
}

#[test]
fn exact_stock_match_updates_launcher_only_and_labels_original_app_code() {
    let records = fixture();
    let changes = plan(&records, &[], false).unwrap();
    assert_eq!(
        changes.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        vec![8, 9, 10]
    );
    assert_eq!(
        changes[0].1,
        content(&records, &records[&key(b'h', &4u64.to_be_bytes())]).unwrap()
    );
    assert_eq!(
        &changes[1].1[..changes[1].1.len() - 32],
        b"\x7fELFold notes"
    );
    assert_eq!(
        &changes[2].1[..changes[2].1.len() - 32],
        b"\x7fELFold colors"
    );
    assert!(changes[1].1.ends_with(&title(b"NOTES")));
    assert!(changes[2].1.ends_with(&title(b"COLORS")));
}

#[test]
fn collected_history_requires_exact_legacy_digest_and_corruption_refuses() {
    let mut records = fixture();
    let digest: [u8; 32] = records[&event(4, 1)][8..40].try_into().unwrap();
    records.remove(&event(4, 1));
    assert!(
        !plan(&records, &[], false)
            .unwrap()
            .iter()
            .any(|(id, _)| *id == 8)
    );
    assert!(
        plan(&records, &[(4, digest)], false)
            .unwrap()
            .iter()
            .any(|(id, _)| *id == 8)
    );
    let manifest = key(b'm', &digest);
    *records.get_mut(&manifest).unwrap().last_mut().unwrap() ^= 1;
    assert!(plan(&records, &[(4, digest)], false).is_err());
}

#[test]
fn app_close_upgrade_recognizes_previously_labelled_stock_without_touching_custom_code() {
    let mut records = fixture();
    let legacy: Vec<_> = (4..=6)
        .map(|id| (id, records[&event(id, 1)][8..40].try_into().unwrap()))
        .collect();
    for (id, data) in plan(&records, &[], false).unwrap() {
        put(&mut records, id, 2, 7, &data);
    }
    for id in 4..=6 {
        records.remove(&event(id, 1));
    }
    let changes = plan(&records, &legacy, true).unwrap();
    assert_eq!(
        changes.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        vec![9, 10]
    );
    for (id, data) in changes {
        assert_eq!(
            data,
            content(&records, &records[&key(b'h', &(id - 4).to_be_bytes())]).unwrap()
        );
        put(&mut records, id, 3, 7, &data);
    }
    assert!(plan(&records, &legacy, true).unwrap().is_empty());
}

#[test]
fn transaction_preserves_data_custom_code_history_quota_and_is_idempotent() {
    for refresh_apps in [false, true] {
        transaction(refresh_apps);
    }
}
fn transaction(refresh_apps: bool) {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "cathedral-labels-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("guest.raw");
    let bytes = 4 * 1024 * 1024;
    let image = Arc::new(Image::create(&path, bytes + 8192).unwrap());
    image.write(0, &[0x5a; 4096]).unwrap();
    image.write(bytes + 4096, &[0xa5; 4096]).unwrap();
    format(Region::new(image.clone(), 4096, bytes).unwrap(), [1; 16]).unwrap();
    drop(image);
    let original = fixture();
    {
        let db =
            open(Region::new(Arc::new(Image::open(&path).unwrap()), 4096, bytes).unwrap()).unwrap();
        let tx = db.begin_write().unwrap();
        {
            let mut table = tx
                .open_table(TableDefinition::<&[u8], &[u8]>::new("private-objects-v3"))
                .unwrap();
            for (k, v) in &original {
                table.insert(k.as_slice(), v.as_slice()).unwrap();
            }
        }
        tx.commit().unwrap();
    }
    let raw = std::fs::read(&path).unwrap();
    upgrade(&path, 4096, bytes, &[], refresh_apps).unwrap();
    assert_eq!(
        std::fs::read(dir.join(if refresh_apps {
            "guest.raw.before-app-close.raw"
        } else {
            "guest.raw.before-readable-labels.raw"
        }))
        .unwrap(),
        raw
    );
    let read = || -> Records {
        let db =
            open(Region::new(Arc::new(Image::open(&path).unwrap()), 4096, bytes).unwrap()).unwrap();
        let tx = db.begin_read().unwrap();
        tx.open_table(TableDefinition::<&[u8], &[u8]>::new("private-objects-v3"))
            .unwrap()
            .iter()
            .unwrap()
            .map(|row| {
                let (k, v) = row.unwrap();
                (k.value().to_vec(), v.value().to_vec())
            })
            .collect()
    };
    let after = read();
    for (k, v) in &original {
        if k == &key(b'q', &[]) || (8u64..=10).any(|id| k == &key(b'h', &id.to_be_bytes())) {
            continue;
        }
        assert_eq!(after.get(k), Some(v), "unrelated record changed: {k:?}");
    }
    let charge = after
        .iter()
        .filter(|(k, _)| **k != key(b'q', &[]))
        .map(|(k, v)| k.len() + v.len())
        .sum::<usize>()
        + 10;
    assert_eq!(after[&key(b'q', &[])], (charge as u64).to_le_bytes());
    upgrade(&path, 4096, bytes, &[], refresh_apps).unwrap();
    assert_eq!(read(), after);
    let raw_after = std::fs::read(&path).unwrap();
    assert_eq!(raw_after.len(), raw.len());
    assert_eq!(&raw_after[..4096], &raw[..4096]);
    assert_eq!(&raw_after[raw.len() - 4096..], &raw[raw.len() - 4096..]);
    // Remove only this test's explicit private directory.
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn old_matrix_upgrade_requires_exact_history_or_explicit_stock_digest() {
    let mut rows = fixture();
    let old: [u8; 32] = rows[&event(3, 1)][8..40].try_into().unwrap();
    put(&mut rows, 3, 2, 2, b"\x7fELFnew matrix");
    rows.remove(&event(3, 1));
    assert!(
        !plan(&rows, &[], true)
            .unwrap()
            .iter()
            .any(|(id, _)| *id == 7)
    );
    let changes = plan(&rows, &[(3, old)], true).unwrap();
    assert!(
        changes
            .iter()
            .any(|(id, bytes)| *id == 7 && bytes == b"\x7fELFnew matrix")
    );
    assert!(!changes.iter().any(|(id, _)| *id == 13));
}

#[test]
fn quota_pressure_collects_only_unreachable_history_and_refuses_live_excess_atomically() {
    for live_excess in [false, true] {
        let dir = std::env::temp_dir().join(format!(
            "cathedral-upgrade-quota-{}-{live_excess}",
            std::process::id()
        ));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("guest.raw");
        // Leave physical scratch space so this fixture isolates the logical quota.
        let bytes = 16 * 1024 * 1024;
        let image = Arc::new(Image::create(&path, bytes + 8192).unwrap());
        image.write(0, &[0x5a; 4096]).unwrap();
        image.write(bytes + 4096, &[0xa5; 4096]).unwrap();
        format(Region::new(image.clone(), 4096, bytes).unwrap(), [1; 16]).unwrap();
        drop(image);
        let mut original = fixture();
        let revisions = cathedral_database_objects::ROOT_QUOTA / (192 * 1024) + 1;
        for revision in 1u64..=revisions {
            let mut data = Vec::new();
            for part in 0u64..6144 {
                data.extend_from_slice(
                    blake3::hash(&[revision.to_le_bytes(), part.to_le_bytes()].concat()).as_bytes(),
                );
            }
            put(
                &mut original,
                if live_excess { 14 + revision } else { 15 },
                if live_excess { 1 } else { revision },
                0,
                &data,
            );
        }
        if !live_excess {
            // A durable pin keeps an obsolete revision alive through the upgrade.
            let mut pin = 15u64.to_be_bytes().to_vec();
            pin.extend_from_slice(&original[&event(15, 1)]);
            original.insert(key(b'p', &[0]), pin);
            put(&mut original, 15, revisions + 1, 0, b"current data");
        }
        original.insert(key(b'n', &[]), (15 + revisions).to_le_bytes().to_vec());
        let charge: usize = original.iter().map(|(k, v)| k.len() + v.len()).sum();
        original.insert(key(b'q', &[]), (charge as u64).to_le_bytes().to_vec());
        assert!(charge as u64 > cathedral_database_objects::ROOT_QUOTA);
        {
            let db = open(Region::new(Arc::new(Image::open(&path).unwrap()), 4096, bytes).unwrap())
                .unwrap();
            let tx = db.begin_write().unwrap();
            {
                let mut table = tx
                    .open_table(TableDefinition::<&[u8], &[u8]>::new("private-objects-v3"))
                    .unwrap();
                for (k, v) in &original {
                    table.insert(k.as_slice(), v.as_slice()).unwrap();
                }
            }
            tx.commit().unwrap();
        }
        let raw = std::fs::read(&path).unwrap();
        let result = upgrade(&path, 4096, bytes, &[], true);
        let read = || -> Records {
            let db = open(Region::new(Arc::new(Image::open(&path).unwrap()), 4096, bytes).unwrap())
                .unwrap();
            let tx = db.begin_read().unwrap();
            tx.open_table(TableDefinition::<&[u8], &[u8]>::new("private-objects-v3"))
                .unwrap()
                .iter()
                .unwrap()
                .map(|row| {
                    let (k, v) = row.unwrap();
                    (k.value().to_vec(), v.value().to_vec())
                })
                .collect()
        };
        let after = read();
        if live_excess {
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("live/pinned content exceeds")
            );
            assert_eq!(
                after, original,
                "refused upgrade must roll back replacements and collection"
            );
        } else {
            result.unwrap();
            let charge: usize = after.iter().map(|(k, v)| k.len() + v.len()).sum();
            assert!(charge as u64 <= cathedral_database_objects::ROOT_QUOTA);
            assert_eq!(after[&key(b'q', &[])], (charge as u64).to_le_bytes());
            assert!(!after.contains_key(&event(15, 2)));
            assert_eq!(after[&event(15, 1)], original[&event(15, 1)]);
            // Compare every structural record, custom program, saved note and pin.
            for (k, v) in &original {
                if [b'e', b'm', b'c', b'k', b'q'].contains(&k[1])
                    || (8u64..=10).any(|id| *k == key(b'h', &id.to_be_bytes()))
                {
                    continue;
                }
                assert_eq!(after.get(k), Some(v), "preserved record {k:?}");
            }
            // Independently follow each current/pinned manifest into its chunks.
            for head in after
                .iter()
                .filter(|(k, _)| k[1] == b'h')
                .map(|(_, v)| v.as_slice())
                .chain([&after[&key(b'p', &[0])][8..]])
            {
                let manifest = &after[&key(b'm', &head[8..40])];
                if manifest[0] == 1 {
                    for digest in manifest[17..].chunks_exact(32) {
                        assert_eq!(
                            after.get(&key(b'c', digest)),
                            original.get(&key(b'c', digest))
                        );
                        assert!(after.contains_key(&key(b'k', digest)));
                    }
                }
            }
            for id in 8u64..=10 {
                assert_eq!(
                    content(&after, &after[&key(b'h', &id.to_be_bytes())]).unwrap(),
                    content(&original, &original[&key(b'h', &(id - 4).to_be_bytes())]).unwrap()
                );
            }
            upgrade(&path, 4096, bytes, &[], true).unwrap();
            assert_eq!(read(), after);
        }
        assert_eq!(
            std::fs::read(dir.join("guest.raw.before-app-close.raw")).unwrap(),
            raw
        );
        let final_image = std::fs::read(&path).unwrap();
        assert_eq!(final_image.len(), raw.len());
        assert_eq!(&final_image[..4096], &raw[..4096]);
        assert_eq!(
            &final_image[final_image.len() - 4096..],
            &raw[raw.len() - 4096..]
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
}
#[test]
fn coordinated_upgrade_checks_final_generation_and_preserves_pinned_excess() {
    for pinned in [false, true] {
        let dir = std::env::temp_dir().join(format!(
            "cathedral-install-quota-{}-{pinned}",
            std::process::id()
        ));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("guest.raw");
        let bytes = 16 * 1024 * 1024;
        let image = Arc::new(Image::create(&path, bytes + 8192).unwrap());
        image.write(0, &[0x5a; 4096]).unwrap();
        image.write(bytes + 4096, &[0xa5; 4096]).unwrap();
        format(Region::new(image.clone(), 4096, bytes).unwrap(), [2; 16]).unwrap();
        drop(image);
        let payload = |tag: u64| {
            let mut data = vec![0; cathedral_database_objects::MAX_BYTES];
            for (i, chunk) in data.chunks_mut(4096).enumerate() {
                chunk[..8].copy_from_slice(&tag.to_le_bytes());
                chunk[8..16].copy_from_slice(&(i as u64).to_le_bytes());
            }
            data[..4].copy_from_slice(b"\x7fELF");
            data
        };
        let mut rows = fixture();
        let old = payload(100);
        let named = dir.join("nested.elf");
        std::fs::write(&named, &old).unwrap();
        let mut additions = vec![
            "Nested".into(),
            named.to_string_lossy().into_owned(),
            "-".into(),
        ];
        realm_programs::install(&mut rows, &additions).unwrap();
        additions[2] = digest(&old).iter().map(|b| format!("{b:02x}")).collect();
        // Original live state fits. A new large matrix temporarily pushes it
        // over quota until the later named replacement retires the old code.
        let count = cathedral_database_objects::ROOT_QUOTA
            / cathedral_database_objects::MAX_BYTES as u64
            - 2;
        for i in 0..count {
            put(&mut rows, 100 + i, 1, 0, &payload(i));
        }
        rows.insert(key(b'n', &[]), (100 + count).to_le_bytes().to_vec());
        if pinned {
            let mut pin = 15u64.to_be_bytes().to_vec();
            pin.extend_from_slice(&rows[&key(b'h', &15u64.to_be_bytes())]);
            rows.insert(key(b'p', &[0]), pin);
        }
        let charge: usize = rows.iter().map(|(k, v)| k.len() + v.len()).sum();
        rows.insert(key(b'q', &[]), (charge as u64).to_le_bytes().to_vec());
        assert!(charge as u64 <= cathedral_database_objects::ROOT_QUOTA);
        assert!(charge as u64 + payload(101).len() as u64 > cathedral_database_objects::ROOT_QUOTA);
        {
            let db = open(Region::new(Arc::new(Image::open(&path).unwrap()), 4096, bytes).unwrap())
                .unwrap();
            let tx = db.begin_write().unwrap();
            {
                let mut table = tx
                    .open_table(TableDefinition::<&[u8], &[u8]>::new("private-objects-v3"))
                    .unwrap();
                for (k, v) in &rows {
                    table.insert(k.as_slice(), v.as_slice()).unwrap();
                }
            }
            tx.commit().unwrap();
        }
        let original = std::fs::read(&path).unwrap();
        let migration = dir.join("migration.txt");
        std::fs::write(&migration, "").unwrap();
        let mut args = vec![
            path.to_string_lossy().into_owned(),
            "4096".into(),
            bytes.to_string(),
            migration.to_string_lossy().into_owned(),
            "true".into(),
            "6".into(),
        ];
        for id in 1u64..=6 {
            let h = &rows[&key(b'h', &id.to_be_bytes())];
            let data = match id {
                2 => vec![],
                3 => payload(101),
                _ => content(&rows, h).unwrap(),
            };
            let file = dir.join(format!("{id}.elf"));
            std::fs::write(&file, data).unwrap();
            args.push(u64::from_le_bytes(h[48..].try_into().unwrap()).to_string());
            args.push(file.to_string_lossy().into_owned());
        }
        std::fs::write(&named, b"\x7fELFsmall replacement").unwrap();
        args.extend(additions);
        let result = install::run(&args);
        if pinned {
            assert!(result.unwrap_err().to_string().contains("live/pinned"));
            assert_eq!(std::fs::read(&path).unwrap(), original);
        } else {
            result.unwrap();
            let read = || {
                let db =
                    open(Region::new(Arc::new(Image::open(&path).unwrap()), 4096, bytes).unwrap())
                        .unwrap();
                compact::records(&db).unwrap()
            };
            let after = read();
            let actual = &after["private-objects-v3"];
            for (k, v) in &rows {
                if matches!(k[1], b'h' | b'p')
                    && ![7u64, 8, 9, 10, 15, 16, 3]
                        .iter()
                        .any(|id| *k == key(b'h', &id.to_be_bytes()))
                {
                    assert_eq!(actual.get(k), Some(v), "saved/custom head or pin changed");
                }
            }
            assert_eq!(
                content(actual, &actual[&key(b'h', &7u64.to_be_bytes())]).unwrap(),
                payload(101)
            );
            for id in [15u64, 16] {
                assert_eq!(
                    content(actual, &actual[&key(b'h', &id.to_be_bytes())]).unwrap(),
                    b"\x7fELFsmall replacement"
                );
            }
            let charge: usize = actual
                .iter()
                .filter(|(k, _)| k[0] == 0)
                .map(|(k, v)| k.len() + v.len())
                .sum();
            assert!(charge as u64 <= cathedral_database_objects::ROOT_QUOTA);
            assert_eq!(actual[&key(b'q', &[])], (charge as u64).to_le_bytes());
            install::run(&args).unwrap();
            assert_eq!(read(), after);
            let raw = std::fs::read(&path).unwrap();
            assert_eq!(&raw[..4096], &original[..4096]);
            assert_eq!(&raw[raw.len() - 4096..], &original[original.len() - 4096..]);
        }
        // Only this test's exclusively created private directory.
        std::fs::remove_dir_all(dir).unwrap();
    }
}
