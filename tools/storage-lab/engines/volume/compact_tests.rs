use super::*;

#[test]
fn maintenance_preserves_cold_records_guards_backup_and_reclaims_space() {
    let directory = std::env::temp_dir().join(format!("cathedral-compact-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("guest.raw");
    let bytes = 4 * 1024 * 1024;
    let image = Arc::new(Image::create(&path, bytes + 8192).unwrap());
    image.write(0, &[0x5a; 4096]).unwrap();
    image.write(bytes + 4096, &[0xa5; 4096]).unwrap();
    let region = Region::new(image.clone(), 4096, bytes).unwrap();
    format(region.clone(), [7; 16]).unwrap();
    let db = open(region.clone()).unwrap();
    // Grow then replace a large value. Keep a separate retained-history table
    // to ensure maintenance is not silently collecting superseded records.
    for size in [128 * 1024, 512] {
        let tx = db.begin_write().unwrap();
        {
            let mut table = tx.open_table(TABLE).unwrap();
            table
                .insert(b"payload".as_slice(), vec![9; size].as_slice())
                .unwrap();
            let mut retained = tx
                .open_table(TableDefinition::<&[u8], &[u8]>::new("private-objects-v3"))
                .unwrap();
            retained
                .insert(
                    size.to_le_bytes().as_slice(),
                    b"retain even old revisions".as_slice(),
                )
                .unwrap();
        }
        tx.commit().unwrap();
    }
    let expected = records(&db).unwrap();
    drop(db);
    let before_length = region.headers().unwrap().length;
    drop(region);
    drop(image);
    let original = std::fs::read(&path).unwrap();
    let backup = directory.join(format!(
        "guest.raw.before-compact-{}.raw",
        blake3::hash(&original).to_hex()
    ));
    // A conflicting recovery point must stop maintenance before any write.
    std::fs::write(&backup, b"do not overwrite").unwrap();
    assert!(run(&path, 4096, bytes).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(std::fs::read(&backup).unwrap(), b"do not overwrite");
    std::fs::remove_file(&backup).unwrap();
    run(&path, 4096, bytes).unwrap();
    assert_eq!(std::fs::read(&backup).unwrap(), original);
    let raw = std::fs::read(&path).unwrap();
    assert_eq!(raw.len(), original.len());
    assert_eq!(&raw[..4096], &original[..4096]);
    assert_eq!(&raw[raw.len() - 4096..], &original[original.len() - 4096..]);
    {
        let region = Region::new(Arc::new(Image::open(&path).unwrap()), 4096, bytes).unwrap();
        assert!(region.headers().unwrap().length < before_length);
        let db = open(region).unwrap();
        assert_eq!(records(&db).unwrap(), expected);
    }
    run(&path, 4096, bytes).unwrap();
    let db =
        open(Region::new(Arc::new(Image::open(&path).unwrap()), 4096, bytes).unwrap()).unwrap();
    assert_eq!(records(&db).unwrap(), expected);
    drop(db);
    // Only the private directory created by this test is removed.
    std::fs::remove_dir_all(directory).unwrap();
}
