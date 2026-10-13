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
            let response = if matches!(words[0], wire::STORAGE_GRANT | wire::STORAGE_REVOKE) {
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
                if key == cathedral_contracts::input::RESET as u64 {
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
            parent.storage_request(request.client.unwrap(), request.incarnation, words)?
        }
        Some(words)
            if matches!(
                words[0],
                wire::PRESENT_BEGIN | wire::PRESENT_COMMIT | wire::PRESENT_ABORT
            ) =>
        {
            presentation::dispatch(parent, world, &request, words)?
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
