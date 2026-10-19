//! Choose this program's arrangement, then serve isolated peer connections.
use super::{children::Children, probes::Pending};
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
    let mut server = Server::open_clients(3)?;
    server.enable_memory();
    let mut children = Children([None, None, None]);
    // Admission installs no view. This program chooses virtual regions afterwards.
    for slot in 0..3 {
        children.start(slot as usize, 0, &mut server, world)?;
        if slot == 0 {
            children.prove_rollback(abi::BUSY)?;
        }
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
    children.prove_rollback(abi::NO_MEMORY)?;
    let mut pending: Option<Pending> = None;
    loop {
        if let Some(probe) = &pending
            && probe.progress(&mut server)?
        {
            pending = None;
        }
        server.pause(u8::from(pending.is_some()));
        let handles = pending.as_ref().map(|p| [p.wait().0]);
        let until = pending.as_ref().map(|p| p.wait().1);
        let Some(event) = server.next_event_or(handles.as_ref().map_or(&[], |h| &h[..]), until)?
        else {
            continue;
        };
        match event {
            Event::Disconnected { client, .. } => {
                if children.0[client].is_some() {
                    children.stop(client, &mut server, world)?;
                }
            }
            Event::Memory(request, object) => {
                drop(object);
                server.reply(&request, &wire::encode(wire::error(abi::DENIED)))?;
            }
            Event::Request(request) => {
                if !request.control
                    && let Some(words) = wire::decode(&request.bytes[..request.len])
                    && words[0] == cathedral_platform_contracts::services::ACQUIRE
                {
                    let child = children.0[request.client.unwrap()].as_mut().unwrap();
                    assert_eq!(child.data.0.raw(), request.incarnation);
                    child
                        .services
                        .as_mut()
                        .unwrap()
                        .reply(&mut server, &request, words)?;
                    continue;
                }
                let reply = match wire::decode(&request.bytes[..request.len]) {
                    Some(words) if request.control => {
                        if words[0] >> 32 != 0 {
                            pending = Some(Pending::start(request, words, &children)?);
                            continue;
                        } else if let [wire::STOP, slot @ 0..=2, 0, 0, 0, 0, 0, 0] = words {
                            wire::error(
                                children
                                    .stop(slot as usize, &mut server, world)
                                    .err()
                                    .map_or(0, |e| e.0 as u64),
                            )
                        } else if let [wire::SPAWN, slot @ 0..=2, mode @ 0..=2, 0, 0, 0, 0, 0] =
                            words
                        {
                            wire::error(
                                children
                                    .start(slot as usize, mode, &mut server, world)
                                    .err()
                                    .map_or(0, |e| e.0 as u64),
                            )
                        } else if matches!(words[0], wire::POINTER_SOURCE | wire::KEY_SOURCE) {
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
