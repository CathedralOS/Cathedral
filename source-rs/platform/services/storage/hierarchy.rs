//! Bounded hierarchy requests, after the provider has checked every scope.
use cathedral_database_objects::{Error, Store};
use cathedral_platform_contracts::storage::{self as wire, metadata};
use cathedral_user_runtime::memory::Object;

pub fn dispatch(
    store: &Store<'_>,
    root: u8,
    r: wire::Request,
    attachment: Option<&mut Object>,
    response: &mut [u8; 64],
) -> Result<(), Error> {
    if r.op == wire::METADATA {
        if attachment.is_some() || r.expected != 0 || r.len != 0 || r.argument > 1 {
            return Err(Error::Invalid);
        }
        let (head, value) = store.metadata(root, r.object)?;
        *response = wire::pack([0, head.revision, 0, 0, 0, 0, 0, 0]);
        response[32..].copy_from_slice(if r.argument == 0 {
            &value.name
        } else {
            &value.media_type
        });
        return Ok(());
    }
    let object = attachment.ok_or(Error::Invalid)?;
    if object.len() != 4096 || r.len > metadata::PATH_BYTES as u64 {
        return Err(Error::Invalid);
    }
    let mapped = object.map().map_err(|_| Error::Capacity)?;
    let bytes = &mapped[..r.len as usize];
    match r.op {
        wire::SET_METADATA if r.argument == 0 && r.len == metadata::BYTES as u64 => {
            let value = metadata::Metadata::decode(bytes).ok_or(Error::Invalid)?;
            let object = store.set_metadata(root, r.object, r.expected, value)?;
            *response = wire::Reply {
                object: object.id,
                revision: object.head.revision,
                length: object.head.len as u64,
                extra: 0,
            }
            .encode();
        }
        wire::LOOKUP if r.expected == 0 && r.argument == 0 => {
            let id = store.lookup(root, r.object, bytes)?;
            *response = wire::Reply {
                object: id,
                ..Default::default()
            }
            .encode();
        }
        wire::FILTER if r.expected == 0 => {
            let mut ids = [0; wire::LIST_IDS];
            let count = store.filter(root, r.object, bytes, r.argument, &mut ids)?;
            let mut words = [0; 8];
            words[1] = count as u64;
            words[2..].copy_from_slice(&ids);
            *response = wire::pack(words);
        }
        wire::COPY_TREE => {
            let object = store.copy_tree_named(root, r.object, r.expected, r.argument, bytes)?;
            *response = wire::Reply {
                object: object.id,
                revision: object.head.revision,
                length: object.head.len as u64,
                extra: 0,
            }
            .encode();
        }
        _ => return Err(Error::Invalid),
    }
    Ok(())
}
