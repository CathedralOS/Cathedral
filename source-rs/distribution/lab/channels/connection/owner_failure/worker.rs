//! Keep a real shared service alive while its owner link fails at each phase.
use super::*;
use cathedral_user_runtime::server::{Event, Request, Server, SharedSlot};

pub(crate) fn run(env: Environment) -> u64 {
    let mut server = Server::open_clients(1).unwrap();
    server.enable_memory();
    server.isolate_control_failures();
    static mut OWNER: SharedSlot = SharedSlot::empty();
    static mut DATA: SharedSlot = SharedSlot::empty();
    let owner = &raw mut OWNER;
    let data = &raw mut DATA;
    // SAFETY: A worker enters once and owns this slot until process exit.
    server
        .enable_shared_control(unsafe { &mut *owner })
        .unwrap();
    // SAFETY: The independent data slot has the same single-entry lifetime.
    server.enable_shared(0, unsafe { &mut *data }).unwrap();
    server
        .install(0, (env.endpoint(0).unwrap(), env.endpoint(1).unwrap()))
        .unwrap();
    let mut held = None;
    loop {
        let Event::Request(request) = server.next_event().unwrap() else {
            panic!("unexpected service event")
        };
        let bytes = &request.bytes[..request.len];
        if request.control {
            assert!(server.control_connected());
            assert_eq!(
                server.reply(&request, &[0; 65]),
                Err(Error(abi::INVALID_ARGUMENT as i64))
            );
            assert!(server.control_connected());
            if bytes == b"hold" || bytes == b"hold-reply" {
                held = Some(request);
                if bytes == b"hold-reply" {
                    server.pause(1);
                }
            } else {
                server.reply(&request, bytes).unwrap();
            }
        } else {
            match bytes {
                b"state" => server
                    .reply(&request, &[u8::from(server.control_connected())])
                    .unwrap(),
                b"held" => server.reply(&request, &[u8::from(held.is_some())]).unwrap(),
                b"reply-now" => {
                    // Pausing owner reads makes reply publication, rather than
                    // polling, discover the closure. Both transports must isolate it.
                    assert!(server.control_connected());
                    let old = held.take().unwrap();
                    server.reply(&old, b"unreachable").unwrap();
                    assert!(!server.control_connected());
                    server.reply(&old, b"late again").unwrap();
                    server.pause(0);
                    server.reply(&request, bytes).unwrap();
                }
                b"finish" => {
                    assert!(!server.control_connected());
                    if let Some(old) = held {
                        abandoned(&mut server, old);
                    }
                    server.reply(&request, bytes).unwrap();
                    server.remove(0);
                    idle(&mut server);
                    return 0;
                }
                _ => server.reply(&request, bytes).unwrap(),
            }
        }
    }
}

fn abandoned(server: &mut Server, old: Request) {
    // An unexposed moved endpoint remains ours on a failed typed reply.
    let (send, receive) = ipc::channel().unwrap();
    assert_eq!(
        server.reply_endpoint(&old, b"late", send),
        Err(Error(abi::PEER_CLOSED as i64))
    );
    send.send(b"still ours").unwrap();
    assert_eq!(&super::super::receive(receive).0[..10], b"still ours");
    send.close().unwrap();
    receive.close().unwrap();
    server.reply(&old, b"abandoned").unwrap();

    let (send, input) = ipc::channel().unwrap();
    let (output, receive) = ipc::channel().unwrap();
    server.set_control((input, output));
    assert!(server.control_connected());
    server
        .reply(&old, b"never delivered to replacement")
        .unwrap();
    assert_eq!(
        receive.receive_until(&mut [0; 64], time::now().unwrap()),
        Err(Error(abi::TIMED_OUT as i64))
    );
    send.send(b"replacement").unwrap();
    let request = server.next_request().unwrap();
    assert!(request.control);
    server
        .reply(&request, &request.bytes[..request.len])
        .unwrap();
    assert_eq!(&super::super::receive(receive).0[..11], b"replacement");
    send.close().unwrap();
    receive.close().unwrap();
    assert!(
        server
            .next_event_for(&[], Some(time::after(2).unwrap()))
            .unwrap()
            .is_none()
    );
    assert!(!server.control_connected());
}

fn idle(server: &mut Server) {
    // No hidden permanently-ready closed handle, and no false INVALID_ARGUMENT
    // when the only remaining work is a caller's maintenance deadline.
    let deadline = time::after(2).unwrap();
    assert!(
        server
            .next_event_for(&[], Some(deadline))
            .unwrap()
            .is_none()
    );
    assert!(time::reached(time::now().unwrap(), deadline));
    let (send, receive) = ipc::channel().unwrap();
    receive.close().unwrap();
    assert!(
        matches!(server.next_event_for(&[Interest::Ipc(receive)], Some(time::after(2).unwrap())), Err(Error(c)) if c == abi::BAD_HANDLE as i64)
    );
    send.close().unwrap();
}
