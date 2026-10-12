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
    let mut world = World::default();
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
        parent.refresh_notified(&mut world)?;
        if let Some(slot) = children.finish(&mut server, &mut world)? {
            parent.erase(slot)?;
        }
        let (handle, deadline) = children.wait();
        children.notify(&mut world)?;
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
            parent.refresh(&mut world)?;
        } else {
            // A parent hint may arrive while waiting for a child request.
            parent.refresh_notified(&mut world)?;
        }
        match event {
            Event::Disconnected { client, .. } => {
                if client < 2 {
                    parent.erase(client)?;
                    children.disconnected(client, &mut server, &mut world)?;
                } else {
                    world.cancel();
                }
            }
            Event::Request(request) => {
                requests::dispatch(request, &mut parent, &mut world, &mut children, &mut server)?
            }
        }
    }
}
