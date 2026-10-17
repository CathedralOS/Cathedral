// Authenticate the connection, then route owner, physical-source and child requests.
use super::*;
use cathedral_user_runtime::server::Request;
pub(super) fn dispatch(
    request: Request,
    parent: &mut Parent,
    world: &mut World,
    children: &mut children::Children,
    server: &mut Server,
) -> Result<(), Error> {
    let response = match wire::decode(&request.bytes[..request.len]) {
        Some(words) if request.control => {
            if let [wire::READ_PIXEL, x, y, 0, 0, 0, 0, 0] = words {
                server.reply(&request, &wire::encode(parent.pixel(x, y)?))?;
                return Ok(());
            }
            if children.dispatch(&request, parent, words, server, world)? {
                flush(parent, world)?;
                return Ok(());
            }
            if words[0] == wire::STORAGE_REQUEST {
                return parent.storage_owner_request(server, request, words);
            }
            let response = if matches!(
                words[0],
                wire::STORAGE_GRANT | wire::STORAGE_REVOKE | wire::STORAGE_STATUS
            ) {
                parent.storage_owner(server, words)?
            } else {
                owner::dispatch(parent, world, words)
            };
            if response[0] == 0 && matches!(words[0], wire::PLACE | wire::REVOKE) {
                flush(parent, world)?;
                surfaces::replay(parent, world, words[1] as usize)?;
            }
            if response[0] == 0 && words[0] == wire::BIND {
                for slot in 0..2 {
                    flush(parent, world)?;
                    surfaces::replay(parent, world, slot)?;
                }
            }
            response
        }
        Some(words) if request.client == Some(2) => match words {
            [wire::POINTER_SOURCE, x, y, buttons @ 0..=7, 0, 0, 0, 0]
                if x < parent.grant.width && y < parent.grant.height =>
            {
                parent.pointer(x, y)?;
                world.pointer(x, y, buttons);
                [0; 8]
            }
            [wire::KEY_SOURCE, key, state, 0, 0, 0, 0, 0] => {
                if key == cathedral_platform_contracts::input::RESET as u64 {
                    world.cancel();
                } else {
                    world.key(key, state);
                }
                [0; 8]
            }
            _ => wire::error(abi::DENIED),
        },
        Some(words)
            if words[0] == wire::STORAGE_INFO
                || (wire::STORAGE_BASE..wire::STORAGE_BASE + 256)
                    .contains(&(words[0] & 0xffff)) =>
        {
            if !request
                .client
                .is_some_and(|client| parent.storage_stream(client))
            {
                server.reply(&request, &wire::encode(wire::error(abi::DENIED)))?;
                return Ok(());
            }
            parent.storage_request(request, words, None)?;
            return Ok(());
        }
        _ if request.client.is_some_and(|client| client >= 3) => {
            // A catalog endpoint confers no interaction or source authority.
            return server.reply(&request, &wire::encode(wire::error(abi::DENIED)));
        }
        Some(words)
            if matches!(
                words[0],
                wire::PRESENT_BEGIN
                    | wire::PRESENT_COMMIT
                    | wire::PRESENT_ABORT
                    | wire::PRESENT_RECTS
            ) =>
        {
            match presentation::dispatch(parent, world, &request, words)? {
                Some(reply) => reply,
                None => return Ok(()),
            }
        }
        Some(words)
            if matches!(
                words[0],
                wire::SURFACE_CREATE | wire::PAINT | wire::ERASE | wire::SURFACE_IMAGE
            ) =>
        {
            surfaces::dispatch(parent, world, &request, words)?
        }
        Some(words) => world.request(request.client.unwrap(), request.incarnation, words),
        None => wire::error(abi::DENIED),
    };
    flush(parent, world)?;
    // Publish binding/input invalidations before acknowledging their mutation.
    children.notify(world)?;
    server.reply(&request, &wire::encode(response))?;
    Ok(())
}
/// A sealed attachment on a catalog stream can only accompany a publication.
pub(super) fn attachment(
    request: Request,
    object: cathedral_user_runtime::memory::Object,
    parent: &mut Parent,
    server: &mut Server,
) -> Result<(), Error> {
    match wire::decode(&request.bytes[..request.len]) {
        Some(words)
            if !request.control
                && words[0] & 0xffff
                    == wire::STORAGE_BASE + cathedral_platform_contracts::storage::PUBLISH =>
        {
            parent.storage_request(request, words, Some(object))
        }
        _ => {
            drop(object);
            server.reply(&request, &wire::encode(wire::error(abi::DENIED)))
        }
    }
}
