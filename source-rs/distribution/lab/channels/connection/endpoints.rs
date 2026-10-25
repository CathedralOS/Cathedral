//! Shared replies move endpoint custody once; rejected or unclaimed rights die.
use super::*;
use cathedral_user_runtime::server::{Event, Server, SharedSlot};
pub(super) fn run(launch: Launch) {
    resources::assign(
        launch,
        Limits {
            private_pages: 2,
            reader_pages: 2,
            channels: 8,
        },
    )
    .unwrap();
    // The nonblocking form validates authority and preserves incompatible grants.
    let (send, recv) = ipc::channel().unwrap();
    let (source, sink) = ipc::channel().unwrap();
    assert!(
        matches!(recv.try_receive_with(&mut [0; 64]), Err(Error(c)) if c == abi::WOULD_BLOCK as i64)
    );
    assert!(
        matches!(send.try_receive_with(&mut [0; 64]), Err(Error(c)) if c == abi::DENIED as i64)
    );
    send.send_with(b"typed", source).unwrap();
    // Validate full output storage and reserved fields before consuming escrow.
    assert_eq!(
        crate::syscall::call(abi::IPC_TRY_RECEIVE_CAP, recv.raw(), 0, 0),
        abi::BAD_ADDRESS
    );
    assert_eq!(
        crate::syscall::call(abi::IPC_TRY_RECEIVE_CAP, recv.raw(), 0, 1),
        abi::INVALID_ARGUMENT
    );

    assert!(
        matches!(recv.try_receive_memory(&mut [0; 64]), Err(Error(c)) if c == abi::TOO_SMALL as i64)
    );
    let (n, moved) = recv.try_receive_with(&mut [0; 64]).unwrap();
    assert_eq!(n, 5);
    moved.unwrap().close().unwrap();
    assert_eq!(
        sink.receive(&mut [0; 64]),
        Err(Error(abi::PEER_CLOSED as i64))
    );
    for handle in [send, recv, sink] {
        handle.close().unwrap();
    }
    for mode in 79..=86 {
        let baseline = resources::budget().unwrap();
        let physical = resources::physical_budget().unwrap();
        let (send, input) = ipc::channel().unwrap();
        let (output, recv) = ipc::channel().unwrap();
        let child = launch.spawn_with(mode, &[input, output]).unwrap();
        let _control = Port::bootstrap().unwrap().connect().unwrap();
        let mut client = Client::connect((send, recv), time::after(300).unwrap()).unwrap();
        if mode == 79 {
            for _ in 0..16 {
                let (bytes, n, endpoint) = client.call_retained_endpoint(b"use").unwrap();
                assert_eq!(&bytes[..n], b"endpoint");
                let endpoint = endpoint.unwrap().into_handle();
                endpoint.send(b"proof").unwrap();
                endpoint.close().unwrap();
                // The receiver accepts custody but deliberately never claims it.
                let (_, _, unused) = client.call_retained_endpoint(b"drop").unwrap();
                drop(unused);
            }
            let (_, n, endpoint) = client.call_retained_endpoint(b"plain").unwrap();
            assert_eq!(n, 5);
            assert!(endpoint.is_none());
            client.retire();
        } else if mode == 84 {
            assert!(
                matches!(client.call_retained(b"reject"), Err(Error(c)) if c == abi::IO_ERROR as i64)
            );
        } else if mode == 85 {
            let (_, _, endpoint) = client.call_retained_endpoint(b"frame first").unwrap();
            let endpoint = endpoint.unwrap().into_handle();
            endpoint.send(b"proof").unwrap();
            endpoint.close().unwrap();
            client.retire();
        } else {
            let expected = match mode {
                81 | 82 => abi::TIMED_OUT,
                86 => abi::PEER_CLOSED,
                _ => abi::IO_ERROR,
            };
            assert!(
                matches!(client.call_retained_endpoint(b"malformed"), Err(Error(c)) if c == expected as i64)
            );
        }
        assert_eq!(
            child.wait_until(time::after(300).unwrap()).unwrap(),
            Outcome::Returned(0)
        );
        drop(client);
        assert_eq!(resources::budget().unwrap(), baseline);
        assert_eq!(resources::physical_budget().unwrap(), physical);
    }
    super::endpoint_faults::run(launch);
    cathedral_user_runtime::write(b"Cathedral shared endpoints: moved custody, drop, stale replies, wrong kind, orphan expiry and refunds passed\n").unwrap();
}
pub(super) fn worker(mode: u64, env: Environment) -> u64 {
    let pair = (env.endpoint(0).unwrap(), env.endpoint(1).unwrap());
    let mut server = Server::open_clients(1).unwrap();
    server.install(0, pair).unwrap();
    static mut SLOT: SharedSlot = SharedSlot::empty();
    let slot = &raw mut SLOT;
    // SAFETY: This entry owns its persistent connection slot once.
    let slot = unsafe { &mut *slot };
    if mode == 88 {
        server.enable_pipeline(0, slot).unwrap();
    } else {
        server.enable_shared(0, slot).unwrap();
    }
    loop {
        match server.next_event().unwrap() {
            Event::Request(r) => {
                if r.control {
                    assert_eq!(&r.bytes[..r.len], b"done");
                    return 0;
                }
                let payload = &r.bytes[..r.len];
                if payload == b"plain" {
                    server.reply(&r, b"plain").unwrap();
                    if mode == 88 {
                        Handle::bootstrap(1).unwrap().send(b"first").unwrap();
                    }
                    continue;
                }
                let (send, recv) = ipc::channel().unwrap();
                if mode == 88 {
                    assert_eq!(
                        server.reply_endpoint(&r, b"endpoint", send),
                        Err(Error(abi::WOULD_BLOCK as i64))
                    );
                    send.send(b"still owned").unwrap();
                    assert_eq!(recv.receive(&mut [0; 64]).unwrap(), 11);
                    send.close().unwrap();
                    recv.close().unwrap();
                    server.reply(&r, b"capacity").unwrap();
                    Handle::bootstrap(1).unwrap().send(b"refused").unwrap();
                    continue;
                }
                server.reply_endpoint(&r, b"endpoint", send).unwrap();
                assert_eq!(send.send(b"stale"), Err(Error(abi::BAD_HANDLE as i64)));
                // The old request cannot consume a newly supplied endpoint.
                let (new_send, new_recv) = ipc::channel().unwrap();
                assert_eq!(
                    server.reply_endpoint(&r, b"duplicate", new_send),
                    Err(Error(abi::PEER_CLOSED as i64))
                );
                new_send.send(b"owned").unwrap();
                assert_eq!(new_recv.receive(&mut [0; 64]).unwrap(), 5);
                new_send.close().unwrap();
                new_recv.close().unwrap();
                if payload == b"drop" {
                    assert_eq!(
                        recv.receive(&mut [0; 64]),
                        Err(Error(abi::PEER_CLOSED as i64))
                    );
                } else {
                    let mut bytes = [0; 64];
                    let n = recv.receive(&mut bytes).unwrap();
                    assert_eq!(&bytes[..n], b"proof");
                }
                recv.close().unwrap();
            }
            Event::Disconnected { .. } => return 0,
            _ => panic!("unexpected endpoint fixture input"),
        }
    }
}
