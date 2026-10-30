// Cross-language transport witness. The service is the existing Rust Server;
// the independently linked client lives in source/distribution/lab/user-entry/.
use cathedral_user_runtime::{
    ipc,
    notification::Notification,
    server::{Event, Server, SharedSlot},
    task::{Environment, Launch, Outcome},
    time,
};

pub fn run(role: u64, mode: u64, environment: Environment) -> u64 {
    if role == 1 {
        return serve(mode, environment);
    }
    for mode in [
        1, 2, 3, 94, 95, 96, 97, 98, 120, 121, 122, 123, 124, 125, 130, 131, 132, 133, 134, 135,
        136, 137, 138, 139, 140,
    ] {
        let baseline = cathedral_user_runtime::resources::budget().unwrap();
        let physical = cathedral_user_runtime::resources::physical_budget().unwrap();
        let notifications = cathedral_user_runtime::notification::budget().unwrap();
        let (send, input) = ipc::channel().unwrap();
        let (output, receive) = ipc::channel().unwrap();
        let provider = Launch::at(0)
            .unwrap()
            .spawn_with(mode, &[input, output])
            .unwrap();
        let client = Launch::at(1)
            .unwrap()
            .spawn_with(mode, &[send, receive])
            .unwrap();
        let outcome = client.wait_until(time::after(1000).unwrap()).unwrap();
        assert_eq!(outcome, Outcome::Returned(0), "Omega client mode {mode}");
        if mode == 3 {
            provider.cancel().unwrap();
            assert_eq!(provider.wait().unwrap(), Outcome::Cancelled);
        } else {
            assert_eq!(
                provider.wait_until(time::after(300).unwrap()).unwrap(),
                Outcome::Returned(0)
            );
        }
        assert_eq!(
            cathedral_user_runtime::resources::budget().unwrap(),
            baseline
        );
        assert_eq!(
            cathedral_user_runtime::resources::physical_budget().unwrap(),
            physical
        );
        assert_eq!(
            cathedral_user_runtime::notification::budget().unwrap(),
            notifications
        );
    }
    cathedral_user_runtime::write(b"Omega service acquisition PASS\n").unwrap();
    owners();
    cathedral_user_runtime::write(b"Omega shared connection PASS\n").unwrap();
    0
}

fn serve(mode: u64, environment: Environment) -> u64 {
    if mode >= 120 {
        return services::serve(mode, environment);
    }
    if (94..=98).contains(&mode) {
        return super::connection::worker(mode, environment);
    }
    let input = environment.endpoint(0).unwrap();
    let output = environment.endpoint(1).unwrap();
    if mode != 1 {
        let mut bytes = [0; 64];
        let n = input.receive(&mut bytes).unwrap();
        assert_eq!(&bytes[..n], cathedral_platform_contracts::connection::OPEN);
        if mode == 2 {
            output.send(b"wrong handshake").unwrap();
            return 0;
        }
        // Keep the valid peer alive but never acknowledge setup. The client
        // must expire and release its partial connection without help from death.
        let (_, idle) = ipc::channel().unwrap();
        let _ = idle.receive(&mut bytes);
        return 1;
    }
    let mut server = Server::open_clients(1).unwrap();
    server.install(0, (input, output)).unwrap();
    static mut SHARED: SharedSlot = SharedSlot::empty();
    let shared = &raw mut SHARED;
    // SAFETY: This single-entry fixture owns the slot until process exit.
    server.enable_shared(0, unsafe { &mut *shared }).unwrap();
    let rights = Notification::create().unwrap();
    let mut count = 0;
    loop {
        match server.next_event().unwrap() {
            Event::Request(request) => {
                assert!(!request.control);
                assert_eq!(request.len, 64);
                count += 1;
                assert_eq!(request.bytes[0], count);
                if count % 2 == 0 {
                    server
                        .reply_notification(&request, &request.bytes, rights[0])
                        .unwrap();
                } else {
                    server.reply(&request, &request.bytes).unwrap();
                }
            }
            Event::Disconnected { client: 0, .. } => break,
            _ => panic!("unexpected Omega client event"),
        }
    }
    assert_eq!(count, 16);
    rights[2].close_object().unwrap();
    for right in rights {
        right.release().unwrap();
    }
    0
}

// The owner connects through the kernel-installed control pair, independently
// of the child's environment broker. No alternate copied request path is used.
fn owners() {
    use cathedral_platform_contracts::connection as wire;
    use cathedral_user_runtime::{connection::Client, task::Port};
    for mode in [110, 111, 112, 113, 114] {
        let budget = cathedral_user_runtime::resources::budget().unwrap();
        let physical = cathedral_user_runtime::resources::physical_budget().unwrap();
        let bells = cathedral_user_runtime::notification::budget().unwrap();
        let child = Launch::at(1).unwrap().spawn(mode).unwrap();
        let pair = Port::at(1).unwrap().connect().unwrap();
        match mode {
            111 => pair.0.send(b"copied requests are not setup").unwrap(),
            112 | 114 => {
                pair.0.send(wire::OPEN).unwrap();
                let mut bytes = [0; 64];
                let n = pair
                    .1
                    .receive_until(&mut bytes, time::after(300).unwrap())
                    .unwrap();
                assert_eq!(&bytes[..n], wire::ACK);
                if mode == 114 {
                    pair.0.send(b"not a reader grant").unwrap();
                }
            }
            _ => {
                let mut client = Client::connect(pair, time::after(300).unwrap()).unwrap();
                if mode == 113 {
                    pair.0.send(b"no copied fallback").unwrap();
                } else {
                    for index in 1u64..=16 {
                        let mut bytes = [0u8; 64];
                        bytes[..8].copy_from_slice(&index.to_le_bytes());
                        bytes[56..].copy_from_slice(&index.to_le_bytes());
                        let (reply, n) = client.call(&bytes, time::after(300).unwrap()).unwrap();
                        assert_eq!(n, 64);
                        assert_eq!(reply, bytes);
                    }
                }
                assert_eq!(
                    child.wait_until(time::after(1000).unwrap()).unwrap(),
                    Outcome::Returned(0),
                    "Omega owner mode {mode}"
                );
                client.retire();
            }
        }
        if (111..=114).contains(&mode) && mode != 113 {
            assert_eq!(
                child.wait_until(time::after(1000).unwrap()).unwrap(),
                Outcome::Returned(0),
                "Omega setup mode {mode}"
            );
            pair.0.close().ok();
            pair.1.close().ok();
        }
        assert_eq!(cathedral_user_runtime::resources::budget().unwrap(), budget);
        assert_eq!(
            cathedral_user_runtime::resources::physical_budget().unwrap(),
            physical
        );
        assert_eq!(
            cathedral_user_runtime::notification::budget().unwrap(),
            bells
        );
    }
    cathedral_user_runtime::write(b"Omega owner connection PASS\n").unwrap();
}
