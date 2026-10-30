// Hold a real Background broker request while querying its installed owner link.
// A presentation dependency must not prevent the app from answering safe-close.
use cathedral_platform_contracts::{imports, matrix, services};
use cathedral_user_runtime::{
    connection::Client,
    ipc,
    notification::Notification,
    server::{Event, Server, SharedSlot},
    task::{Launch, Outcome, Port},
    time,
};

pub fn run() -> u64 {
    for retire_owner in [false, true] {
        let budget = cathedral_user_runtime::resources::budget().unwrap();
        let physical = cathedral_user_runtime::resources::physical_budget().unwrap();
        let bells = cathedral_user_runtime::notification::budget().unwrap();
        exercise(retire_owner);
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
    cathedral_user_runtime::write(b"Omega Background independent control PASS\n").unwrap();
    0
}

fn exercise(retire_owner: bool) {
    let (send, input) = ipc::channel().unwrap();
    let (output, receive) = ipc::channel().unwrap();
    let child = Launch::at(0)
        .unwrap()
        .spawn_with(0, &[send, receive])
        .unwrap();
    let mut owner = Client::connect(
        Port::at(0).unwrap().connect().unwrap(),
        time::after(300).unwrap(),
    )
    .unwrap();
    let mut server = Server::open_clients(1).unwrap();
    // This fixture is the initial process, with no owner service above it.
    server.pause(1);
    server.install(0, (input, output)).unwrap();
    static mut SHARED: SharedSlot = SharedSlot::empty();
    let shared = &raw mut SHARED;
    // SAFETY: Each exercise owns this slot through server drop, before reuse.
    server.enable_shared(0, unsafe { &mut *shared }).unwrap();
    let rights = Notification::create().unwrap();
    let mut polls = 0;
    loop {
        let Event::Request(request) = server.next_event().unwrap() else {
            panic!("Background disconnected before control checks");
        };
        let words = matrix::decode(&request.bytes[..request.len]).unwrap();
        if words == services::request(imports::INTERACTION, 1, 1, 0) {
            server
                .reply_notification(
                    &request,
                    &matrix::encode([0, 1, 0, 0, 0, 0, 0, 0]),
                    rights[0],
                )
                .unwrap();
        } else if words == services::request(imports::STORAGE, 1, 1, 0) {
            server
                .reply(
                    &request,
                    &matrix::encode([services::UNAVAILABLE, 0, 0, 0, 0, 0, 0, 0]),
                )
                .unwrap();
        } else if words[0] == matrix::POLL {
            polls += 1;
            if polls == 3 {
                assert!(retire_owner);
                break; // The live broker progressed after owner retirement.
            }
            // The broker stays deliberately unanswered until owner replies.
            // A whole second allows slow emulation but cannot repair a cycle.
            for (operation, tail, status) in [
                (matrix::LOSS, 0, 0),
                (matrix::HEALTH, 0, 0),
                (matrix::LOSS, 1, cathedral_kernel_contracts::user::DENIED),
            ] {
                let (bytes, length) = owner
                    .call(
                        &matrix::encode([operation, 0, 0, 0, 0, 0, 0, tail]),
                        time::after(1000).unwrap(),
                    )
                    .expect("owner reply blocked behind presentation");
                assert_eq!(length, 64);
                assert_eq!(matrix::decode(&bytes), Some([status, 0, 0, 0, 0, 0, 0, 0]));
            }
            if polls == 2 {
                if !retire_owner {
                    break;
                }
                owner.retire();
            }
            server
                .reply(&request, &matrix::encode([0, 77, 640, 480, 0, 0, 0, 0]))
                .unwrap();
        } else if words[0] == matrix::SURFACE_CREATE {
            server
                .reply(&request, &matrix::encode([0, 1, 0, 0, 0, 0, 0, 0]))
                .unwrap();
        } else {
            assert_eq!(words[0], matrix::PAINT);
            server.reply(&request, &matrix::encode([0; 8])).unwrap();
            // Its periodic refresh is 100 ticks. It must park before that,
            // and an actual interaction signal must wake it promptly.
            assert!(
                server
                    .next_event_for(&[], Some(time::after(20).unwrap()))
                    .unwrap()
                    .is_none(),
                "idle Background polled without a wake"
            );
            rights[1].signal(false).unwrap();
        }
    }
    child.cancel().unwrap();
    assert_eq!(child.wait().unwrap(), Outcome::Cancelled);
    owner.retire();
    server.remove(0);
    rights[2].close_object().unwrap();
    for right in rights {
        right.release().unwrap();
    }
}
