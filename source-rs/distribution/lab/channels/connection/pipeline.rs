//! Delayed rendering must not hold input behind its reply; bounded publication
//! and reply queues retain custody through pressure, out-of-order completion and death.
use super::*;
use cathedral_user_runtime::{
    connection::Pipeline,
    memory::{Object, Private},
    server::{Event, Request, Server, SharedSlot},
};
type Calls = Pipeline<4>;
fn submit(client: &mut Calls, bytes: &[u8], object: Option<&Object>) -> u64 {
    let until = time::after(300).unwrap();
    loop {
        match client.submit(bytes, object) {
            Ok(id) => return id,
            Err(Error(c)) if c == abi::WOULD_BLOCK as i64 => {
                assert!(!time::reached(time::now().unwrap(), until));
                event::wait(&client.interests().unwrap(), until).unwrap();
            }
            Err(e) => panic!("pipeline submit: {e:?}"),
        }
    }
}
fn reply(client: &mut Calls) -> (u64, cathedral_user_runtime::connection::Response) {
    let until = time::after(300).unwrap();
    loop {
        if let Some(reply) = client.poll_reply().unwrap() {
            return reply;
        }
        assert!(!time::reached(time::now().unwrap(), until));
        event::wait(&client.interests().unwrap(), until).unwrap();
    }
}
pub(super) fn run(launch: Launch) {
    resources::assign(
        launch,
        Limits {
            private_pages: 2,
            reader_pages: 8,
            channels: 8,
        },
    )
    .unwrap();
    for mode in 75..=77 {
        let (send, input) = ipc::channel().unwrap();
        let (output, recv) = ipc::channel().unwrap();
        let child = launch.spawn_with(mode, &[input, output]).unwrap();
        let (control, events) = Port::bootstrap().unwrap().connect().unwrap();
        let connected = Calls::connect((send, recv), time::after(300).unwrap());
        if mode == 77 {
            assert!(matches!(connected, Err(Error(c)) if c == abi::PEER_CLOSED as i64));
            assert_eq!(
                child.wait_until(time::after(300).unwrap()).unwrap(),
                Outcome::Returned(0)
            );
            continue;
        }
        let mut client = connected.unwrap();
        let objects: [_; 4] = core::array::from_fn(|i| {
            let mut page = Private::allocate(1).unwrap();
            page.bytes_mut().fill(i as u8);
            page.freeze().unwrap()
        });
        if mode == 76 {
            submit(&mut client, b"die", Some(&objects[0]));
            submit(&mut client, b"die", Some(&objects[1]));
            assert_eq!(
                child.wait_until(time::after(300).unwrap()).unwrap(),
                Outcome::Returned(0)
            );
            assert!(matches!(client.poll_reply(), Err(Error(c)) if c == abi::PEER_CLOSED as i64));
            assert_eq!(
                client.submit(b"no replay", None),
                Err(Error(abi::PEER_CLOSED as i64))
            );
            continue;
        }
        client.begin(b"pause", None).unwrap();
        assert_eq!(&reply(&mut client).1.0[..6], b"paused");
        for round in 0..16u8 {
            let mut ids = [0; 4];
            ids[0] = submit(&mut client, &[round, 0], Some(&objects[0]));
            if round == 0 {
                // Server has paused this dispatch slot. Refusal must not publish
                // an attachment or consume a correlation ID.
                let before = resources::budget().unwrap();
                assert_eq!(
                    client.submit(&[round, 1], Some(&objects[1])),
                    Err(Error(abi::WOULD_BLOCK as i64))
                );
                assert_eq!(resources::budget().unwrap(), before);
                control.send(b"resume").unwrap();
                assert_eq!(&receive(events).0[..7], b"resumed");
            }
            for i in 1..4 {
                ids[i] = submit(&mut client, &[round, i as u8], Some(&objects[i]));
            }
            assert_eq!(ids, core::array::from_fn(|i| ids[0] + i as u64));
            assert_eq!(client.submit(b"full", None), Err(Error(abi::BUSY as i64)));
            assert_eq!(&receive(events).0[..6], b"queued");
            let mut seen = [false; 4];
            for _ in 0..4 {
                let (id, (bytes, len, object)) = reply(&mut client);
                let i = ids.iter().position(|candidate| *candidate == id).unwrap();
                assert!(!seen[i]);
                seen[i] = true;
                assert_eq!(&bytes[..len], &[round, i as u8]);
                if i == 3 {
                    let mut object = object.unwrap();
                    assert!(object.map().unwrap().iter().all(|b| *b == 3));
                } else {
                    assert!(object.is_none());
                }
            }
            assert!(!client.pending());
            let render = submit(&mut client, b"render", None);
            let input = submit(&mut client, b"input", None);
            let (id, (bytes, len, _)) = reply(&mut client);
            assert_eq!(id, input);
            assert_eq!(&bytes[..len], b"input");
            assert!(client.pending()); // Rendering still has no reply.
            control.send(b"release").unwrap();
            assert_eq!(reply(&mut client).0, render);
        }
        client.retire();
        assert_eq!(
            child.wait_until(time::after(300).unwrap()).unwrap(),
            Outcome::Returned(0)
        );
    }
    cathedral_user_runtime::write(b"Cathedral shared pipeline: delayed reply isolation, four in-flight calls, mailbox backpressure, queued replies, attachment custody, peer death and unselected pipeline refusal passed\n").unwrap();
}
fn request(server: &mut Server) -> (Request, Option<Object>) {
    match server.next_event().unwrap() {
        Event::Request(r) => (r, None),
        Event::Memory(r, o) => (r, Some(o)),
        _ => panic!("request"),
    }
}
pub(super) fn worker(mode: u64, env: Environment) -> u64 {
    let mut server = Server::open_clients(1).unwrap();
    static mut SLOT: SharedSlot = SharedSlot::empty();
    let slot = &raw mut SLOT;
    // SAFETY: One task entry lends its static connection state for this server's lifetime.
    let slot = unsafe { &mut *slot };
    if mode == 77 {
        server.enable_shared(0, slot).unwrap();
    } else {
        server.enable_pipeline(0, slot).unwrap();
    }
    server.enable_memory();
    server
        .install(0, (env.endpoint(0).unwrap(), env.endpoint(1).unwrap()))
        .unwrap();
    if mode == 77 {
        assert!(matches!(
            server.next_event().unwrap(),
            Event::Disconnected { .. }
        ));
        return 0;
    }
    if mode == 76 {
        let _a = request(&mut server);
        let _b = request(&mut server);
        return 0;
    }
    let (pause, _) = request(&mut server);
    server.reply(&pause, b"paused").unwrap();
    server.pause(2);
    let (resume, _) = request(&mut server);
    assert!(resume.control);
    server.reply(&resume, b"resumed").unwrap();
    server.pause(0);
    for round in 0..16u8 {
        let mut held = [const { None }; 4];
        for (i, slot) in held.iter_mut().enumerate() {
            let (r, object) = request(&mut server);
            assert_eq!(&r.bytes[..r.len], &[round, i as u8]);
            let mut object = object.unwrap();
            assert!(object.map().unwrap().iter().all(|b| *b == i as u8));
            *slot = Some((r, object));
        }
        for i in [3, 1, 2, 0] {
            let r = held[i].as_ref().unwrap().0;
            if i == 3 {
                server
                    .reply_memory(&r, &r.bytes[..r.len], &held[i].as_ref().unwrap().1)
                    .unwrap();
            } else {
                server.reply(&r, &r.bytes[..r.len]).unwrap();
            }
            server.reply(&r, b"duplicate must not publish").unwrap();
        }
        Handle::bootstrap(1).unwrap().send(b"queued").unwrap();
        drop(held);
        let (render, _) = request(&mut server);
        assert_eq!(&render.bytes[..render.len], b"render");
        let (input, _) = request(&mut server);
        server.reply(&input, b"input").unwrap();
        let (release, _) = request(&mut server);
        assert!(release.control);
        server.reply(&render, b"render").unwrap();
    }
    assert!(matches!(
        server.next_event().unwrap(),
        Event::Disconnected { .. }
    ));
    0
}
