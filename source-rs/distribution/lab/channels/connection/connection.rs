// Ordinary Server/Client transport, including abandoned setup and stale replies.
use super::*;
use cathedral_platform_contracts::connection as wire;
use cathedral_user_runtime::{
    connection::Client,
    event::{self, Interest},
    notification::{Notification as N, Outcome as Wake},
    resources::{self, Limits},
    task::{Environment, Launch, Outcome, Port},
};
pub(super) fn run() {
    mixed();
    let baseline = resources::budget().unwrap();
    let physical = resources::physical_budget().unwrap();
    let launch = Launch::bootstrap().unwrap();
    resources::assign(
        launch,
        Limits {
            private_pages: 4,
            reader_pages: 4,
            channels: 8,
        },
    )
    .unwrap();
    for mode in [50, 51, 52, 53, 54, 55] {
        let (send, input) = ipc::channel().unwrap();
        let (output, recv) = ipc::channel().unwrap();
        let child = launch.spawn_with(mode, &[input, output]).unwrap();
        let (control, events) = Port::bootstrap().unwrap().connect().unwrap();
        if mode == 52 || mode == 53 {
            send.send(wire::OPEN).unwrap();
            let mut bytes = [0; 64];
            assert_eq!(recv.receive(&mut bytes).unwrap(), wire::ACK.len());
            control.send(b"alive").unwrap();
            assert_eq!(&receive(events).0[..5], b"alive"); // Other requests work during setup.
            if mode == 52 {
                send.close().unwrap();
                recv.close().unwrap();
            }
            // Mode 53 keeps endpoints alive. Observe beyond the 300-tick setup
            // deadline so the test's own deadline cannot race service retirement.
            let mut gone = [0; 64];
            let n = events
                .receive_until(&mut gone, time::after(600).unwrap())
                .unwrap();
            assert_eq!(&gone[..n], b"gone");
            if mode == 53 {
                send.close().unwrap();
                recv.close().unwrap();
            }
        } else {
            let mut client = Client::connect((send, recv), time::after(300).unwrap()).unwrap();
            if mode == 50 {
                for i in 0..64 {
                    let bytes = [i; 64];
                    let (got, n) = if i % 3 == 0 {
                        client.begin(&bytes, None).unwrap();
                        assert!(client.pending());
                        assert_eq!(client.begin(b"second", None), Err(Error(abi::BUSY as i64)));
                        let (got, n, object) = loop {
                            if let Some(reply) = client.poll().unwrap() {
                                break reply;
                            }
                            event::wait(&client.interests().unwrap(), time::after(300).unwrap())
                                .unwrap();
                        };
                        assert!(!client.pending());
                        assert!(object.is_none());
                        (got, n)
                    } else if i % 2 == 0 {
                        client.call_retained(&bytes).unwrap()
                    } else {
                        client.call(&bytes, time::after(300).unwrap()).unwrap()
                    };
                    assert_eq!((got, n), (bytes, 64));
                }
                client.retire();
                assert_eq!(&receive(events).0[..4], b"gone");
            } else {
                let error = client
                    .call(
                        b"request",
                        time::after(if mode == 51 { 3 } else { 300 }).unwrap(),
                    )
                    .unwrap_err();
                let expected = match mode {
                    51 => abi::TIMED_OUT,
                    54 => abi::PEER_CLOSED,
                    _ => abi::IO_ERROR,
                };
                assert_eq!(error, Error(expected as i64));
                assert_eq!(
                    client.call(b"do not replay", time::after(300).unwrap()),
                    Err(Error(abi::PEER_CLOSED as i64))
                );
                if mode == 51 {
                    assert_eq!(&receive(events).0[..4], b"gone");
                }
            }
        }
        assert_eq!(
            child.wait_until(time::after(300).unwrap()).unwrap(),
            Outcome::Returned(0)
        );
    }
    attachments::run(launch);
    control::run(launch);
    multiplex::run(launch);
    replacement::run(launch);
    pipeline::run(launch);
    capacity::run(launch);
    endpoints::run(launch);
    notification_grants::run(launch);
    resources::assign(
        launch,
        Limits {
            private_pages: 0,
            reader_pages: 0,
            channels: 8,
        },
    )
    .unwrap();
    assert_eq!(resources::budget().unwrap(), baseline);
    assert_eq!(resources::physical_budget().unwrap(), physical);
    cathedral_user_runtime::write(b"Cathedral shared service: request correlation, mixed waits, timeout retirement, abandoned setup and stale replies passed\n").unwrap();
}
fn mixed() {
    let [wait, signal, control] = N::create().unwrap();
    signal.signal(false).unwrap();
    // Exercise the full ABI buffer; a live set can grow beyond the old 128-byte
    // alignment. Every entry must be checked even when the first is ready.
    let mut full = [Interest::Notification(wait); abi::MAX_WAIT_HANDLES];
    assert_eq!(event::wait(&full, time::now().unwrap()), Ok(0));
    full[abi::MAX_WAIT_HANDLES - 1] = Interest::Notification(signal);
    assert_eq!(
        event::wait(&full, time::now().unwrap()),
        Err(Error(abi::DENIED as i64))
    );
    let (send, receive) = ipc::channel().unwrap();
    send.send(b"full wait").unwrap();
    assert_eq!(
        ipc::wait_many(&[receive; abi::MAX_WAIT_HANDLES], time::now().unwrap()),
        Ok(0)
    );
    send.close().unwrap();
    receive.close().unwrap();
    assert_eq!(
        event::wait(
            &[Interest::Notification(wait), Interest::Notification(signal)],
            time::now().unwrap()
        ),
        Err(Error(abi::DENIED as i64))
    );
    assert_eq!(
        event::wait(&[Interest::Notification(wait)], time::now().unwrap()),
        Ok(0)
    );
    assert_eq!(
        event::wait(&[Interest::Notification(wait)], time::now().unwrap()),
        Ok(0)
    );
    assert_eq!(wait.wait(None), Ok(Wake::Signaled));
    assert_eq!(
        event::wait(&[Interest::Notification(wait)], time::now().unwrap()),
        Err(Error(abi::TIMED_OUT as i64))
    );
    for h in [wait, signal, control] {
        h.release().unwrap();
    }
}
