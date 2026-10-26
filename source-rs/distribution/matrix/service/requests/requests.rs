// Authenticate the connection, ask Omega for the operation, then execute with local custody.
use super::*;
use cathedral_stock_matrix::policy::requests::{self as decisions, Caller, Operation};
use cathedral_user_runtime::server::Request;

fn facts(request: &Request, parent: &Parent, separate_producer: bool) -> decisions::Request {
    let caller = match request.client {
        Some(client) if client >= PRODUCER_BASE => Caller::Producer,
        _ if request.control => Caller::Owner,
        Some(SOURCE) => Caller::Source,
        Some(client) if client >= STORAGE_BASE => Caller::Storage,
        Some(client) if client < CHILDREN => Caller::Child,
        _ => Caller::Unknown,
    };
    decisions::Request {
        caller,
        words: wire::decode(&request.bytes[..request.len]),
        separate_producer,
        storage_stream: request
            .client
            .is_some_and(|client| parent.storage_stream(client)),
    }
}
pub(super) fn dispatch(
    request: Request,
    parent: &mut Parent,
    world: &mut World,
    children: &mut children::Children,
    server: &mut Server,
) -> Result<(), Error> {
    let facts = facts(
        &request,
        parent,
        request
            .client
            .is_some_and(|slot| children.separate_producer(slot)),
    );
    let words = facts.words.unwrap_or([0; 8]);
    let response = match facts.route() {
        Operation::Producer => {
            return children.producer_request(&request, facts.words, world, server);
        }
        Operation::Acquire => return children.acquire(&request, words, parent, world, server),
        Operation::Resolve => return owner::resolve(&request, words, parent, server),
        Operation::Pixel => {
            return server.reply(&request, &wire::encode(parent.pixel(words[1], words[2])?));
        }
        Operation::Owner => {
            if children.dispatch(&request, parent, words, server, world)? {
                flush(parent, world)?;
                return Ok(());
            }
            owner::fallback(&facts, parent, world, children, server)?
        }
        Operation::Source => source::dispatch(parent, world, words)?,
        Operation::Storage => return parent.storage_request(request, words, None),
        Operation::DenyNow => {
            return server.reply(&request, &wire::encode(wire::error(abi::DENIED)));
        }
        Operation::Deny => wire::error(abi::DENIED),
        // The workflow retains the exact request until it can answer.
        Operation::Launch => {
            return children.launch(request, words[1], words[2], parent, world, server);
        }
        Operation::Close => return children.close(request, parent, world, server),
        Operation::Fund => children.fund(
            &request,
            [words[1], words[2], words[3], words[4], words[5]],
            parent,
        )?,
        Operation::AppEntry => children.app_entry(&request, words[1], world),
        Operation::Select => return children.select_app(request, words[1], parent, world, server),
        Operation::Layout => children.apply_layout(&request, words, parent, world)?,
        Operation::Status => children.observations.query(
            request.client.expect("child route"),
            request.incarnation,
            words[1],
        ),
        Operation::SaveAvailability => parent.recovery_request(&request, false)?,
        Operation::RetrySaves => parent.recovery_request(&request, true)?,
        Operation::ManagedTask => children.managed_task(&request),
        Operation::RetryLauncher => children.retry_launcher(&request),
        Operation::Terminate => {
            children.terminate_task(&request, words[1], parent, world, server)?
        }
        Operation::Present => match presentation::dispatch(parent, world, &request, words)? {
            Some(reply) => reply,
            None => return Ok(()),
        },
        Operation::Surface => surfaces::dispatch(parent, world, &request, words)?,
        Operation::World => world.request(
            request.client.expect("child route"),
            request.incarnation,
            words,
        ),
        Operation::RecoveryOwner | Operation::StorageOwner | Operation::LayoutOwner => {
            return Err(Error(abi::IO_ERROR as i64));
        }
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
    let facts = facts(&request, parent, false);
    match facts.attachment() {
        Operation::Storage => parent.storage_request(
            request,
            facts.words.expect("decoded attachment"),
            Some(object),
        ),
        _ => {
            drop(object);
            server.reply(&request, &wire::encode(wire::error(abi::DENIED)))
        }
    }
}
