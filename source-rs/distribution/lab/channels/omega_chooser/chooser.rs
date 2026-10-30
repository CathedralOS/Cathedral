// Exercise the standalone chooser across its storage and image custody seams.
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
    // Full page + continuation, then corrupt padding, UTF-8, row order and extent.
    for fault in 0..5 {
        let budget = cathedral_user_runtime::resources::budget().unwrap();
        let physical = cathedral_user_runtime::resources::physical_budget().unwrap();
        let bells = cathedral_user_runtime::notification::budget().unwrap();
        exercise(fault);
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
    cathedral_user_runtime::write(b"Omega chooser catalogue/control/images PASS\n").unwrap();
    0
}
fn exercise(fault: u64) {
    let (send, input) = ipc::channel().unwrap();
    let (output, receive) = ipc::channel().unwrap();
    let (storage_send, storage_input) = ipc::channel().unwrap();
    let (storage_output, storage_receive) = ipc::channel().unwrap();
    let mut offered = [Some(storage_send), Some(storage_receive)];
    let child = Launch::at(0)
        .unwrap()
        .spawn_with(1, &[send, receive])
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
    let mut pages = 0;
    let mut polls = 0;
    let mut images = 0;
    let mut surfaces = 0;
    let mut launches = 0;
    loop {
        let (request, mut object) = match server.next_event().unwrap() {
            Event::Request(request) => (request, None),
            Event::Memory(request, object) => (request, Some(object)),
            Event::Disconnected { client: 1, .. } if fault != 0 => continue,
            _ => panic!(
                "chooser disconnected: fault {fault}, polls {polls}, images {images}; child {:?}",
                child.wait_until(time::after(100).unwrap())
            ),
        };
        let words = matrix::decode(&request.bytes[..request.len]).unwrap();
        let reply = if words == services::request(imports::INTERACTION, 1, 1, 0) {
            server
                .reply_notification(
                    &request,
                    &matrix::encode([0, 1, 0, 0, 0, 0, 0, 0]),
                    rights[0],
                )
                .unwrap();
            continue;
        } else if words[0] == services::ACQUIRE {
            assert_eq!(words, services::request(imports::STORAGE, 1, 1, words[5]));
            server
                .reply_endpoint(
                    &request,
                    &matrix::encode([0, 1, 0, 0, 0, 0, 0, 0]),
                    offered[words[5] as usize].take().unwrap(),
                )
                .unwrap();
            continue;
        } else if request.client == Some(1) {
            // Root/provider expiry must leave room for its verdict to travel
            // back; equal client/provider deadlines retire the whole chain.
            let remaining = words[7].wrapping_sub(time::now().unwrap());
            assert!(remaining > matrix::STORAGE_DEADLINE_TICKS);
            assert!(remaining <= storage::EXCHANGE_TICKS);
            if words[0] == matrix::STORAGE_INFO {
                [0, 9, 17, 0, 0, 0, 0, 0]
            } else {
                assert_eq!(words[0] >> matrix::STORAGE_SCOPE_BITS, 9);
                assert_eq!((words[0] & 65535) - matrix::STORAGE_BASE, storage::ENTRIES);
                pages += 1;
                if pages == 1 {
                    assert_eq!(words[3], 0);
                    // Owner work progresses while the storage response is held.
                    assert_eq!(query(&mut owner, child, matrix::LOSS), [0; 8]);
                    assert_eq!(query(&mut owner, child, matrix::HEALTH), [0; 8]);
                    let mut page = cathedral_user_runtime::memory::Private::allocate(1).unwrap();
                    for i in 0..51 {
                        let metadata = storage::metadata::Metadata::new(
                            if i == 0 { b"Template" } else { b"User" },
                            storage::metadata::EXECUTABLE,
                        )
                        .unwrap();
                        let row = storage::Entry {
                            object: i as u64 + 1,
                            revision: 1,
                            metadata,
                        }
                        .encode();
                        page.bytes_mut()[i * 80..(i + 1) * 80].copy_from_slice(&row);
                    }
                    match fault {
                        1 => page.bytes_mut()[25] = b'x', // Data after Template's NUL.
                        2 => page.bytes_mut()[16..20].copy_from_slice(&[0xed, 0xa0, 0x80, 0]), // UTF-8 surrogate.
                        3 => page.bytes_mut()[80..88].copy_from_slice(&1u64.to_le_bytes()),
                        _ => {}
                    }
                    let page = page.freeze().unwrap();
                    let count = if fault == 4 { 52 } else { 51 };
                    server
                        .reply_memory(
                            &request,
                            &matrix::encode([0, count, 51, 0, 0, 0, 0, 0]),
                            &page,
                        )
                        .unwrap();
                    continue;
                }
                assert_eq!((fault, pages, words[3]), (0, 2, 51));
                [0, 0, 51, 0, 0, 0, 0, 0]
            }
        } else {
            match words[0] {
                matrix::POLL => {
                    polls += 1;
                    if polls > 1 && images > 0 {
                        assert_eq!(launches, u64::from(fault == 0));
                        assert_eq!(pages, if fault == 0 { 2 } else { 1 });
                        break; // Refresh must arrive without another notification.
                    }
                    if polls == 1 {
                        [0, 77, 640, 480, 0, 2, 5, 1] // Enter launches the selected first user.
                    } else {
                        // Launch completion causes an immediate poll before the
                        // initial scene. Do not inject Enter again on that poll.
                        [0, 77, 640, 480, 0, 0, 0, 0]
                    }
                }
                14 => {
                    // Distribution launch request.
                    assert_eq!(fault, 0, "malformed catalogue reached launch");
                    assert_eq!(words[1], 2);
                    launches += 1;
                    [0, 0, 99, 0, 0, 0, 0, 0]
                }
                matrix::LAUNCH_STATUS => [0, matrix::LAUNCH_RUNNING, 0, 0, 0, 0, 0, 0],
                matrix::SURFACE_CREATE => {
                    surfaces += 1;
                    [0, surfaces, 0, 0, 0, 0, 0, 0]
                }
                matrix::SURFACE_IMAGE => {
                    images += 1;
                    let pixels = object
                        .as_mut()
                        .expect("image backing missing")
                        .map()
                        .unwrap();
                    let extent = (words[5] * words[6] * 4) as usize;
                    assert!(extent > 4080 && extent <= 65536);
                    assert!(pixels[..extent].iter().any(|&b| b != 0));
                    assert!(pixels[extent..].iter().all(|&b| b == 0));
                    assert_eq!(query(&mut owner, child, matrix::LOSS), [0; 8]);
                    [0; 8]
                }
                matrix::PRESENT_RECTS | matrix::PAINT => [0; 8],
                op => panic!("unexpected chooser request {op}"),
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
