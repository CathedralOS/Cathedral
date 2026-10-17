//! Reconstruct directly into fresh output pages, freeze, then attach read rights.
//! No second heap-sized payload copy and no caller-writable publication buffer.
use cathedral_database_objects::Store;
use cathedral_platform_contracts::database as wire;
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
    let result = prepare(store, root, r, &mut bytes);
    match result {
        Ok(object) => {
            if server.reply_memory(request, &bytes, &object).is_ok() {
                return;
            }
            bytes.fill(0);
            bytes[..8].copy_from_slice(&wire::BUSY.to_le_bytes());
        }
        Err(status) => {
            bytes.fill(0);
            bytes[..8].copy_from_slice(&status.to_le_bytes());
        }
    }
    server.reply(request, &bytes).unwrap();
}
fn prepare(
    store: &Store<'_>,
    root: u8,
    r: wire::Request,
    reply: &mut [u8; 64],
) -> Result<Object, u64> {
    if r.expected == 0 || r.len == 0 || r.len > wire::MAX_BYTES as u64 {
        return Err(wire::INVALID);
    }
    if r.op == wire::BULK_PIN && r.offset >= cathedral_database_objects::PIN_SLOTS as u64 {
        return Err(wire::INVALID);
    }
    let offset = usize::try_from(r.offset).map_err(|_| wire::INVALID)?;
    let mut pages = Private::allocate((r.len as usize).div_ceil(4096)).map_err(|_| wire::BUSY)?;
    let out = &mut pages.bytes_mut()[..r.len as usize];
    let head = if r.op == wire::BULK_PIN {
        store.read_pinned(root, r.offset as u8, r.expected, 0, out)
    } else {
        store.read(root, r.expected, offset, out)
    }
    .map_err(super::service::status)?;
    reply[8..16].copy_from_slice(&head.revision.to_le_bytes());
    reply[16..24].copy_from_slice(&(head.len as u64).to_le_bytes());
    reply[24..32].copy_from_slice(&r.len.to_le_bytes());
    pages.freeze().map_err(|_| wire::BUSY)
}
