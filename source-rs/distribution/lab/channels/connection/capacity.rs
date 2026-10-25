//! Five live streams fit the unchanged handle cap by releasing setup-only rights.
use super::*;
use cathedral_user_runtime::server::{Event, Server, SharedSlot};
pub(super) fn run(launch: Launch) {
    resources::assign(
        launch,
        Limits {
            private_pages: 5,
            reader_pages: 5,
            channels: 8,
        },
    )
    .unwrap();
    let pairs: [_; 4] = core::array::from_fn(|_| {
        let (send, input) = ipc::channel().unwrap();
        let (output, receive) = ipc::channel().unwrap();
        ((send, receive), (input, output))
    });
    let child = launch
        .spawn_with(74, &[pairs[0].1.0, pairs[0].1.1])
        .unwrap();
    let control_pair = Port::bootstrap().unwrap().connect().unwrap();
    for (_, (input, output)) in &pairs[1..] {
        for endpoint in [*input, *output] {
            control_pair.0.send_with(b"endpoint", endpoint).unwrap();
            assert_eq!(&receive(control_pair.1).0[..2], b"ok");
        }
    }
    let mut clients =
        pairs.map(|(pair, _)| Client::connect(pair, time::after(300).unwrap()).unwrap());
    let mut control = Client::connect(control_pair, time::after(300).unwrap()).unwrap();
    assert_eq!(
        cathedral_user_runtime::notification::budget().unwrap()[3],
        15
    );
    // Fifteen retained rights leave one slot, insufficient to create another trio.
    assert_eq!(N::create(), Err(Error(abi::NO_MEMORY as i64)));
    for round in 0..3u8 {
        for (index, client) in clients.iter_mut().enumerate() {
            let bytes = [index as u8, round];
            let (reply, n) = client.call(&bytes, time::after(300).unwrap()).unwrap();
            assert_eq!(&reply[..n], &bytes);
        }
    }
    assert_eq!(
        &control.call(b"stop", time::after(300).unwrap()).unwrap().0[..4],
        b"stop"
    );
    for client in &mut clients {
        client.retire();
    }
    control.retire();
    assert_eq!(
        child.wait_until(time::after(300).unwrap()).unwrap(),
        Outcome::Returned(0)
    );
    assert_eq!(
        cathedral_user_runtime::notification::budget().unwrap()[3],
        0
    );
    cathedral_user_runtime::write(b"Cathedral shared capacity: five streams, setup-right release, bounded refusal and final handle refunds passed\n").unwrap();
}
pub(super) fn worker(env: Environment) -> u64 {
    let mut server = Server::open_clients(4).unwrap();
    static mut SLOTS: [SharedSlot; 5] = [const { SharedSlot::empty() }; 5];
    let slots = &raw mut SLOTS;
    // SAFETY: Single-entry process lends each disjoint persistent slot once.
    let [control, a, b, c, d] = unsafe { &mut *slots };
    server.enable_shared_control(control).unwrap();
    for (index, slot) in [a, b, c, d].into_iter().enumerate() {
        let pair = if index == 0 {
            (env.endpoint(0).unwrap(), env.endpoint(1).unwrap())
        } else {
            let take = || {
                let (_, endpoint) = Handle::bootstrap(0)
                    .unwrap()
                    .receive_with(&mut [0; 64])
                    .unwrap();
                Handle::bootstrap(1).unwrap().send(b"ok").unwrap();
                endpoint.unwrap()
            };
            (take(), take())
        };
        server.enable_shared(index, slot).unwrap();
        server.install(index, pair).unwrap();
    }
    loop {
        let Event::Request(request) = server.next_event().unwrap() else {
            panic!("request")
        };
        assert_eq!(
            cathedral_user_runtime::notification::budget().unwrap()[3],
            15
        );
        server
            .reply(&request, &request.bytes[..request.len])
            .unwrap();
        if request.control {
            return 0;
        }
    }
}
