//! Refresh parent authority before every request, then dispatch local ownership.
mod parent;
mod requests;
use cathedral_contracts::{matrix as wire, user as abi};
use cathedral_matrix_service::world::{Grant, Rect, World};
use cathedral_user_runtime::{
    Error,
    server::{Event, Server},
};
use parent::Parent;

pub fn run(upstream: bool) -> Result<(), Error> {
    let mut parent = Parent::new(upstream);
    let mut world = World::default();
    let mut server = Server::open_clients_from(u64::from(upstream), 2)?;
    loop {
        let event = server.next_event()?;
        parent.refresh(&mut world)?;
        match event {
            Event::Disconnected { client, .. } => world.disconnected(client),
            Event::Request(request) => {
                let response = match wire::decode(&request.bytes[..request.len]) {
                    Some(words) if request.control => {
                        requests::owner(&mut parent, &mut world, words)
                    }
                    Some(words) => {
                        world.request(request.client.unwrap(), request.incarnation, words)
                    }
                    None => wire::error(abi::DENIED),
                };
                server.reply(&request, &wire::encode(response))?;
            }
        }
    }
}
