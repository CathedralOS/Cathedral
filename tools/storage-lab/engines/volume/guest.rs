//! Independent host verification of the actual guest's atomic three-record update.
use super::*;
use redb::ReadableTable;
use redb::ReadableTableMetadata;
pub fn verify(path: &Path, offset: u64, bytes: u64, revision: u64) -> Result<()> {
    let image = Arc::new(Image::open(path)?);
    let db = open(Region::new(image, offset, bytes)?)?;
    let tx = db.begin_read()?;
    let table = tx.open_table(TableDefinition::<u64, &[u8]>::new("guest-runtime"))?;
    assert_eq!(table.len()?, 3);
    for key in 1..=3 {
        assert_eq!(
            table.get(key)?.ok_or("missing guest record")?.value(),
            revision.to_le_bytes()
        );
    }
    println!("VERIFIED: guest record/head/event at revision {revision}");
    Ok(())
}
