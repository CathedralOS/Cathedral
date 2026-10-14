// Consume local invalidations, wake children, then dispatch ownership and input.
use cathedral_contracts::{matrix as wire, user as abi};
use cathedral_matrix_service::world::{Grant, Rect, World};
use cathedral_user_runtime::{
    Error,
    server::{Event, Server},
    task::Environment,
};
use parent::Parent;

pub fn run(environment: Environment, storage_root: bool) -> Result<(), Error> {
    static mut PARENT: Parent = Parent::empty();
    let parent_address = &raw mut PARENT;
    // SAFETY: Single-threaded executable enters once. Persistent buffers live in
    // process-owned data pages, not constructor temporaries on the guarded stack.
    let parent = unsafe { &mut *parent_address };
    parent.initialize(&environment, storage_root)?;
    static mut WORLD: World = World::empty();
    let world_address = &raw mut WORLD;
    // SAFETY: Single-threaded executable enters this service exactly once. This
    // process-owned arena has no aliases and never occupies the guarded user stack.
    let world = unsafe { &mut *world_address };
    let mut server = Server::open_clients_from(if storage_root { 2 } else { 0 }, 3)?;
    server.enable_memory();
    if environment.len() == 4 {
        server.install(
            2,
            (
                environment.endpoint(2).unwrap(),
                environment.endpoint(3).unwrap(),
            ),
        )?;
    }
    let mut children = children::Children::default();
    static mut DEFERRED: publication::Deferred = publication::Deferred::empty();
    let deferred_address = &raw mut DEFERRED;
    // SAFETY: Same single-entry process lifetime as the parent/world arenas above.
    let deferred = unsafe { &mut *deferred_address };
    loop {
        parent.progress_presentation(world, &mut server)?;
        if !parent.publishing() {
            parent.refresh_notified(world)?;
            children.finish(&mut server, world, parent)?;
            requests::flush(parent, world)?;
        }
        let (handle, mut deadline) = children.wait();
        children.notify(world)?;
        let mut extra = [cathedral_user_runtime::ipc::Handle::from_raw(0); 3];
        let mut count = 0;
        if let Some((receive, until)) = parent.presentation_wait() {
            extra[count] = receive;
            count += 1;
            // Owner forwarding is resumed after publication; its elapsed deadline
            // is still checked then. Avoid a busy loop on that older deadline.
            deadline = Some(until);
        }
        if !parent.publishing() {
            if let Some(handle) = handle {
                extra[count] = handle;
                count += 1;
            }
            if let Some(wake) = parent.wake {
                extra[count] = wake;
                count += 1;
            }
        }
        let queued = if parent.publishing() {
            None
        } else {
            deferred.take(&mut server)
        };
        let event = match queued {
            Some(event) => event,
            None => match server.next_event_or(&extra[..count], deadline)? {
                Some(event) => event,
                None => continue,
            },
        };
        if parent.publishing() {
            deferred.hold(event, &mut server, world, parent.publishing_client())?;
            continue;
        }
        // Admitted children receive coalesced invalidations on their wake channel.
        // Legacy manually assembled two-endpoint fixtures still refresh on demand.
        if parent.wake.is_none() {
            parent.refresh(world)?;
        } else {
            // A parent hint may arrive while waiting for a child request.
            parent.refresh_notified(world)?;
        }
        match event {
            Event::Disconnected { client, .. } => {
                if client < 2 {
                    children.disconnected(client, &mut server, world, parent)?;
                } else {
                    world.cancel();
                }
            }
            Event::Memory(request, object) => {
                requests::image(request, object, parent, world, &mut server)?;
            }
            Event::Request(request) => {
                requests::dispatch(request, parent, world, &mut children, &mut server)?
            }
        }
    }
}
