//! Choose this program's arrangement, then serve isolated peer connections.
use cathedral_kernel_contracts::user as abi;
use cathedral_matrix::world::{Grant, World};
use cathedral_platform_contracts::matrix as wire;
use cathedral_user_runtime::{
    Error,
    server::{Event, Server},
};

pub fn run() -> Result<(), Error> {
    // Process-owned state avoids copying the bounded queues onto the user stack.
    static mut WORLD: World<3> = World::empty();
    let address = &raw mut WORLD;
    // SAFETY: One single-threaded entry, no other access to this arena.
    let world = unsafe { &mut *address };
    world.inherit(Grant {
        epoch: 1,
        width: 300,
        height: 600,
        keys: wire::KEY_MASK,
    });
    // These are virtual input regions. No framebuffer/device capability is held.
    for slot in 0..3 {
        assert_eq!(
            world.owner_command([
                wire::PLACE,
                slot,
                0,
                slot * 200,
                300,
                200,
                wire::KEY_MASK,
                0
            ]),
            [0; 8]
        );
    }
    let mut server = Server::open_clients(3)?;
    server.enable_memory();
    loop {
        match server.next_event()? {
            Event::Disconnected { client, .. } => world.disconnected(client),
            Event::Memory(request, object) => {
                drop(object);
                server.reply(&request, &wire::encode(wire::error(abi::DENIED)))?;
            }
            Event::Request(request) => {
                let reply = match wire::decode(&request.bytes[..request.len]) {
                    Some(words) if request.control => {
                        if matches!(words[0], wire::POINTER_SOURCE | wire::KEY_SOURCE) {
                            world.source_command(words)
                        } else {
                            world.owner_command(words)
                        }
                    }
                    Some(words) => {
                        world.request(request.client.unwrap(), request.incarnation, words)
                    }
                    None => wire::error(abi::DENIED),
                };
                server.reply(&request, &wire::encode(reply))?;
            }
        }
    }
}
