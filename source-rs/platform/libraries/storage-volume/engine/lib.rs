#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;
mod backend;
#[cfg(test)]
extern crate std;
#[cfg(test)]
mod tests;
// Experimental redb entrance: inspect an existing ready volume, adapt its
// logical extent to the granted sectors, then validate the engine identity.
use alloc::string::{String, ToString};
use cathedral_block_device::Device;
use cathedral_storage_volume::{BlockRegion, inspect};
pub use redb;
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};

pub fn open<D: Device + Send + 'static>(
    mut device: D,
    start: u64,
    bytes: u64,
    cache: usize,
) -> Result<Database, String> {
    let header =
        inspect(&mut BlockRegion::new(&mut device, start, bytes).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    if !header.ready || header.length == 0 {
        return Err("volume formatting is incomplete".into());
    }
    let id = header.id;
    // This backend entrance creates only at length zero, forbidden above.
    // Nonempty invalid engine bytes are rejected rather than reformatted.
    let db = Database::builder()
        .set_cache_size(cache)
        .create_with_backend(backend::Backend::new(device, start, header))
        .map_err(|e| e.to_string())?;
    {
        let read = db.begin_read().map_err(|e| e.to_string())?;
        let table = read
            .open_table(TableDefinition::<&[u8], &[u8]>::new("volume-lab"))
            .map_err(|e| e.to_string())?;
        let found = table
            .get(b"volume-id".as_slice())
            .map_err(|e| e.to_string())?
            .ok_or("missing volume identity")?;
        if found.value() != id {
            return Err("engine/envelope identity mismatch".into());
        }
    }
    Ok(db)
}
