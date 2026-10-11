//! Refresh parent authority before every request, then dispatch local ownership.
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
        if let Some(slot) = children.finish(&mut server, &mut world)? {
            parent.erase(slot)?;
        }
        let (handle, deadline) = children.wait();
        let Some(event) = server.next_event_or(handle.as_slice(), deadline)? else {
            continue;
        };
        parent.refresh(&mut world)?;
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
