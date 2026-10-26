//! The owner's local layout commands. These never enter the child dispatcher.
use super::*;
/// Resolve only in this matrix's supplied realm; the path carries no caller scope.
pub(super) fn resolve(
    request: &Request,
    words: wire::Packet,
    parent: &mut Parent,
    server: &mut Server,
) -> Result<(), Error> {
    let encoded = wire::encode(words);
    let path = &encoded[8..40];
    let len = path.iter().position(|b| *b == 0).unwrap_or(32);
    let result = parent.local_storage(|scope| {
        scope.bind().map_err(|c| Error(c as i64))?;
        scope.lookup(&path[..len]).map_err(|c| Error(c as i64))
    });
    let reply = match result {
        Ok(id) => [0, id, 0, 0, 0, 0, 0, 0],
        Err(code) => wire::error(code.0 as u64),
    };
    server.reply(request, &wire::encode(reply))
}
/// Child control declined this request. Omega chooses the remaining owner and
/// success consequences; the host keeps resource mutation and publication order.
pub(super) fn fallback(
    facts: &decisions::Request,
    parent: &mut Parent,
    world: &mut World,
    children: &mut children::Children,
    server: &mut Server,
) -> Result<wire::Packet, Error> {
    let words = facts.words.expect("decoded owner request");
    let response = match facts.owner_fallback() {
        Operation::RecoveryOwner => parent.recovery_owner(server, words),
        Operation::StorageOwner => parent.storage_owner(server, words)?,
        Operation::LayoutOwner => dispatch(parent, world, words),
        _ => return Err(Error(abi::IO_ERROR as i64)),
    };
    let effects = facts.owner_effects(CHILDREN, response[0] == 0);
    if effects & decisions::REMEMBER_STORAGE != 0 {
        children.remember_storage(words);
    }
    if effects & decisions::CHANGED != 0 {
        children.recovery_owner_changed(words);
        children.launch_owner_changed(words);
        children.management_owner_changed(words);
    }
    if effects & decisions::CLEAR_SLOT != 0 {
        parent.grant_recovery(words[1] as usize, None);
    }
    if effects & decisions::CLEAR_PAIR != 0 {
        parent.grant_recovery(0, None);
        parent.grant_recovery(1, None);
    }
    if effects & decisions::REPLAY_SLOT != 0 {
        flush(parent, world)?;
        surfaces::replay(parent, world, words[1] as usize)?;
    }
    if effects & decisions::REPLAY_ALL != 0 {
        for slot in 0..CHILDREN {
            flush(parent, world)?;
            surfaces::replay(parent, world, slot)?;
        }
    }
    Ok(response)
}
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
        ] if slot < CHILDREN as u64
            && count > 0
            && count <= wire::INPUT_STREAMS as u64
            && rights & !wire::PRODUCER_FOCUS == 0
            && channels <= wire::INPUT_STREAMS as u64
            && groups <= wire::INPUT_GROUPS as u64 =>
        {
            if parent.upstream {
                return wire::error(
                    parent
                        .grant_input_producer(
                            world,
                            slot as usize,
                            count as usize,
                            rights,
                            [channels as usize, groups as usize],
                        )
                        .err()
                        .map_or(0, |e| e.0 as u64),
                );
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
