//! Disposable-image fixture: exhaust real retained-record quota with live data.
//! No fabricated ledger or fault injection; the ordinary provider must refuse.
use super::*;
pub fn fill(path: &Path, offset: u64, bytes: u64) -> Result<()> {
    use cathedral_database_objects::{Error, Store};
    let db = open(Region::new(Arc::new(Image::open(path)?), offset, bytes)?)?;
    let store = Store::open(&db).map_err(|e| format!("open: {e:?}"))?;
    while store
        .collect(0)
        .map_err(|e| format!("collect: {e:?}"))?
        .more
    {}
    let mut size = 64 * 1024;
    let mut random = 123456789u64;
    let mut count = 0;
    loop {
        let data: Vec<u8> = (0..size)
            .map(|_| {
                random ^= random << 13;
                random ^= random >> 7;
                random ^= random << 17;
                random as u8
            })
            .collect();
        match store.publish(0, 0, 0, &data, 0) {
            Ok(_) => count += 1,
            Err(Error::Capacity) if size != 0 => size /= 2,
            Err(Error::Capacity) => break,
            Err(e) => return Err(format!("fill: {e:?}").into()),
        }
    }
    assert_eq!(
        store
            .collect(0)
            .map_err(|e| format!("collect: {e:?}"))?
            .removed,
        0
    );
    assert_eq!(store.publish(0, 0, 0, b"draft", 0), Err(Error::Capacity));
    println!(
        "CAPACITY FIXTURE: {count} live filler objects; empty creation refused; collection cannot reclaim live data"
    );
    Ok(())
}
