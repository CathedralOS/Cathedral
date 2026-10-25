//! Exercise the real server against full single-message kernel channels.
use super::*;
use cathedral_user_runtime::server::{Event, Replies, Server};

pub fn run() -> u64 {
    let mut server = Server::open_clients(1).unwrap();
    static mut REPLIES: Replies = Replies::empty();
    let address = &raw mut REPLIES;
    // SAFETY: This worker enters once and exclusively owns the static arena.
    server.retain_replies(unsafe { &mut *address });
    let (send, input) = ipc::channel().unwrap();
    let (output, receive) = ipc::channel().unwrap();
    server.install(0, (input, output)).unwrap();
    send.send(b"a").unwrap();
    let first = server.next_request().unwrap();
    send.send(b"b").unwrap();
    let second = server.next_request().unwrap();
    server.reply(&first, b"one").unwrap();
    server.reply(&second, b"two").unwrap(); // Must retain, not silently discard.
    send.send(b"c").unwrap();
    assert!(
        server
            .next_event_or(&[], Some(time::now().unwrap()))
            .unwrap()
            .is_none()
    );
    let mut bytes = [0; 64];
    assert_eq!(receive.receive(&mut bytes).unwrap(), 3);
    assert_eq!(&bytes[..3], b"one");
    let third = server.next_request().unwrap(); // Flush two, then resume c.
    assert_eq!(third.bytes[0], b'c');
    assert_eq!(receive.receive(&mut bytes).unwrap(), 3);
    assert_eq!(&bytes[..3], b"two");
    server.reply(&third, b"three").unwrap();
    receive.receive(&mut bytes).unwrap();
    assert_eq!(&bytes[..5], b"three");

    server.reply(&first, b"old").unwrap();
    server.reply(&second, b"stale").unwrap();
    server.remove(0);
    for handle in [send, input, output, receive] {
        handle.close().unwrap();
    }
    let (send, input) = ipc::channel().unwrap();
    let (output, receive) = ipc::channel().unwrap();
    server.install(0, (input, output)).unwrap();
    server.reply(&first, b"late").unwrap(); // Cannot reply to/disconnect replacement.
    send.send(b"new").unwrap();
    let fresh = server.next_request().unwrap();
    assert_ne!(fresh.incarnation, first.incarnation);
    assert_eq!(
        receive.receive_until(&mut bytes, time::now().unwrap()),
        Err(Error(abi::TIMED_OUT as i64))
    );
    server.reply(&fresh, b"fresh").unwrap();
    receive.receive(&mut bytes).unwrap();
    assert_eq!(&bytes[..5], b"fresh");

    // An over-pipelined client is disconnected rather than growing an arena or
    // losing a success silently. The enclosing server remains usable.
    let flood = [b'x', b'y', b'z'].map(|code| {
        send.send(&[code]).unwrap();
        server.next_request().unwrap()
    });
    for request in flood {
        server.reply(&request, b"answer").unwrap();
    }
    assert!(
        matches!(server.next_event().unwrap(), Event::Disconnected { client: 0, incarnation } if incarnation == fresh.incarnation)
    );
    for handle in [send, input, output, receive] {
        handle.close().unwrap();
    }
    cathedral_user_runtime::write(b"Cathedral IPC: retained replies preserve order, bound backpressure and reject stale incarnations\n").unwrap();
    0
}
