//! Allocate upstream backing before publishing a fresh child-local surface name.
use super::*;
pub(super) fn dispatch(
    parent: &mut Parent,
    world: &mut World,
    request: &Request,
    words: wire::Packet,
) -> Result<wire::Packet, Error> {
    let slot = request.client.unwrap();
    if words[0] == wire::SURFACE_CREATE {
        if words[3..] != [0; 5] {
            return Ok(wire::error(abi::DENIED));
        }
        let rank = match world.prepare_surface(slot, request.incarnation, words[1], words[2]) {
            Ok(rank) => rank,
            Err(status) => return Ok(wire::error(status)),
        };
        let reply = parent.create_surface(rank)?;
        if reply[0] != 0 {
            return Ok(wire::error(reply[0]));
        }
        let name = world.create_surface(slot, words[2] as usize, reply[1]);
        return Ok([0, name, 0, 0, 0, 0, 0, 0]);
    }
    match world.surface(slot, request.incarnation, words) {
        Ok(words) => parent.surface(words),
        Err(status) => Ok(wire::error(status)),
    }
}
pub(in super::super) fn flush(parent: &Parent, world: &mut World) -> Result<(), Error> {
    while let Some(words) = world.take_retired_surface() {
        let response = parent.surface(words)?;
        // Parent rebind/revoke can already have retired the upstream object.
        if !matches!(response[0], 0 | abi::DENIED | abi::BAD_HANDLE) {
            return Err(Error(response[0] as i64));
        }
    }
    Ok(())
}

pub(super) fn replay(parent: &Parent, world: &mut World, slot: usize) -> Result<(), Error> {
    for rank in 0..wire::MAX_SURFACES {
        if let Some(words) = world.replay(slot, rank) {
            parent.surface(words)?;
        }
    }
    Ok(())
}
