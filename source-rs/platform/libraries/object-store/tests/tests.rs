// Storage failure scenarios. disk models volatile caches and torn sectors.
use super::*;
use cathedral_contracts::storage as wire;
use disk::Disk;

fn replace(
    store: &mut Store<Disk>,
    object: u64,
    expected: u64,
    payload: &[u8],
) -> Result<Record, u64> {
    store.transact(
        0,
        expected,
        &[Change::value(wire::REPLACE, object, payload)?],
        |_| {},
    )?;
    store.read(0, object)
}
