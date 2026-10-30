// Drive the real Omega Notes process, load a sealed record, hold and fail its save.
// Control must progress without replaying the mutation or discarding its loss.
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::{imports, matrix, services, storage};
use cathedral_user_runtime::{
    connection::Client,
    ipc,
    notification::Notification,
    server::{Event, Server, SharedSlot},
    task::{Launch, Outcome, Port},
    time,
};

pub fn run() -> u64 {
    for (retire_owner, full_document) in [(false, false), (true, false), (false, true)] {
        let budget = cathedral_user_runtime::resources::budget().unwrap();
        let physical = cathedral_user_runtime::resources::physical_budget().unwrap();
        let bells = cathedral_user_runtime::notification::budget().unwrap();
        exercise(retire_owner, full_document);
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
    cathedral_user_runtime::write(b"Omega Notes storage/control/images PASS\n").unwrap();
    0
}

fn exercise(retire_owner: bool, full_document: bool) {
    let saved_length = if full_document { 239 } else { 3 };
    let record_length = saved_length + 4;
    let edited_length = saved_length + 1;
    let (send, input) = ipc::channel().unwrap();
    let (output, receive) = ipc::channel().unwrap();
    let (storage_send, storage_input) = ipc::channel().unwrap();
    let (storage_output, storage_receive) = ipc::channel().unwrap();
    let mut offered = [Some(storage_send), Some(storage_receive)];
    let child = Launch::at(0)
        .unwrap()
        .spawn_with(0, &[send, receive])
        .unwrap();
    let mut owner = Client::connect(
        Port::at(0).unwrap().connect().unwrap(),
        time::after(300).unwrap(),
    )
    .unwrap();
    let mut server = Server::open_clients(2).unwrap();
    server.enable_memory();
    server.pause(1);
    server.install(0, (input, output)).unwrap();
    server.install(1, (storage_input, storage_output)).unwrap();
    static mut SHARED: [SharedSlot; 2] = [const { SharedSlot::empty() }; 2];
    let slots = &raw mut SHARED;
    // SAFETY: Each exercise exclusively owns both slots through server drop.
    let slots = unsafe { &mut *slots };
    let (first, second) = slots.split_at_mut(1);
    server.enable_shared(0, &mut first[0]).unwrap();
    server.enable_shared(1, &mut second[0]).unwrap();
    let rights = Notification::create().unwrap();
    let mut saves = 0;
    let mut polls = 0;
    let mut images = 0;
    let mut surfaces = 0;
    loop {
        let (request, mut object) = match server.next_event().unwrap() {
            Event::Request(request) => (request, None),
            Event::Memory(request, object) => (request, Some(object)),
            _ => panic!("Notes disconnected after {polls} polls / {images} images; child {:?}", child.wait_until(time::after(100).unwrap())),
        };
        let words = matrix::decode(&request.bytes[..request.len]).unwrap();
        let reply = if words == services::request(imports::INTERACTION, 1, 1, 0) {
            server
                .reply_notification(
                    &request,
                    &matrix::encode([0, 1, 0, 0, 0, 0, 0, 0]),
                    rights[0],
                )
                .unwrap_or_else(|error| panic!("interaction {error:?}; child {:?}", child.wait_until(time::after(100).unwrap())));
            continue;
        } else if words[0] == services::ACQUIRE {
            assert_eq!(words, services::request(imports::STORAGE, 1, 1, words[5]));
            let part = usize::try_from(words[5]).unwrap();
            server
                .reply_endpoint(
                    &request,
                    &matrix::encode([0, 1, 0, 0, 0, 0, 0, 0]),
                    offered[part].take().unwrap(),
                )
                .unwrap();
            continue;
        } else if request.client == Some(1) {
            assert_ne!(words[7], 0);
            if words[0] == matrix::STORAGE_INFO {
                [0, 9, 17, 0, 0, 0, 0, 0]
            } else {
                assert_eq!(words[0] >> matrix::STORAGE_SCOPE_BITS, 9);
                match (words[0] & 65535) - matrix::STORAGE_BASE {
                    storage::LIST => [0, 1, 42, 0, 0, 0, 0, 0],
                    storage::HEAD => [0, 42, 1, record_length as u64, 0, 0, 0, 0],
                    storage::BULK => {
                        let mut page = cathedral_user_runtime::memory::Private::allocate(1).unwrap();
                        page.bytes_mut()[..4].copy_from_slice(b"NTS1");
                        if full_document {
                            page.bytes_mut()[4..record_length].fill(b'a');
                        } else {
                            page.bytes_mut()[4..record_length].copy_from_slice(b"abc");
                        }
                        let document = page.freeze().unwrap();
                        server.reply_memory(&request, &matrix::encode([0, 42, 1, 0, record_length as u64, 0, 0, 0]), &document).unwrap();
                        continue;
                    },
                    storage::PUBLISH => {
                        saves += 1;
                        assert_eq!(saves, 1, "save replayed");
                        let bytes = object.as_mut().unwrap().map().unwrap();
                        assert_eq!(&bytes[..4], b"NTS1");
                        if full_document {
                            assert!(bytes[4..record_length].iter().all(|&b| b == b'a'));
                        } else {
                            assert_eq!(&bytes[4..record_length], b"abc");
                        }
                        assert_eq!(bytes[record_length], b'x');
                        assert!(bytes[record_length + 1..].iter().all(|&b| b == 0));
                        assert_eq!(query(&mut owner, child, matrix::LOSS)[1], 1);
                        assert_eq!(query(&mut owner, child, matrix::HEALTH), [0, 1, edited_length as u64, 0, 1, 1, 1, 0]);
                        if retire_owner {
                            owner.retire();
                        }
                        [abi::IO_ERROR, 0, 0, 0, 0, 0, 0, 0]
                    }
                    op => panic!("unexpected storage operation {op}"),
                }
            }
        } else {
            match words[0] {
                matrix::POLL => {
                    polls += 1;
                    // More than one drain batch is already queued. No new bell
                    // arrives: the MORE flag must keep the app draining.
                    if polls == 1 {
                        [0, 77, 640, 480, 0, 2, 38, 1] // x retained across document load
                    } else if polls <= 11 {
                        [0, 77, 640, 480, 0, matrix::MORE, 0, 0]
                    } else {
                        assert_eq!(polls, 12);
                        [0, 77, 640, 480, 0, 2, 6, 1] // F1 save
                    }
                }
                matrix::SURFACE_CREATE => {
                    surfaces += 1;
                    [0, surfaces, 0, 0, 0, 0, 0, 0]
                }
                matrix::SURFACE_IMAGE => {
                    images += 1;
                    let bytes = object.as_mut().unwrap().map().unwrap();
                    assert!(bytes.iter().any(|&byte| byte != 0));
                    if words[5] == 124 && saves == 0 {
                        assert_eq!((polls, images), (1, 3));
                        server.reply(&request, &matrix::encode([0; 8])).unwrap();
                        // Idle means parked. A timeout must not manufacture a
                        // poll, while one actual signal must resume the app.
                        assert!(
                            server
                                .next_event_for(&[], Some(time::after(100).unwrap()))
                                .unwrap()
                                .is_none(),
                            "idle Notes polled without a wake"
                        );
                        rights[1].signal(false).unwrap();
                        continue;
                    }
                    if saves == 1 {
                        assert_eq!(saves, 1);
                        if !retire_owner {
                            assert_eq!(query(&mut owner, child, matrix::LOSS)[1], 1);
                            assert_eq!(query(&mut owner, child, matrix::HEALTH), [0, 1, edited_length as u64, abi::IO_ERROR, 1, 1, 1, 0]);
                        }
                        break; // Cancellation while holding a published image.
                    }
                    assert!(matches!((words[5], words[6]), (180, 64) | (20, 16) | (124, 8)));
                    [0; 8]
                }
                matrix::PRESENT_RECTS | matrix::PAINT => [0; 8],
                op => panic!("unexpected presentation operation {op}"),
            }
        };
        server.reply(&request, &matrix::encode(reply)).unwrap();
    }
    child.cancel().unwrap();
    assert_eq!(child.wait().unwrap(), Outcome::Cancelled);
    owner.retire();
    server.remove(0);
    server.remove(1);
    for endpoint in offered.into_iter().flatten() {
        endpoint.close().unwrap();
    }
    rights[2].close_object().unwrap();
    for right in rights {
        right.release().unwrap();
    }
}

fn query(
    owner: &mut Client,
    child: cathedral_user_runtime::task::Child,
    operation: u64,
) -> matrix::Packet {
    let (bytes, length) = owner
        .call(
            &matrix::encode([operation, 0, 0, 0, 0, 0, 0, 0]),
            time::after(600).unwrap(),
        )
        .unwrap_or_else(|error| {
            panic!(
                "owner operation {operation} failed {error:?}; child {:?}",
                child.wait_until(time::after(100).unwrap())
            )
        });
    assert_eq!(length, 64);
    matrix::decode(&bytes).unwrap()
}
