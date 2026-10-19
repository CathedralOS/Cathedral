//! The owner's local layout commands. These never enter the child dispatcher.
use super::*;
pub(super) fn dispatch(
    parent: &mut Parent,
    world: &mut World,
    words: wire::Packet,
) -> wire::Packet {
    let result = match words {
        [wire::BIND, width, height, keys, 0, 0, 0, 0] if !parent.upstream => {
            let grant = Grant {
                epoch: 1,
                width,
                height,
                keys,
            };
            if grant.valid() {
                parent.grant = grant;
                world.inherit(grant);
                world.surface_capacity(wire::MAX_SURFACES);
                Ok(())
            } else {
                Err(abi::DENIED)
            }
        }
        [wire::SURFACE_LIMIT, child @ 0..=1, count, 0, 0, 0, 0, 0]
            if count <= wire::MAX_SURFACES as u64 =>
        {
            surface_limit(world, child as usize, count as usize)
        }
        _ if matches!(words[0], wire::POINTER_SOURCE | wire::KEY_SOURCE) && !parent.upstream => {
            return world.source_command(words);
        }
        _ => return world.owner_command(words),
    };
    wire::error(result.err().unwrap_or(0))
}
