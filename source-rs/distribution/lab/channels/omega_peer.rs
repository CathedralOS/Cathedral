//! Cross-language transport witness. The service is the existing Rust Server;
//! the independently linked client lives in source/distribution/lab/user-entry/.
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
    for mode in [1, 2, 3, 94, 95, 96, 97, 98] {
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
    cathedral_user_runtime::write(b"Omega shared connection PASS\n").unwrap();
    0
}

fn serve(mode: u64, environment: Environment) -> u64 {
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
