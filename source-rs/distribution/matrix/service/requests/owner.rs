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
        [wire::PLACE, child @ 0..=1, x, y, width, height, keys, 0] => world.place(
            child as usize,
            Rect {
                x,
                y,
                width,
                height,
            },
            keys,
        ),
        [wire::SURFACE_LIMIT, child @ 0..=1, count, 0, 0, 0, 0, 0]
            if count <= wire::MAX_SURFACES as u64 =>
        {
            surface_limit(world, child as usize, count as usize)
        }
        [wire::REVOKE, child @ 0..=1, 0, 0, 0, 0, 0, 0] => world.revoke(child as usize),
        [wire::POINTER_SOURCE, x, y, buttons @ 0..=7, 0, 0, 0, 0] if !parent.upstream => {
            world.pointer(x, y, buttons);
            Ok(())
        }
        [wire::KEY_SOURCE, key, state, 0, 0, 0, 0, 0] if !parent.upstream => {
            world.key(key, state);
            Ok(())
        }
        [wire::HEALTH, 0, 0, 0, 0, 0, 0, 0] => Ok(()),
        _ => Err(abi::DENIED),
    };
    wire::error(result.err().unwrap_or(0))
}
