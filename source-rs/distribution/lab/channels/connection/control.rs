//! Owner-control upgrade keeps its authority and retires before replacement.
use super::*;
use cathedral_user_runtime::{
    memory::Private,
    server::{Event, Server},
};
pub(super) fn run(launch: Launch) {
    for mode in 68..=70 {
        let (send, input) = ipc::channel().unwrap();
        let (output, recv) = ipc::channel().unwrap();
        let child = launch.spawn_with(mode, &[input, output]).unwrap();
        if mode == 70 {
            send.send(wire::OPEN).unwrap();
            let mut bytes = [0; 64];
            let n = recv.receive(&mut bytes).unwrap();
            assert_eq!(&bytes[..n], wire::ACK);
            assert_eq!(
                child.wait_until(time::after(600).unwrap()).unwrap(),
                Outcome::Returned(0)
            );
            send.close().unwrap();
            recv.close().unwrap();
            continue;
        }
        let mut client = Client::connect((send, recv), time::after(300).unwrap()).unwrap();
        if mode == 68 {
            let mut page = Private::allocate(1).unwrap();
            page.bytes_mut().fill(68);
            let object = page.freeze().unwrap();
            client.begin(b"control", Some(&object)).unwrap();
            let (bytes, n, object) = loop {
                if let Some(reply) = client.poll().unwrap() {
                    break reply;
                }
                event::wait(&client.interests().unwrap(), time::after(300).unwrap()).unwrap();
            };
            assert_eq!(&bytes[..n], b"control");
            assert!(object.unwrap().map().unwrap().iter().all(|b| *b == 68));
            client.retire();
        } else {
            assert_eq!(
                client.call(b"hold", time::after(3).unwrap()),
                Err(Error(abi::TIMED_OUT as i64))
            );
        }
        assert_eq!(
            child.wait_until(time::after(300).unwrap()).unwrap(),
            Outcome::Returned(0)
        );
    }
    cathedral_user_runtime::write(b"Cathedral shared control: yielding calls, memory custody, replacement and setup expiry passed\n").unwrap();
}
pub(super) fn worker(mode: u64, pair: (Handle, Handle)) -> u64 {
    let mut server = Server::open().unwrap();
    server.set_control(pair);
    server.enable_shared_control().unwrap();
    server.enable_memory();
    let mut held = None;
    loop {
        match server.next_event() {
            Ok(Event::Request(request)) => {
                assert!(request.control && request.client.is_none());
                assert_eq!(mode, 69);
                held = Some(request);
            }
            Ok(Event::Memory(request, object)) => {
                assert!(request.control && request.client.is_none());
                assert_eq!(mode, 68);
                server
                    .reply_memory(&request, &request.bytes[..request.len], &object)
                    .unwrap();
            }
            Err(Error(c)) if c == abi::PEER_CLOSED as i64 => break,
            _ => panic!("unexpected control event"),
        }
    }
    if let Some(old) = held {
        let (send, input) = ipc::channel().unwrap();
        let (output, recv) = ipc::channel().unwrap();
        server.set_control((input, output));
        server.enable_shared_control().unwrap();
        let object = Private::allocate(1).unwrap().freeze().unwrap();
        server.reply_memory(&old, b"late", &object).unwrap();
        assert_eq!(
            recv.receive_until(&mut [0; 64], time::now().unwrap()),
            Err(Error(abi::TIMED_OUT as i64))
        );
        for h in [send, input, output, recv] {
            h.close().unwrap();
        }
    }
    0
}
