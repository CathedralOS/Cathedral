// Consume local invalidations, wake children, then dispatch ownership and input.
use cathedral_kernel_contracts::user as abi;
use cathedral_matrix_service::world::{Grant, Rect, World};
use cathedral_platform_contracts::matrix as wire;
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
    let mut server = Server::open_clients_from(
        if storage_root { 2 } else { 0 },
        if parent.separate_storage { 5 } else { 3 },
    )?;
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
    children.separate_storage = parent.separate_storage;
    static mut DEFERRED: deferred::Deferred = deferred::Deferred::empty();
    let deferred_address = &raw mut DEFERRED;
    // SAFETY: Same single-entry process lifetime as the parent/world arenas above.
    let deferred = unsafe { &mut *deferred_address };
    loop {
        parent.progress_storage(&mut server)?;
        parent.progress_presentation(world, &mut server)?;
        children.finish(&mut server, world, parent)?;
        if !parent.upstream_busy() {
            parent.refresh_notified(world)?;
            requests::flush(parent, world)?;
        }
        let (handle, mut deadline) = children.wait();
        children.notify(world)?;
        let mut extra = [cathedral_user_runtime::ipc::Handle::from_raw(0); 4];
        let mut count = 0;
        if parent.upstream_busy() || parent.storing() {
            deadline = None;
        }
        if let Some((receive, until)) = parent.storage_wait() {
            extra[count] = receive;
            count += 1;
            deadline = Some(until);
        }
        if let Some((receive, until)) = parent.presentation_wait() {
            extra[count] = receive;
            count += 1;
            // Ready owner replies still progress. Deferred teardown must not
            // spin on its expired deadline while publication owns the stream.
            deadline = Some(deadline.map_or(until, |d| {
                if cathedral_user_runtime::time::reached(d, until) {
                    until
                } else {
                    d
                }
            }));
        }
        if let Some(handle) = handle {
            extra[count] = handle;
            count += 1;
        }
        if !parent.upstream_busy()
            && let Some(wake) = parent.wake
        {
            extra[count] = wake;
            count += 1;
        }
        let queued = deferred.take(parent);
        deferred.pause(&mut server, parent);
        #[cfg(feature = "trace-storage")]
        let from_queue = queued.is_some();
        let event = match queued {
            Some(event) => event,
            None => match server.next_event_or(&extra[..count], deadline)? {
                Some(event) => event,
                None => continue,
            },
        };
        #[cfg(feature = "trace-storage")]
        if !from_queue
            && let Event::Request(r) = &event
            && r.client.is_some_and(|c| parent.storage_stream(c))
        {
            let op = wire::decode(&r.bytes[..r.len]).map_or(0, |w| w[0] & 0xffff);
            cathedral_user_runtime::time::storage_trace(
                if parent.upstream {
                    "nested-receive"
                } else {
                    "root-receive"
                },
                r.client.unwrap() as u64,
                op,
            );
        }
        if deferred::Deferred::blocks(&event, parent) {
            let mut blocked = 0;
            if parent.publishing() {
                blocked |= 1 << parent.publishing_client();
            }
            if parent.storing() {
                blocked |= 1 << parent.storing_client();
            }
            deferred.hold(event, &mut server, world, blocked)?;
            continue;
        }
        // Admitted children receive coalesced invalidations on their wake channel.
        // Legacy manually assembled two-endpoint fixtures still refresh on demand.
        if !parent.upstream_busy() {
            if parent.wake.is_none() {
                parent.refresh(world)?;
            } else {
                // A parent hint may arrive while waiting for a child request.
                parent.refresh_notified(world)?;
            }
        }
        match event {
            Event::Disconnected { client, .. } => {
                if client < 2 {
                    children.disconnected(client, &mut server, world, parent)?;
                } else if client == 2 {
                    world.cancel();
                } else {
                    parent.storage_close(parent.storage_slot(client))?;
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
