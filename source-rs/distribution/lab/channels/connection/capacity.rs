//! Twenty live streams retain close authority and still obey the metadata ceiling.
use super::*;
use cathedral_user_runtime::server::{Event, Server, SharedSlot};
const STREAMS: usize = 20;
const CLIENTS: usize = STREAMS - 1;
const RIGHTS: usize = STREAMS * 3;
pub(super) fn run(launch: Launch) {
    resources::assign(
        launch,
        Limits {
            private_pages: STREAMS as u64,
            reader_pages: STREAMS as u64,
            channels: 24,
        },
    )
    .unwrap();
    let make_pair = || {
        let (send, input) = ipc::channel().unwrap();
        let (output, receive) = ipc::channel().unwrap();
        ((send, receive), (input, output))
    };
    let (first, peer) = make_pair();
    let child = launch.spawn_with(74, &[peer.0, peer.1]).unwrap();
    let control_pair = Port::bootstrap().unwrap().connect().unwrap();
    let mut pairs = [first; CLIENTS];
    // Move each new peer pair immediately. Keeping all forty peer endpoints
    // locally before transfer would test the unrelated endpoint metadata cap.
    for pair in &mut pairs[1..] {
        let (local, peer) = make_pair();
        for endpoint in [peer.0, peer.1] {
            control_pair.0.send_with(b"endpoint", endpoint).unwrap();
            assert_eq!(&receive(control_pair.1).0[..2], b"ok");
        }
        *pair = local;
    }
    static mut CLIENT_STORAGE: [Option<Client>; CLIENTS] = [const { None }; CLIENTS];
    let clients = &raw mut CLIENT_STORAGE;
    // SAFETY: One fixture entry owns this arena; large peer states stay off-stack.
    let clients = unsafe { &mut *clients };
    for (slot, pair) in clients.iter_mut().zip(pairs) {
        *slot = Some(Client::connect(pair, time::after(300).unwrap()).unwrap());
    }
    let mut control = Client::connect(control_pair, time::after(300).unwrap()).unwrap();
    assert_eq!(
        cathedral_user_runtime::notification::budget().unwrap()[3],
        RIGHTS as u64
    );
    // Exhaust grant metadata while the account still has object funding. Refusal
    // must not corrupt the twenty service streams or leave partial create grants.
    let mut extra = [None; abi::NOTIFY_HANDLES / 3];
    let count = (abi::NOTIFY_HANDLES - RIGHTS) / 3;
    for slot in &mut extra[..count] {
        *slot = Some(N::create().unwrap());
    }
    let budget = cathedral_user_runtime::notification::budget().unwrap();
    assert!(budget[0] > budget[1]);
    assert_eq!(budget[3], (RIGHTS + count * 3) as u64);
    assert_eq!(N::create(), Err(Error(abi::NO_MEMORY as i64)));
    assert_eq!(
        cathedral_user_runtime::notification::budget().unwrap(),
        budget
    );
    // Leave two free metadata slots: create needs three and must publish none.
    // This remains a partial-capacity check when the ceiling is divisible by three.
    let handles = extra[count - 1].take().unwrap();
    handles[0].release().unwrap();
    handles[1].release().unwrap();
    let partial = cathedral_user_runtime::notification::budget().unwrap();
    assert_eq!(partial[2] - partial[3], 2);
    assert_eq!(N::create(), Err(Error(abi::NO_MEMORY as i64)));
    assert_eq!(
        cathedral_user_runtime::notification::budget().unwrap(),
        partial
    );
    handles[2].close_object().unwrap();
    handles[2].release().unwrap();
    for handles in extra.into_iter().flatten() {
        handles[2].close_object().unwrap();
        for handle in handles {
            handle.release().unwrap();
        }
    }
    for round in 0..3u8 {
        for (index, client) in clients.iter_mut().enumerate() {
            let client = client.as_mut().unwrap();
            let bytes = [index as u8, round];
            let (reply, n) = client.call(&bytes, time::after(300).unwrap()).unwrap();
            assert_eq!(&reply[..n], &bytes);
        }
    }
    // Exercise readiness in the final slot, beyond the former 32-interest ABI.
    // The unused receiver cannot win, so only the peer's reply notification does.
    let (unused, quiet) = ipc::channel().unwrap();
    let last = clients.last_mut().unwrap().as_mut().unwrap();
    last.begin(b"wide wait", None).unwrap();
    let mut interests = [Interest::Ipc(quiet); abi::MAX_WAIT_HANDLES];
    interests[abi::MAX_WAIT_HANDLES - 1] = last.interests().unwrap()[1];
    loop {
        assert_eq!(
            event::wait(&interests, time::after(300).unwrap()).unwrap(),
            abi::MAX_WAIT_HANDLES - 1
        );
        if let Some((bytes, n, object)) = last.poll().unwrap() {
            assert_eq!(&bytes[..n], b"wide wait");
            assert!(object.is_none());
            break;
        }
    }
    // Both wait entrypoints validate the whole enlarged set before readiness.
    let (send, receive) = ipc::channel().unwrap();
    send.send(b"last slot").unwrap();
    let mut handles = [quiet; abi::MAX_WAIT_HANDLES];
    handles[abi::MAX_WAIT_HANDLES - 1] = receive;
    assert_eq!(
        ipc::wait_many(&handles, time::after(300).unwrap()).unwrap(),
        abi::MAX_WAIT_HANDLES - 1
    );
    handles[0] = receive;
    handles[abi::MAX_WAIT_HANDLES - 1] = Handle::from_raw(0);
    assert_eq!(
        ipc::wait_many(&handles, time::now().unwrap()),
        Err(Error(abi::BAD_HANDLE as i64))
    );
    let mut bytes = [0; 64];
    assert_eq!(receive.receive(&mut bytes).unwrap(), 9);
    receive.close().unwrap();
    send.close().unwrap();
    let [wait, signal, close] = N::create().unwrap();
    signal.signal(false).unwrap();
    interests[0] = Interest::Notification(wait);
    interests[abi::MAX_WAIT_HANDLES - 1] = Interest::Ipc(Handle::from_raw(0));
    assert_eq!(
        event::wait(&interests, time::now().unwrap()),
        Err(Error(abi::BAD_HANDLE as i64))
    );
    assert_eq!(
        wait.wait(Some(time::now().unwrap())).unwrap(),
        Wake::Signaled
    );
    close.close_object().unwrap();
    for right in [wait, signal, close] {
        right.release().unwrap();
    }
    quiet.close().unwrap();
    unused.close().unwrap();
    assert_eq!(
        &control.call(b"stop", time::after(300).unwrap()).unwrap().0[..4],
        b"stop"
    );
    for client in clients {
        drop(client.take());
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
    cathedral_user_runtime::write(b"Cathedral shared capacity: twenty streams, final wait slot, bounded refusal and final handle refunds passed\n").unwrap();
}
pub(super) fn worker(env: Environment) -> u64 {
    let mut server = Server::open_clients(CLIENTS).unwrap();
    static mut SLOTS: [SharedSlot; STREAMS] = [const { SharedSlot::empty() }; STREAMS];
    let slots = &raw mut SLOTS;
    // SAFETY: Single-entry process lends each disjoint persistent slot once.
    let (control, slots) = unsafe { &mut *slots }.split_first_mut().unwrap();
    server.enable_shared_control(control).unwrap();
    for (index, slot) in slots.iter_mut().enumerate() {
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
            RIGHTS as u64
        );
        server
            .reply(&request, &request.bytes[..request.len])
            .unwrap();
        if request.control {
            return 0;
        }
    }
}
