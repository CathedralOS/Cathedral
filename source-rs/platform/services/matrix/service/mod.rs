//! Consume local invalidations, wake children, then dispatch ownership and input.
mod children;
mod parent;
mod requests;
use cathedral_contracts::{matrix as wire, user as abi};
use cathedral_matrix_service::world::{Grant, Rect, World};
use cathedral_user_runtime::{
    Error,
    server::{Event, Server},
    task::Environment,
};
use parent::Parent;

pub fn run(environment: Environment) -> Result<(), Error> {
    let mut parent = Parent::new(&environment)?;
    static mut WORLD: World = World::empty();
    let world_address = &raw mut WORLD;
    // SAFETY: Single-threaded executable enters this service exactly once. This
    // process-owned arena has no aliases and never occupies the guarded user stack.
    let world = unsafe { &mut *world_address };
    let mut server = Server::open_clients(3)?;
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
    loop {
        parent.refresh_notified(world)?;
        children.finish(&mut server, world)?;
        requests::flush(&parent, world)?;
        let (handle, deadline) = children.wait();
        children.notify(world)?;
        let mut extra = [cathedral_user_runtime::ipc::Handle::from_raw(0); 2];
        let mut count = 0;
        for h in [handle, parent.wake].into_iter().flatten() {
            extra[count] = h;
            count += 1;
        }
        let Some(event) = server.next_event_or(&extra[..count], deadline)? else {
            continue;
        };
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
                    children.disconnected(client, &mut server, world)?;
                } else {
                    world.cancel();
                }
            }
            Event::Request(request) => {
                requests::dispatch(request, &mut parent, world, &mut children, &mut server)?
            }
        }
    }
}
