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
    if !request.control
        && let Some(words) = wire::decode(&request.bytes[..request.len])
        && words[0] == cathedral_platform_contracts::services::ACQUIRE
    {
        return children.acquire(&request, words, server);
    }
    let response = match wire::decode(&request.bytes[..request.len]) {
        Some(words) if request.control => {
            if words[0] == wire::RESOLVE_PATH && words[5..] == [0; 3] {
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
                return server.reply(&request, &wire::encode(reply));
            }
            if let [wire::READ_PIXEL, x, y, 0, 0, 0, 0, 0] = words {
                server.reply(&request, &wire::encode(parent.pixel(x, y)?))?;
                return Ok(());
            }
            if children.dispatch(&request, parent, words, server, world)? {
                flush(parent, world)?;
                return Ok(());
            }
            let response = if matches!(
                words[0],
                wire::SAVE_RECOVERY_GRANT | wire::SAVE_RECOVERY_POLL
            ) {
                parent.recovery_owner(server, words)
            } else if matches!(
                words[0],
                wire::STORAGE_GRANT
                    | wire::STORAGE_REVOKE
                    | wire::STORAGE_STATUS
                    | wire::STORAGE_BINDING
            ) {
                let response = parent.storage_owner(server, words)?;
                if response[0] == 0 {
                    children.remember_storage(words);
                }
                response
            } else {
                owner::dispatch(parent, world, words)
            };
            if response[0] == 0 {
                children.recovery_owner_changed(words);
                children.management_owner_changed(words);
                match words {
                    [wire::REVOKE, slot @ 0..=LAST_CHILD, 0, 0, 0, 0, 0, 0] => {
                        parent.grant_recovery(slot as usize, None)
                    }
                    [wire::BIND, ..] => {
                        parent.grant_recovery(0, None);
                        parent.grant_recovery(1, None);
                    }
                    _ => (),
                }
            }
            if response[0] == 0 && matches!(words[0], wire::PLACE | wire::REVOKE) {
                flush(parent, world)?;
                surfaces::replay(parent, world, words[1] as usize)?;
            }
            if response[0] == 0 && words[0] == wire::BIND {
                for slot in 0..CHILDREN {
                    flush(parent, world)?;
                    surfaces::replay(parent, world, slot)?;
                }
            }
            response
        }
        Some(words) if request.client == Some(SOURCE) => match words {
            [wire::POINTER_SOURCE, x, y, 0..=7, 0, 0, 0, 0]
                if x < parent.grant.width && y < parent.grant.height =>
            {
                parent.pointer(x, y)?;
                world.source_command(words)
            }
            [wire::KEY_SOURCE, key, _, 0, 0, 0, 0, 0] => {
                if key == cathedral_platform_contracts::input::RESET as u64 {
                    world.cancel();
                    [0; 8]
                } else {
                    world.source_command(words)
                }
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
        _ if request.client.is_some_and(|client| client >= STORAGE_BASE) => {
            // A catalog endpoint confers no interaction or source authority.
            return server.reply(&request, &wire::encode(wire::error(abi::DENIED)));
        }
        // A child asks for a sibling, or for its sibling to go; both are
        // answered later, from launch_object or the forwarded loss query.
        Some([wire::LAUNCH, object, withhold, 0, 0, 0, 0, 0]) if request.client.is_some() => {
            return children.launch(request, object, withhold, parent, world, server);
        }
        Some([wire::CLOSE, 0, 0, 0, 0, 0, 0, 0]) if request.client.is_some() => {
            return children.close(request, parent, world, server);
        }
        Some([wire::FUND, private, readers, channels, slots, frames, 0, 0]) => children.fund(
            &request,
            [private, readers, channels, slots, frames],
            parent,
        )?,
        Some([wire::APP_ENTRY, index, 0, 0, 0, 0, 0, 0]) => {
            children.app_entry(&request, index, world)
        }
        Some([wire::SELECT_APP, ticket, 0, 0, 0, 0, 0, 0]) => {
            children.select_app(&request, ticket, parent, world, server)?
        }
        Some(words) if words[0] == wire::APPLY_LAYOUT => {
            children.apply_layout(&request, words, parent, world)?
        }
        Some([wire::LAUNCH_STATUS, receipt, 0, 0, 0, 0, 0, 0]) => {
            children
                .observations
                .query(request.client.unwrap(), request.incarnation, receipt)
        }
        Some([wire::SAVE_AVAILABILITY, 0, 0, 0, 0, 0, 0, 0]) => {
            parent.recovery_request(&request, false)?
        }
        Some([wire::RETRY_SAVES, 0, 0, 0, 0, 0, 0, 0]) => {
            parent.recovery_request(&request, true)?
        }
        Some([wire::MANAGED_TASK, 0, 0, 0, 0, 0, 0, 0]) => children.managed_task(&request),
        Some([wire::RETRY_LAUNCHER, 0, 0, 0, 0, 0, 0, 0]) => children.retry_launcher(&request),
        Some([wire::TERMINATE_TASK, ticket, 0, 0, 0, 0, 0, 0]) => {
            children.terminate_task(&request, ticket, parent, world, server)?
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
/// Sealed attachments carry content or bounded hierarchy request payloads.
pub(super) fn attachment(
    request: Request,
    object: cathedral_user_runtime::memory::Object,
    parent: &mut Parent,
    server: &mut Server,
) -> Result<(), Error> {
    match wire::decode(&request.bytes[..request.len]) {
        Some(words)
            if !request.control
                && matches!(
                    (words[0] & 0xffff).wrapping_sub(wire::STORAGE_BASE),
                    cathedral_platform_contracts::storage::PUBLISH
                        | cathedral_platform_contracts::storage::CREATE_NAMED
                        | cathedral_platform_contracts::storage::SET_METADATA
                        | cathedral_platform_contracts::storage::LOOKUP
                        | cathedral_platform_contracts::storage::FILTER
                        | cathedral_platform_contracts::storage::COPY_TREE
                        | cathedral_platform_contracts::storage::COPY
                ) =>
        {
            parent.storage_request(request, words, Some(object))
        }
        _ => {
            drop(object);
            server.reply(&request, &wire::encode(wire::error(abi::DENIED)))
        }
    }
}
