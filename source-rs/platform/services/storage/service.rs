//! Connection -> private root -> scope check -> bounded request -> store
//! operation -> reply. Protocol lives in platform/contracts/storage.rs. Root
//! slot bindings are explicit composition policy, stable across reconnection,
//! not global IDs. Scope stamps are the only delegation evidence: a request may
//! touch an object only if every stamped scope is one of its ancestors.
use cathedral_database_objects::{Error, Object, Stage, Store};
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::storage as wire;
use cathedral_user_runtime::{
    memory::Object as Attachment,
    server::{Event, Server},
    write,
};
/// The protocol's object bound is the store's; neither may drift alone.
const _: () = assert!(wire::MAX_BYTES == cathedral_database_objects::MAX_BYTES);

pub fn serve(
    mut db: cathedral_storage_engine::redb::Database,
    reopen: fn() -> cathedral_storage_engine::redb::Database,
) -> cathedral_storage_engine::redb::Database {
    let mut server = Server::open_clients(wire::ROOTS).unwrap();
    server.enable_memory();
    #[cfg(feature = "recovery-lab")]
    let mut gate = super::gate::Gate::default();
    write(b"Cathedral: object storage ready\n").unwrap();
    loop {
        let (request, mut attachment) = match server.next_event().unwrap() {
            Event::Request(request) => (request, None),
            Event::Memory(request, object) => (request, Some(object)),
            Event::Disconnected { .. } => continue,
        };
        let store = Store::open(&db).unwrap();
        let bytes = &request.bytes[..request.len];
        let command = wire::Request::decode(bytes);
        if request.control {
            if attachment.is_none() && bytes == wire::HEALTH {
                server.reply(&request, b"ready").unwrap();
                continue;
            }
            if attachment.is_none()
                && command.is_some_and(|r| r == wire::Request::new(wire::CLOSE, 0, 0, 0, 0))
            {
                server.reply(&request, &[0; 64]).unwrap();
                return db;
            }
            #[cfg(feature = "recovery-lab")]
            if attachment.is_none() && gate.control(&store, &mut server, &request) {
                continue;
            }
            server
                .reply(&request, &wire::error(abi::INVALID_ARGUMENT))
                .unwrap();
            continue;
        }
        let root = request.client.unwrap() as u8;
        let Some(r) = command else {
            drop(attachment);
            server
                .reply(&request, &wire::error(abi::INVALID_ARGUMENT))
                .unwrap();
            continue;
        };
        #[cfg(feature = "trace-storage")]
        cathedral_user_runtime::time::storage_trace("provider-begin", root as u64, r.op);
        #[cfg(feature = "trace-storage")]
        let io_before = cathedral_ata_pio::metrics::snapshot();
        #[cfg(feature = "recovery-lab")]
        gate.wait(1, r.op, root);
        let authority = if r.deadline != 0
            && cathedral_user_runtime::time::now()
                .is_ok_and(|now| cathedral_user_runtime::time::reached(now, r.deadline))
        {
            Err(abi::TIMED_OUT)
        } else {
            scoped(&store, root, r)
        };
        if let Err(code) = authority {
            drop(attachment);
            server.reply(&request, &wire::error(code)).unwrap();
        } else if matches!(r.op, wire::BULK | wire::BULK_PIN) && attachment.is_none() {
            super::bulk::reply(&store, root, r, &mut server, &request);
        } else {
            let mut response = [0; 64];
            let result = dispatch(&store, root, r, attachment.as_mut(), &mut response);
            // Release the reader grant before replying: synchronous callers can
            // release/reuse their pages immediately after observing completion.
            drop(attachment);
            #[cfg(feature = "recovery-lab")]
            gate.wait(2, r.op, root);
            let bytes = match result {
                Ok(()) => response,
                Err(Error::ReopenCapacity) => {
                    // The uncommitted writer and attachment are gone. Keep the
                    // service's authenticated connections while recovering redb.
                    drop(db);
                    db = reopen();
                    write(b"Cathedral storage: capacity refused; engine reopened\n").unwrap();
                    wire::error(abi::NO_MEMORY)
                }
                Err(error) => wire::error(status(error)),
            };
            server.reply(&request, &bytes).unwrap();
        }
        #[cfg(feature = "trace-storage")]
        {
            let after = cathedral_ata_pio::metrics::snapshot();
            let delta: [u64; 5] = core::array::from_fn(|i| after[i].wrapping_sub(io_before[i]));
            cathedral_user_runtime::time::storage_sample(
                "ata-io",
                &[
                    root as u64,
                    r.op,
                    delta[0],
                    delta[1],
                    delta[2],
                    delta[3],
                    delta[4],
                ],
            );
            cathedral_user_runtime::time::storage_trace("provider-end", root as u64, r.op);
        }
    }
}

/// Every stamped scope must contain every subject. An object outside a scope
/// and an object that does not exist are refused alike, so a scoped client
/// cannot probe what lies beyond its subtree. The scope object itself is the
/// realm's root: readable, listable and copyable, but never replaced or
/// deleted by what lives under it, so a program cannot overwrite its own code.
fn scoped(store: &Store<'_>, root: u8, r: wire::Request) -> Result<(), u64> {
    let (subjects, count) = r.subjects();
    for &scope in r.scopes.iter().filter(|s| **s != 0) {
        if matches!(r.op, wire::PUBLISH | wire::DELETE | wire::SET_METADATA) && r.object == scope {
            return Err(abi::DENIED);
        }
        for &subject in &subjects[..count] {
            match store.within(root, subject, scope) {
                Ok(true) => (),
                Ok(false) | Err(Error::Missing) => return Err(abi::DENIED),
                Err(error) => return Err(status(error)),
            }
        }
    }
    Ok(())
}

/// Consult the retained-byte ledger before a mutation; scan only under quota
/// pressure. Never drain the whole backlog inside an application's deadline.
/// Further requests (or explicit COLLECT) continue bounded reclamation.
fn collect(store: &Store<'_>, root: u8) -> Result<(), Error> {
    store.collect_under_pressure(root)?;
    Ok(())
}

fn observe(stage: Stage) {
    #[cfg(feature = "trace-storage")]
    {
        let label = match stage {
            Stage::Prepared => "prepared",
            Stage::Committing => "commit-begin",
            Stage::Committed => "committed",
        };
        cathedral_user_runtime::time::storage_trace(label, 0, 0);
    }
    crate::metrics::report(match stage {
        Stage::Prepared => "prepared",
        Stage::Committed => "committed",
        Stage::Committing => return,
    });
}

fn dispatch(
    store: &Store<'_>,
    root: u8,
    r: wire::Request,
    attachment: Option<&mut Attachment>,
    response: &mut [u8; 64],
) -> Result<(), Error> {
    let describe = |response: &mut [u8; 64], object: Object| {
        *response = wire::Reply {
            object: object.id,
            revision: object.head.revision,
            length: object.head.len as u64,
            extra: 0,
        }
        .encode();
    };
    if matches!(
        r.op,
        wire::METADATA | wire::SET_METADATA | wire::LOOKUP | wire::FILTER
    ) || (matches!(r.op, wire::COPY | wire::COPY_TREE) && attachment.is_some())
    {
        if matches!(r.op, wire::SET_METADATA | wire::COPY | wire::COPY_TREE) {
            collect(store, root)?;
        }
        return super::hierarchy::dispatch(store, root, r, attachment, response);
    }
    let plain = attachment.is_none();
    match r.op {
        wire::HEAD if plain && r.expected == 0 && r.argument == 0 && r.len == 0 => {
            let head = store.head(root, r.object)?;
            describe(response, Object { id: r.object, head });
        }
        wire::READ if plain && r.len <= wire::READ_BYTES as u64 => {
            let offset = usize::try_from(r.argument).map_err(|_| Error::Invalid)?;
            let mut bytes = [0; wire::READ_BYTES];
            let head = store.read(
                root,
                r.object,
                r.expected,
                offset,
                &mut bytes[..r.len as usize],
            )?;
            describe(response, Object { id: r.object, head });
            response[32..].copy_from_slice(&bytes);
        }
        wire::PUBLISH | wire::CREATE_NAMED if r.object == 0 || r.argument == 0 => {
            if r.op == wire::CREATE_NAMED
                && (r.object != 0 || r.expected != 0 || r.len < wire::metadata::BYTES as u64)
            {
                return Err(Error::Invalid);
            }
            // Check logical and backing sizes before mapping or hashing input.
            if r.len > wire::MAX_BYTES as u64 {
                return Err(Error::TooLarge);
            }
            let mapped;
            let bytes: &[u8] = match attachment {
                Some(attachment) => {
                    if attachment.len() > wire::MAX_BYTES || attachment.len() < r.len as usize {
                        return Err(Error::Invalid);
                    }
                    // A mapping refused for lack of memory is capacity, not a
                    // malformed request.
                    mapped = attachment.map().map_err(|error| {
                        if error.0 as u64 == abi::NO_MEMORY {
                            Error::Capacity
                        } else {
                            Error::Invalid
                        }
                    })?;
                    &mapped[..r.len as usize]
                }
                None if r.len == 0 => &[],
                None => return Err(Error::Invalid),
            };
            #[cfg(feature = "trace-storage")]
            cathedral_user_runtime::time::storage_trace("collect-begin", root as u64, r.op);
            collect(store, root)?;
            #[cfg(feature = "trace-storage")]
            cathedral_user_runtime::time::storage_trace("publish-begin", root as u64, r.op);
            let object = if r.op == wire::CREATE_NAMED {
                let metadata = wire::metadata::Metadata::decode(&bytes[..wire::metadata::BYTES])
                    .ok_or(Error::Invalid)?;
                store.create_named_observed(
                    root,
                    r.argument,
                    metadata,
                    &bytes[wire::metadata::BYTES..],
                    observe,
                )?
            } else {
                store.publish_observed(root, r.object, r.expected, bytes, r.argument, observe)?
            };
            describe(response, object);
        }
        wire::COPY if plain && r.len == 0 => {
            collect(store, root)?;
            let object = store.copy(root, r.object, r.expected, r.argument)?;
            describe(response, object);
        }
        wire::COPY_TREE if plain && r.len == 0 => {
            collect(store, root)?;
            let object = store.copy_tree(root, r.object, r.expected, r.argument)?;
            describe(response, object);
        }
        wire::DELETE if plain && r.argument == 0 && r.len == 0 => {
            store.delete(root, r.object, r.expected)?;
        }
        wire::LIST if plain && r.expected == 0 && r.len == 0 => {
            let mut ids = [0; wire::LIST_IDS];
            let count = store.list(root, r.object, r.argument, &mut ids)?;
            let mut words = [0; 8];
            words[1] = count as u64;
            words[2..].copy_from_slice(&ids);
            *response = wire::pack(words);
        }
        wire::PIN | wire::RELEASE if plain && r.len == 0 && r.argument < wire::PIN_SLOTS => {
            if r.op == wire::PIN {
                store.pin(root, r.object, r.expected, r.argument as u8)?;
            } else {
                store.release(root, r.object, r.expected, r.argument as u8)?;
            }
        }
        wire::COLLECT
            if plain && r.object == 0 && r.expected == 0 && r.argument == 0 && r.len == 0 =>
        {
            let collection = store.collect(root)?;
            *response = wire::pack([
                0,
                collection.removed as u64,
                collection.charged,
                u64::from(collection.more),
                0,
                0,
                0,
                0,
            ]);
        }
        _ => return Err(Error::Invalid),
    }
    Ok(())
}

pub(super) fn status(error: Error) -> u64 {
    match error {
        Error::Invalid => abi::INVALID_ARGUMENT,
        Error::TooLarge => wire::TOO_LARGE,
        Error::Conflict => abi::WOULD_BLOCK,
        Error::Capacity => abi::NO_MEMORY,
        Error::Missing => wire::MISSING,
        Error::NotEmpty => abi::BUSY,
        // An engine error may hide publication. Retire rather than claim refusal.
        error => panic!("private-object engine failed: {:?}", error),
    }
}
