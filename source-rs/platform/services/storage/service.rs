//! Connection -> private root -> bounded request -> store operation -> reply.
//! Protocol lives in platform/contracts/storage.rs. Root slot bindings are
//! explicit composition policy, stable across reconnection, not global IDs.
use cathedral_database_objects::{Error, Object, Stage, Store};
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::storage as wire;
use cathedral_user_runtime::{
    memory::Object as Attachment,
    server::{Event, Server},
    write,
};

pub fn serve(store: &Store<'_>) {
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
        let bytes = &request.bytes[..request.len];
        let command = wire::Request::decode(bytes);
        if request.control {
            if attachment.is_none() && bytes == wire::HEALTH {
                server.reply(&request, b"ready").unwrap();
                continue;
            }
            if attachment.is_none()
                && command.is_some_and(|r| {
                    r.op == wire::CLOSE
                        && r.object == 0
                        && r.expected == 0
                        && r.offset == 0
                        && r.len == 0
                })
            {
                server.reply(&request, &[0; 64]).unwrap();
                return;
            }
            #[cfg(feature = "recovery-lab")]
            if attachment.is_none() && gate.control(store, &mut server, &request) {
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
        if matches!(r.op, wire::BULK | wire::BULK_PIN) && attachment.is_none() {
            super::bulk::reply(store, root, r, &mut server, &request);
        } else {
            let mut response = [0; 64];
            let result = dispatch(store, root, r, attachment.as_mut(), &mut response);
            // Release the reader grant before replying: synchronous callers can
            // release/reuse their pages immediately after observing completion.
            drop(attachment);
            #[cfg(feature = "recovery-lab")]
            gate.wait(2, r.op, root);
            let bytes = match result {
                Ok(()) => response,
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

/// Finish bounded old garbage before allocating the next content revision.
/// Current heads remain intact even if maintenance fails or power is lost.
fn collect(store: &Store<'_>, root: u8) -> Result<(), Error> {
    for _ in 0..64 {
        if !store.collect(root)?.more {
            return Ok(());
        }
    }
    Err(Error::Capacity)
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
    let plain = attachment.is_none();
    match r.op {
        wire::HEAD if plain && r.expected == 0 && r.offset == 0 && r.len == 0 => {
            let head = store.head(root, r.object)?;
            describe(response, Object { id: r.object, head });
        }
        wire::READ if plain && r.len <= wire::READ_BYTES as u64 => {
            let offset = usize::try_from(r.offset).map_err(|_| Error::Invalid)?;
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
        wire::PUBLISH if r.offset == 0 => {
            // Check logical and backing sizes before mapping or hashing input.
            if r.len > wire::MAX_BYTES as u64 {
                return Err(Error::TooLarge);
            }
            let attachment = attachment.ok_or(Error::Invalid)?;
            if attachment.len() > wire::MAX_BYTES || attachment.len() < r.len as usize {
                return Err(Error::Invalid);
            }
            let bytes = attachment.map().map_err(|_| Error::Invalid)?;
            #[cfg(feature = "trace-storage")]
            cathedral_user_runtime::time::storage_trace("collect-begin", root as u64, r.op);
            collect(store, root)?;
            #[cfg(feature = "trace-storage")]
            cathedral_user_runtime::time::storage_trace("publish-begin", root as u64, r.op);
            let object = store.publish_observed(
                root,
                r.object,
                r.expected,
                &bytes[..r.len as usize],
                observe,
            )?;
            describe(response, object);
        }
        wire::COPY if plain && r.offset == 0 && r.len == 0 => {
            collect(store, root)?;
            let object = store.copy(root, r.object, r.expected)?;
            describe(response, object);
        }
        wire::DELETE if plain && r.offset == 0 && r.len == 0 => {
            store.delete(root, r.object, r.expected)?;
        }
        wire::LIST if plain && r.expected == 0 && r.offset == 0 && r.len == 0 => {
            let mut ids = [0; wire::LIST_IDS];
            let count = store.list(root, r.object, &mut ids)?;
            let mut words = [0; 8];
            words[1] = count as u64;
            words[2..].copy_from_slice(&ids);
            *response = wire::pack(words);
        }
        wire::PIN | wire::RELEASE if plain && r.len == 0 && r.offset < wire::PIN_SLOTS => {
            if r.op == wire::PIN {
                store.pin(root, r.object, r.expected, r.offset as u8)?;
            } else {
                store.release(root, r.object, r.expected, r.offset as u8)?;
            }
        }
        wire::COLLECT
            if plain && r.object == 0 && r.expected == 0 && r.offset == 0 && r.len == 0 =>
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
        // An engine error may hide publication. Retire rather than claim refusal.
        error => panic!("private-object engine failed: {:?}", error),
    }
}
