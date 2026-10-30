//! Endpoint custody: a usable pair, second-part refusal, and malformed grants.
use cathedral_platform_contracts::{imports, matrix, services};
use cathedral_user_runtime::{
    ipc,
    notification::Notification,
    server::{Event, Server, SharedSlot},
    task::Environment,
};

pub(super) fn serve(mode: u64, environment: Environment) -> u64 {
    let mut server = Server::open_clients(1).unwrap();
    if mode == 135 {
        server.enable_memory();
    }
    server
        .install(
            0,
            (
                environment.endpoint(0).unwrap(),
                environment.endpoint(1).unwrap(),
            ),
        )
        .unwrap();
    static mut SHARED: SharedSlot = SharedSlot::empty();
    let shared = &raw mut SHARED;
    // SAFETY: This single-entry fixture exclusively owns the slot through exit.
    server.enable_shared(0, unsafe { &mut *shared }).unwrap();
    let (sender, receiver) = ipc::channel().unwrap();
    let mut endpoints = [Some(sender), Some(receiver)];
    let rights = Notification::create().unwrap();
    let mut calls = 0;
    loop {
        match server.next_event().unwrap() {
            Event::Request(request) => {
                let words = matrix::decode(&request.bytes[..request.len]).unwrap();
                assert!(calls < 2);
                assert_eq!(words, services::request(imports::STORAGE, 1, 1, calls));
                let part = calls as usize;
                calls += 1;
                let mut reply = [0, 1, 0, 0, 0, 0, 0, 0];
                match mode {
                    131 if part == 1 => {
                        reply = [services::UNAVAILABLE, 0, 0, 0, 0, 0, 0, 0];
                        server.reply(&request, &matrix::encode(reply)).unwrap();
                    }
                    132 => server
                        .reply_notification(&request, &matrix::encode(reply), rights[0])
                        .unwrap(),
                    134 => server.reply(&request, &matrix::encode(reply)).unwrap(),
                    _ => {
                        if mode == 133 {
                            reply[7] = 1;
                        }
                        server
                            .reply_endpoint(
                                &request,
                                &matrix::encode(reply),
                                endpoints[part].take().unwrap(),
                            )
                            .unwrap();
                    }
                }
            }
            Event::Memory(request, mut object) => {
                assert_eq!(mode, 135);
                assert_eq!(calls, 0);
                calls += 1;
                let words = matrix::decode(&request.bytes[..request.len]).unwrap();
                assert_eq!(words, [135, 0, 0, 0, 0, 0, 0, 0]);
                let bytes = object.map().unwrap();
                assert_eq!(&bytes[..6], &[0xbc, 0x9a, 0x78, 0x56, 0x34, 0x12]);
                assert!(bytes[6..].iter().all(|&byte| byte == 0));
                server.reply(&request, &matrix::encode([0; 8])).unwrap();
            }
            Event::Disconnected { client: 0, .. } => break,
            _ => panic!("unexpected endpoint acquisition event"),
        }
    }
    assert_eq!(calls, if mode <= 131 { 2 } else { 1 });
    for endpoint in endpoints.into_iter().flatten() {
        endpoint.close().unwrap();
    }
    rights[2].close_object().unwrap();
    for right in rights {
        right.release().unwrap();
    }
    0
}
