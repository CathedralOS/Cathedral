//! The normal Rust service handshake and exact acquisition packets. These cases
//! exercise the Omega consumer's new service policy, not another mock transport.
use cathedral_platform_contracts::{imports, matrix, services};
use cathedral_user_runtime::{
    event,
    notification::Notification,
    server::{Event, Server, SharedSlot},
    task::Environment,
    time,
};

pub(super) fn serve(mode: u64, environment: Environment) -> u64 {
    if mode >= 130 {
        return super::endpoints::serve(mode, environment);
    }
    let mut server = Server::open_clients(1).unwrap();
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
    // SAFETY: The single-entry fixture owns this slot until process exit.
    server.enable_shared(0, unsafe { &mut *shared }).unwrap();
    let rights = Notification::create().unwrap();
    let mut calls = 0;
    loop {
        match server.next_event().unwrap() {
            Event::Request(request) => {
                calls += 1;
                let words = matrix::decode(&request.bytes[..request.len]).unwrap();
                if calls == 1 && mode == 125 {
                    let _ = event::wait(&[], time::after(400).unwrap());
                }
                if calls == 1 && mode == 121 {
                    assert_eq!(words, services::request(imports::INTERACTION, 1, 1, 0));
                    server
                        .reply(
                            &request,
                            &matrix::encode([services::UNAVAILABLE, 0, 0, 0, 0, 0, 0, 0]),
                        )
                        .unwrap();
                } else if calls == 1 || (calls == 2 && mode == 121) {
                    assert_eq!(words, services::request(imports::INTERACTION, 1, 1, 0));
                    let mut reply = [0, 1, 0, 0, 0, 0, 0, 0];
                    if mode == 124 {
                        reply[7] = 9;
                    }
                    if mode == 122 {
                        server.reply(&request, &matrix::encode(reply)).unwrap();
                    } else {
                        server
                            .reply_notification(
                                &request,
                                &matrix::encode(reply),
                                rights[usize::from(mode == 123)],
                            )
                            .unwrap();
                    }
                } else if words[0] == services::ACQUIRE {
                    assert_eq!(words, services::request(imports::STORAGE, 1, 1, 0));
                    server
                        .reply(
                            &request,
                            &matrix::encode([services::UNAVAILABLE, 0, 0, 0, 0, 0, 0, 0]),
                        )
                        .unwrap();
                } else {
                    assert_eq!(words, [matrix::POLL, 0, 0, 0, 0, 0, 0, 0]);
                    server
                        .reply(&request, &matrix::encode([0, 77, 640, 480, 0, 0, 0, 0]))
                        .unwrap();
                }
            }
            Event::Disconnected { client: 0, .. } => break,
            _ => panic!("unexpected service acquisition event"),
        }
    }
    assert_eq!(
        calls,
        match mode {
            121 => 4,
            122..=124 => 1,
            _ => 3,
        }
    );
    rights[2].close_object().unwrap();
    for right in rights {
        right.release().unwrap();
    }
    0
}
