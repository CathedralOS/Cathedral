//! Name a parent's children in one sealed page: `Entry` rows for the direct
//! children after a cursor, so a realm is read once instead of once per child.
use cathedral_database_objects::Store;
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::storage as wire;
use cathedral_user_runtime::{
    memory::{Object, Private},
    server::{Request, Server},
};
pub fn reply(
    store: &Store<'_>,
    root: u8,
    r: wire::Request,
    server: &mut Server,
    request: &Request,
) {
    let mut bytes = [0; 64];
    match prepare(store, root, r, &mut bytes) {
        Ok(Some(page)) => match server.reply_memory(request, &bytes, &page) {
            Ok(()) => return,
            Err(error) => bytes = wire::error(error.0 as u64),
        },
        Ok(None) => (),
        Err(status) => bytes = wire::error(status),
    }
    server.reply(request, &bytes).unwrap();
}
/// The argument word is the number to continue after. An empty listing is
/// answered in the packet alone.
fn prepare(
    store: &Store<'_>,
    root: u8,
    r: wire::Request,
    reply: &mut [u8; 64],
) -> Result<Option<Object>, u64> {
    if r.expected != 0 || r.len != 0 {
        return Err(abi::INVALID_ARGUMENT);
    }
    let mut rows = [wire::Entry::default(); wire::ENTRIES_PER_PAGE];
    let count = store
        .entries(root, r.object, r.argument, &mut rows)
        .map_err(super::service::status)?;
    let mut words = [0; 8];
    words[1] = count as u64;
    words[2] = rows[..count].last().map_or(r.argument, |row| row.object);
    *reply = wire::pack(words);
    if count == 0 {
        return Ok(None);
    }
    let mut page = Private::allocate(1).map_err(|error| error.0 as u64)?;
    for (slot, row) in page
        .bytes_mut()
        .chunks_exact_mut(wire::ENTRY_BYTES)
        .zip(&rows[..count])
    {
        slot.copy_from_slice(&row.encode());
    }
    page.freeze().map(Some).map_err(|error| error.0 as u64)
}
