use super::*;
struct File(std::path::PathBuf);
impl File {
    fn new(bytes: &[u8]) -> Self {
        let name = format!(
            "cathedral-realm-{}-{}.elf",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let file = Self(std::env::temp_dir().join(name));
        std::fs::write(&file.0, bytes).unwrap();
        file
    }
    fn path(&self) -> String {
        self.0.to_string_lossy().into_owned()
    }
}
impl Drop for File {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
fn base() -> Records {
    let programs = [
        (0, b"login".as_slice()),
        (0, b""),
        (2, b"matrix"),
        (3, b"launcher"),
        (3, b"notes"),
        (3, b"colors"),
    ]
    .map(|(parent, bytes)| seed::Program {
        parent,
        bytes: bytes.to_vec(),
    });
    let mut rows = seed::expected(&programs);
    hierarchy::insert_config(&mut rows, 7, 3).unwrap();
    put(&mut rows, 8, 2, b"matrix").unwrap();
    hierarchy::describe(&mut rows, 8, b"User", metadata::EXECUTABLE).unwrap();
    put(&mut rows, 9, 8, b"valuable user data").unwrap();
    hierarchy::describe(&mut rows, 9, b"Keep", metadata::TEXT).unwrap();
    hierarchy::insert_config(&mut rows, 10, 8).unwrap();
    let mut config = Config::decode(&hierarchy::configuration()).unwrap();
    config.margin = 40;
    config.launcher_height = 256;
    config.app_pages = 32;
    config.app_readers = 4;
    config.app_surfaces = 6;
    put(&mut rows, 10, 8, &config.encode()).unwrap();
    rows.insert(seed::key(b'n', &[]), 11u64.to_le_bytes().to_vec());
    rows.insert(seed::key(b'q', &[]), 0u64.to_le_bytes().to_vec());
    rows
}
#[test]
fn named_install_preserves_ids_data_layout_and_custom_code() {
    let split = File::new(b"\x7fELFsplit one");
    let notes = File::new(b"\x7fELFnotes");
    let mut args = vec![
        "Split".into(),
        split.path(),
        "-".into(),
        "Split/Notes".into(),
        notes.path(),
        "-".into(),
    ];
    let mut rows = base();
    let data = rows[&seed::key(b'h', &9u64.to_be_bytes())].clone();
    install(&mut rows, &args).unwrap();
    assert_eq!(rows[&seed::key(b'h', &9u64.to_be_bytes())], data);
    assert_eq!(resolve(&rows, 3, "Split").unwrap(), Some(11));
    assert_eq!(resolve(&rows, 8, "Split").unwrap(), Some(12));
    assert_eq!(resolve(&rows, 3, "Split/Notes").unwrap(), Some(13));
    assert_eq!(resolve(&rows, 8, "Split/Notes").unwrap(), Some(14));
    let head = &rows[&seed::key(b'h', &10u64.to_be_bytes())];
    let config = Config::decode(&rows[&seed::key(b'm', &head[8..40])][17..]).unwrap();
    assert_eq!(
        (
            config.margin,
            config.launcher_height,
            config.app_pages,
            config.app_readers,
            config.app_surfaces
        ),
        (40, 256, 384, 768, 64)
    );
    let once = rows.clone();
    install(&mut rows, &args).unwrap();
    assert_eq!(rows, once);
    args[2] = rows[&seed::key(b'h', &11u64.to_be_bytes())][8..40]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    // Customized user code and its subtree survive a recognized template update.
    put(&mut rows, 12, 8, b"\x7fELFcustom split").unwrap();
    let custom = rows[&seed::key(b'h', &12u64.to_be_bytes())].clone();
    let child = rows[&seed::key(b'h', &14u64.to_be_bytes())].clone();
    std::fs::write(&split.0, b"\x7fELFsplit two").unwrap();
    install(&mut rows, &args).unwrap();
    assert_eq!(rows[&seed::key(b'h', &12u64.to_be_bytes())], custom);
    assert_eq!(rows[&seed::key(b'h', &14u64.to_be_bytes())], child);
    assert_eq!(
        u64::from_le_bytes(
            rows[&seed::key(b'h', &11u64.to_be_bytes())][..8]
                .try_into()
                .unwrap()
        ),
        2
    );
    assert_eq!(rows[&seed::key(b'h', &9u64.to_be_bytes())], data);
}
#[test]
fn first_install_does_not_claim_an_existing_template_name() {
    let split = File::new(b"\x7fELFsplit");
    let notes = File::new(b"\x7fELFnotes");
    let mut rows = base();
    put(&mut rows, 11, 3, b"\x7fELFmy own program").unwrap();
    hierarchy::describe(&mut rows, 11, b"Split", metadata::EXECUTABLE).unwrap();
    rows.insert(seed::key(b'n', &[]), 12u64.to_le_bytes().to_vec());
    let before = rows[&seed::key(b'h', &11u64.to_be_bytes())].clone();
    install(
        &mut rows,
        &[
            "Split".into(),
            split.path(),
            "-".into(),
            "Split/Notes".into(),
            notes.path(),
            "-".into(),
        ],
    )
    .unwrap();
    assert_eq!(rows[&seed::key(b'h', &11u64.to_be_bytes())], before);
    assert_eq!(resolve(&rows, 3, "Split/Notes").unwrap(), None);
    assert_eq!(resolve(&rows, 8, "Split").unwrap(), None);
}

#[test]
fn named_addition_collects_old_history_after_staging_and_keeps_current_user_data() {
    let mut rows = base();
    for revision in 0u64..(cathedral_database_objects::ROOT_QUOTA / (128 * 1024) - 1) {
        let mut data = vec![0; 128 * 1024];
        for (i, chunk) in data.chunks_mut(4096).enumerate() {
            chunk[..8].copy_from_slice(&(revision * 32 + i as u64).to_le_bytes());
            chunk[8..].fill(42);
        }
        put(&mut rows, 9, 8, &data).unwrap();
    }
    put(&mut rows, 9, 8, b"valuable user data").unwrap();
    let saved = rows[&seed::key(b'h', &9u64.to_be_bytes())].clone();
    let charge: usize = rows.iter().map(|(k, v)| k.len() + v.len()).sum();
    assert!(charge < cathedral_database_objects::ROOT_QUOTA as usize);
    rows.insert(seed::key(b'q', &[]), (charge as u64).to_le_bytes().to_vec());
    let mut program = vec![0; 160 * 1024];
    for (i, chunk) in program.chunks_mut(4096).enumerate() {
        chunk[..8].copy_from_slice(&(i as u64).to_le_bytes());
        chunk[8..].fill(77);
    }
    program[..4].copy_from_slice(b"\x7fELF");
    let program = File::new(&program);
    let directory =
        std::env::temp_dir().join(format!("cathedral-realm-quota-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("guest.raw");
    let bytes = 16 * 1024 * 1024;
    let region = Region::new(Arc::new(Image::create(&path, bytes).unwrap()), 0, bytes).unwrap();
    format(region.clone(), [9; 16]).unwrap();
    let db = open(region).unwrap();
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
    drop(db);
    hierarchy::upgrade(
        &path,
        0,
        bytes,
        &["Split".into(), program.path(), "-".into()],
    )
    .unwrap();
    let db = open(Region::new(Arc::new(Image::open(&path).unwrap()), 0, bytes).unwrap()).unwrap();
    let all = compact::records(&db).unwrap();
    let actual = &all["private-objects-v3"];
    assert_eq!(actual[&seed::key(b'h', &9u64.to_be_bytes())], saved);
    assert!(resolve(actual, 8, "Split").unwrap().is_some());
    assert!(
        rows.keys()
            .any(|key| key[1] == b'e' && !actual.contains_key(key)),
        "old revisions must have been collected"
    );
    assert!(
        u64::from_le_bytes(actual[&seed::key(b'q', &[])].as_slice().try_into().unwrap())
            <= cathedral_database_objects::ROOT_QUOTA
    );
    drop(db);
    std::fs::remove_dir_all(directory).unwrap();
}
