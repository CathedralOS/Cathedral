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
        Ok(object) => match server.reply_memory(request, &bytes, &object) {
            Ok(()) => return,
            Err(error) => bytes = wire::error(error.0 as u64),
        },
        Err(status) => bytes = wire::error(status),
    }
    server.reply(request, &bytes).unwrap();
}
/// The argument word is the byte offset for BULK and the pin slot for BULK_PIN.
/// A BULK with no length reads the whole object from its offset: the length
/// comes from the head, so a launcher spends no round trip on HEAD first.
fn prepare(
    store: &Store<'_>,
    root: u8,
    r: wire::Request,
    reply: &mut [u8; 64],
) -> Result<Object, u64> {
    if r.len > wire::MAX_BYTES as u64 {
        return Err(abi::INVALID_ARGUMENT);
    }
    if r.op == wire::BULK_PIN && (r.expected == 0 || r.len == 0 || r.argument >= wire::PIN_SLOTS) {
        return Err(abi::INVALID_ARGUMENT);
    }
    let offset = usize::try_from(r.argument).map_err(|_| abi::INVALID_ARGUMENT)?;
    let len = if r.op == wire::BULK && r.len == 0 {
        let head = store.head(root, r.object).map_err(super::service::status)?;
        if r.expected != 0 && head.revision != r.expected {
            return Err(abi::WOULD_BLOCK);
        }
        head.len.checked_sub(offset).ok_or(abi::INVALID_ARGUMENT)?
    } else {
        r.len as usize
    };
    if len == 0 || len > wire::MAX_BYTES {
        return Err(abi::INVALID_ARGUMENT);
    }
    let count = len.div_ceil(4096);
    let mut pages = Private::allocate(count).map_err(|error| {
        // Diagnosable in every build: the pages behind earlier replies stay
        // charged here until their last reader releases them.
        let budget = cathedral_user_runtime::resources::budget().ok();
        let physical = cathedral_user_runtime::resources::physical_budget().ok();
        cathedral_user_runtime::time::storage_sample(
            "bulk-reply-refused",
            &[
                count as u64,
                error.0 as u64,
                budget.map_or(0, |b| b.used.private_pages),
                budget.map_or(0, |b| b.limits.private_pages),
                physical.map_or(0, |p| p.charged),
                physical.map_or(0, |p| p.reserved),
            ],
        );
        error.0 as u64
    })?;
    let out = &mut pages.bytes_mut()[..len];
    let head = if r.op == wire::BULK_PIN {
        store.read_pinned(root, r.argument as u8, r.object, r.expected, 0, out)
    } else {
        store.read(root, r.object, r.expected, offset, out)
    }
    .map_err(super::service::status)?;
    *reply = wire::Reply {
        object: r.object,
        revision: head.revision,
        length: head.len as u64,
        extra: len as u64,
    }
    .encode();
    pages.freeze().map_err(|error| error.0 as u64)
}
