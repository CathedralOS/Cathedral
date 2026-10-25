//! Typed notification replies preserve wait-only authority and independent hints.
//! The provider keeps its source right; rejected/unclaimed copies refund custody.
use super::*;
use cathedral_user_runtime::server::{Event, Server, SharedSlot};
pub(super) fn run(launch: Launch) {
    typed_receive();
    resources::assign(
        launch,
        Limits {
            private_pages: 2,
            reader_pages: 2,
            channels: 8,
        },
    )
    .unwrap();
    for mode in [89, 90, 91, 92, 93, 99] {
        let baseline = resources::budget().unwrap();
        let physical = resources::physical_budget().unwrap();
        let notifications = cathedral_user_runtime::notification::budget().unwrap();
        let (send, input) = ipc::channel().unwrap();
        let (output, recv) = ipc::channel().unwrap();
        let child = launch.spawn_with(mode, &[input, output]).unwrap();
        let _control = Port::bootstrap().unwrap().connect().unwrap();
        let mut client = Client::connect((send, recv), time::after(300).unwrap()).unwrap();
        if mode == 99 {
            // Fill only notification metadata: service object funding remains spare.
            let (grant_send, grant_receive) = ipc::channel().unwrap();
            let rights = N::create().unwrap();
            let mut held = [None; abi::NOTIFY_HANDLES];
            let budget = cathedral_user_runtime::notification::budget().unwrap();
            let count = (budget[2] - budget[3]) as usize;
            for slot in held.iter_mut().take(count) {
                rights[0].delegate(grant_send, b"fill").unwrap();
                *slot = N::try_accept(grant_receive, &mut [0; 64]).unwrap().1;
            }
            let (bytes, n, grant) = client.call_retained_notification(b"capacity").unwrap();
            assert_eq!(&bytes[..n], b"capacity");
            assert!(grant.is_none());
            for right in held.into_iter().flatten().chain(rights) {
                right.release().unwrap();
            }
            grant_send.close().unwrap();
            grant_receive.close().unwrap();
            let (_, _, grant) = client.call_retained_notification(b"wait").unwrap();
            drop(grant);
            client.retire();
        } else if mode == 89 {
            let (_, _, grant) = client.call_retained_notification(b"wait").unwrap();
            let wait = grant.unwrap().into_handle();
            assert_eq!(wait.signal(false), Err(Error(abi::DENIED as i64)));
            assert_eq!(wait.close_object(), Err(Error(abi::DENIED as i64)));
            // Reply processing must not consume a distinct pending input hint.
            for _ in 0..16 {
                assert_eq!(client.call_retained(b"signal").unwrap().1, 6);
                assert_eq!(wait.wait(Some(time::now().unwrap())), Ok(Wake::Signaled));
                assert_eq!(wait.wait(Some(time::now().unwrap())), Ok(Wake::TimedOut));
                let before = cathedral_user_runtime::notification::budget().unwrap();
                let (_, _, unused) = client.call_retained_notification(b"wait").unwrap();
                drop(unused);
                assert_eq!(
                    cathedral_user_runtime::notification::budget().unwrap(),
                    before
                );
            }
            let (_, n, grant) = client.call_retained_notification(b"plain").unwrap();
            assert_eq!(n, 5);
            assert!(grant.is_none());
            wait.release().unwrap();
            client.retire();
        } else {
            let error = match mode {
                90 => client.call_retained(b"wait").err(),
                91 => client.call_retained_endpoint(b"wait").err(),
                _ => client.call_retained_notification(b"wrong kind").err(),
            };
            assert_eq!(error, Some(Error(abi::IO_ERROR as i64)));
            assert_eq!(
                client.call_retained(b"no replay"),
                Err(Error(abi::PEER_CLOSED as i64))
            );
        }
        assert_eq!(
            child.wait_until(time::after(300).unwrap()).unwrap(),
            Outcome::Returned(0)
        );
        drop(client);
        assert_eq!(resources::budget().unwrap(), baseline);
        assert_eq!(resources::physical_budget().unwrap(), physical);
        assert_eq!(
            cathedral_user_runtime::notification::budget().unwrap(),
            notifications
        );
    }
    super::notification_faults::run(launch);
    cathedral_user_runtime::write(b"Cathedral shared notification grants: wait authority, independent hints, correlation, rejection and refunds passed\n").unwrap();
}
fn typed_receive() {
    let baseline = cathedral_user_runtime::notification::budget().unwrap();
    let (send, recv) = ipc::channel().unwrap();
    let [wait, signal, control] = N::create().unwrap();
    assert_eq!(
        N::try_accept(recv, &mut [0; 64]),
        Err(Error(abi::WOULD_BLOCK as i64))
    );
    assert_eq!(
        N::try_accept(send, &mut [0; 64]),
        Err(Error(abi::DENIED as i64))
    );
    wait.delegate(send, b"typed").unwrap();
    assert_eq!(
        crate::syscall::call(abi::NOTIFY_TRY_RECEIVE, recv.raw(), 0, 0),
        abi::BAD_ADDRESS
    );
    assert_eq!(
        crate::syscall::call(abi::NOTIFY_TRY_RECEIVE, recv.raw(), 0, 1),
        abi::INVALID_ARGUMENT
    );
    assert!(
        matches!(recv.try_receive_with(&mut [0; 64]), Err(Error(c)) if c == abi::TOO_SMALL as i64)
    );
    assert!(
        matches!(recv.try_receive_memory(&mut [0; 64]), Err(Error(c)) if c == abi::TOO_SMALL as i64)
    );
    let (n, received) = N::try_accept(recv, &mut [0; 64]).unwrap();
    assert_eq!(n, 5);
    let received = received.unwrap();
    assert_eq!(received.signal(false), Err(Error(abi::DENIED as i64)));
    received.release().unwrap();
    // A different queued grant must remain available to its own typed receiver.
    let (source, sink) = ipc::channel().unwrap();
    send.send_with(b"endpoint", source).unwrap();
    assert_eq!(
        N::try_accept(recv, &mut [0; 64]),
        Err(Error(abi::TOO_SMALL as i64))
    );
    recv.try_receive_with(&mut [0; 64])
        .unwrap()
        .1
        .unwrap()
        .close()
        .unwrap();
    sink.close().unwrap();
    control.close_object().unwrap();
    for right in [wait, signal, control] {
        right.release().unwrap();
    }
    send.close().unwrap();
    recv.close().unwrap();
    assert_eq!(
        cathedral_user_runtime::notification::budget().unwrap(),
        baseline
    );
}
pub(super) fn worker(mode: u64, env: Environment) -> u64 {
    let pair = (env.endpoint(0).unwrap(), env.endpoint(1).unwrap());
    let mut server = Server::open_clients(1).unwrap();
    server.install(0, pair).unwrap();
    static mut SLOT: SharedSlot = SharedSlot::empty();
    let slot = &raw mut SLOT;
    // SAFETY: The process's single entry owns this connection slot.
    server.enable_shared(0, unsafe { &mut *slot }).unwrap();
    let [wait, signal, control] = N::create().unwrap();
    loop {
        match server.next_event().unwrap() {
            Event::Request(r) => {
                match &r.bytes[..r.len] {
                    b"plain" => server.reply(&r, b"plain").unwrap(),
                    b"signal" => {
                        signal.signal(false).unwrap();
                        signal.signal(false).unwrap(); // Coalesced hint, not an event count.
                        server.reply(&r, b"signal").unwrap();
                    }
                    _ if mode == 92 => {
                        let image = cathedral_user_runtime::memory::Private::allocate(1)
                            .unwrap()
                            .freeze()
                            .unwrap();
                        server.reply_memory(&r, b"memory", &image).unwrap();
                    }
                    _ if mode == 93 => {
                        let (send, recv) = ipc::channel().unwrap();
                        server.reply_endpoint(&r, b"endpoint", send).unwrap();
                        recv.close().unwrap();
                    }
                    b"capacity" => {
                        assert_eq!(
                            server.reply_notification(&r, b"wait", wait),
                            Err(Error(abi::NO_MEMORY as i64))
                        );
                        assert_eq!(wait.wait(Some(time::now().unwrap())), Ok(Wake::TimedOut));
                        server.reply(&r, b"capacity").unwrap();
                    }
                    _ => {
                        server.reply_notification(&r, b"wait", wait).unwrap();
                        assert_eq!(
                            server.reply_notification(&r, b"duplicate", wait),
                            Err(Error(abi::PEER_CLOSED as i64))
                        );
                        // Delegation retains the provider's source right.
                        assert_eq!(wait.wait(Some(time::now().unwrap())), Ok(Wake::TimedOut));
                        if mode == 100 {
                            assert_eq!(
                                Handle::bootstrap(0).unwrap().receive(&mut [0; 64]).unwrap(),
                                4
                            );
                            break;
                        }
                    }
                }
            }
            Event::Disconnected { .. } => break,
            _ => panic!("unexpected notification request"),
        }
    }
    control.close_object().unwrap();
    for right in [wait, signal, control] {
        right.release().unwrap();
    }
    0
}
