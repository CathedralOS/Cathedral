//! Measure the no_std engine's live allocations across the intended guest trace.
mod backend;
mod meter;
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
#[global_allocator]
static METER: meter::Meter = meter::Meter;
const TABLE: TableDefinition<u64, &[u8]> = TableDefinition::new("runtime");
fn open(path: &std::path::Path, create: bool, cache: usize) -> Database {
    Database::builder()
        .set_cache_size(cache)
        .create_with_backend(backend::Backend::open(path, create))
        .unwrap()
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let path = std::path::Path::new(&args[1]);
    let cache: usize = args[2].parse().unwrap();
    println!("phase,live_bytes,peak_bytes,largest_allocation");
    let base = meter::live();
    meter::begin();
    let db = open(path, true, cache);
    meter::report("create", base);
    for (phase, size) in [("small_commit", 32), ("large_commit", 16384)] {
        meter::begin();
        let payload = vec![0x5a; size];
        let mut tx = db.begin_write().unwrap();
        tx.set_durability(redb::Durability::Immediate).unwrap();
        {
            let mut table = tx.open_table(TABLE).unwrap();
            table.insert(1, payload.as_slice()).unwrap();
            table.insert(2, b"matching-head".as_slice()).unwrap();
        }
        tx.commit().unwrap();
        drop(payload);
        meter::report(phase, base);
    }
    meter::begin();
    drop(db);
    meter::report("close", base);
    meter::begin();
    let db = open(path, false, cache);
    let tx = db.begin_read().unwrap();
    let table = tx.open_table(TABLE).unwrap();
    assert_eq!(table.get(1).unwrap().unwrap().value(), &[0x5a; 16384]);
    assert_eq!(table.get(2).unwrap().unwrap().value(), b"matching-head");
    drop(table);
    drop(tx);
    drop(db);
    meter::report("reopen", base);
}
