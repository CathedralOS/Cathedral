//! The owner's local layout commands. These never enter the child dispatcher.
use super::*;
pub(super) fn dispatch(
    parent: &mut Parent,
    world: &mut World,
    words: wire::Packet,
) -> wire::Packet {
    let result = match words {
        [
            wire::INPUT_PRODUCER_GRANT,
            slot,
            count,
            rights,
            channels,
            groups,
            0,
            0,
        ] => {
            // Upstream may legally supply all four channels/groups. Until the
            // parent protocol negotiates a lower ceiling, a nested stock host
            // cannot promise spare capacity for another local producer.
            if parent.upstream {
                return wire::error(abi::NO_MEMORY);
            }
            if let Err(code) = source::bind(world) {
                return wire::error(code);
            }
            return world.owner_command([
                wire::INPUT_PRODUCER_GRANT,
                slot,
                count,
                rights,
                channels,
                groups,
                0,
                0,
            ]);
        }
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
                source::bind(world)
            } else {
                Err(abi::DENIED)
            }
        }
        [
            wire::SURFACE_LIMIT,
            child @ 0..=LAST_CHILD,
            count,
            0,
            0,
            0,
            0,
            0,
        ] if count <= wire::MAX_SURFACES as u64 => {
            surface_limit(world, child as usize, count as usize)
        }
        _ if matches!(words[0], wire::POINTER_SOURCE | wire::KEY_SOURCE) && !parent.upstream => {
            return world.source_command(words);
        }
        _ => return world.owner_command(words),
    };
    wire::error(result.err().unwrap_or(0))
}
