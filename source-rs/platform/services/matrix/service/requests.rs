//! Authenticate the connection, then route owner, physical-source and child requests.
mod owner;
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
            if let [wire::STOP, slot @ 0..=1, 0, 0, 0, 0, 0, 0] = words {
                parent.erase(slot as usize)?;
            }
            if children.dispatch(&request, words, server, world)? {
                return Ok(());
            }
            let response = owner::dispatch(parent, world, words);
            if response[0] == 0 && matches!(words[0], wire::PLACE | wire::REVOKE) {
                parent.erase(words[1] as usize)?;
                for surface in world.replay(words[1] as usize).into_iter().flatten() {
                    parent.surface(surface)?;
                }
            }
            if response[0] == 0 && words[0] == wire::BIND {
                for slot in 0..2 {
                    parent.erase(slot)?;
                    for surface in world.replay(slot).into_iter().flatten() {
                        parent.surface(surface)?;
                    }
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
        Some(words) if matches!(words[0], wire::PAINT | wire::ERASE) => {
            match world.surface(request.client.unwrap(), request.incarnation, words) {
                Ok(words) => parent.surface(words)?,
                Err(status) => wire::error(status),
            }
        }
        Some(words) => world.request(request.client.unwrap(), request.incarnation, words),
        None => wire::error(abi::DENIED),
    };
    // Publish binding/input invalidations before acknowledging their mutation.
    children.notify(world)?;
    server.reply(&request, &wire::encode(response))?;
    Ok(())
}
