//! One atomic record/head/event publication, plus cold-open verification. The
//! host checks the same records using its own engine instance after QEMU stops.
use cathedral_storage_engine::{
    self as engine,
    redb::{Durability, ReadableDatabase, ReadableTable, TableDefinition},
};
const TABLE: TableDefinition<u64, &[u8]> = TableDefinition::new("guest-runtime");
pub fn run() {
    let disk = cathedral_ata_pio::Ata::open().unwrap();
    let db = engine::open(disk, 4096, 4 * 1024 * 1024, 32768).unwrap();
    let old = {
        let tx = db.begin_read().unwrap();
        match tx.open_table(TABLE) {
            Ok(table) => {
                let value = table.get(1).unwrap().unwrap();
                let revision = u64::from_le_bytes(value.value().try_into().unwrap());
                assert_eq!(table.get(2).unwrap().unwrap().value(), value.value());
                assert_eq!(table.get(3).unwrap().unwrap().value(), value.value());
                revision
            }
            Err(engine::redb::TableError::TableDoesNotExist(_)) => 0,
            Err(error) => panic!("read table: {error:?}"),
        }
    };
    let next = old.checked_add(1).unwrap().to_le_bytes();
    let mut tx = db.begin_write().unwrap();
    tx.set_durability(Durability::Immediate).unwrap();
    {
        let mut table = tx.open_table(TABLE).unwrap();
        for key in 1..=3 {
            table.insert(key, next.as_slice()).unwrap();
        }
    }
    tx.commit().unwrap();
    if old == 0 {
        // The first boot is killed while this live database still owns its cache;
        // no engine close, worker exit, or guest shutdown can clean the file first.
        cathedral_user_runtime::write(b"Cathedral database: acknowledged revision 1\n").unwrap();
        loop {
            cathedral_user_runtime::yield_now();
        }
    }
    drop(db);
    cathedral_user_runtime::write(b"Cathedral database: committed\n").unwrap();
}
