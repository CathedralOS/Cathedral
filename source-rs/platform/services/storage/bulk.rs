//! Reconstruct directly into fresh output pages, freeze, then attach read rights.
//! No second heap-sized payload copy and no caller-writable publication buffer.
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
        Ok(object) => {
            if server.reply_memory(request, &bytes, &object).is_ok() {
                return;
            }
            bytes = wire::error(abi::BUSY);
        }
        Err(status) => bytes = wire::error(status),
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
        return Err(abi::INVALID_ARGUMENT);
    }
    if r.op == wire::BULK_PIN && r.offset >= wire::PIN_SLOTS {
        return Err(abi::INVALID_ARGUMENT);
    }
    let offset = usize::try_from(r.offset).map_err(|_| abi::INVALID_ARGUMENT)?;
    let mut pages = Private::allocate((r.len as usize).div_ceil(4096)).map_err(|_| abi::BUSY)?;
    let out = &mut pages.bytes_mut()[..r.len as usize];
    let head = if r.op == wire::BULK_PIN {
        store.read_pinned(root, r.offset as u8, r.object, r.expected, 0, out)
    } else {
        store.read(root, r.object, r.expected, offset, out)
    }
    .map_err(super::service::status)?;
    *reply = wire::Reply {
        object: r.object,
        revision: head.revision,
        length: head.len as u64,
        extra: r.len,
    }
    .encode();
    pages.freeze().map_err(|_| abi::BUSY)
}
