//! Preselected empty slots survive removal without reviving old reply authority.
use super::*;
use cathedral_user_runtime::{
    connection::Pipeline,
    memory::Private,
    server::{Event, Server, SharedSlot},
};
pub(super) fn run(launch: Launch) {
    case::<1>(launch, 73);
    case::<4>(launch, 78);
}
fn case<const WIDTH: usize>(launch: Launch, mode: u64) {
    let (send, input) = ipc::channel().unwrap();
    let (output, recv) = ipc::channel().unwrap();
    let child = launch.spawn_with(mode, &[input, output]).unwrap();
    let (control, events) = Port::bootstrap().unwrap().connect().unwrap();
    let mut old = Pipeline::<WIDTH>::connect((send, recv), time::after(300).unwrap()).unwrap();
    old.begin(b"hold", None).unwrap();
    if WIDTH > 1 {
        for bytes in [b"queued-a", b"queued-b"] {
            let until = time::after(300).unwrap();
            loop {
                match old.submit(bytes, None) {
                    Ok(_) => break,
                    Err(Error(c)) if c == abi::WOULD_BLOCK as i64 => {
                        assert!(!time::reached(time::now().unwrap(), until));
                        event::wait(&old.interests().unwrap(), until).unwrap();
                    }
                    Err(e) => panic!("submit before replacement: {e:?}"),
                }
            }
        }
    }
    // The worker holds the original request and, in pipeline mode, published
    // one response and queued another. Drop must retire all three independently.
    assert_eq!(&receive(events).0[..4], b"held");
    old.retire();
    assert_eq!(&receive(events).0[..4], b"gone");
    let (send, input) = ipc::channel().unwrap();
    let (output, recv) = ipc::channel().unwrap();
    control.send_with(b"input", input).unwrap();
    assert_eq!(&receive(events).0[..2], b"ok");
    control.send_with(b"output", output).unwrap();
    assert_eq!(&receive(events).0[..2], b"ok");
    let mut next = Pipeline::<WIDTH>::connect((send, recv), time::after(300).unwrap()).unwrap();
    assert_eq!(
        &next.call(b"new", time::after(300).unwrap()).unwrap().0[..5],
        b"fresh"
    );
    next.retire();
    assert_eq!(
        child.wait_until(time::after(300).unwrap()).unwrap(),
        Outcome::Returned(0)
    );
    cathedral_user_runtime::write(b"Cathedral shared replacement: prearmed empty slot, queued-reply removal/reinstall and stale grant rejection passed\n").unwrap();
}
pub(super) fn worker(mode: u64, env: Environment) -> u64 {
    let mut server = Server::open().unwrap();
    static mut SLOT: SharedSlot = SharedSlot::empty();
    let address = &raw mut SLOT;
    // SAFETY: Single-entry process lends this slot once, across both occupants.
    let slot = unsafe { &mut *address };
    if mode == 78 {
        server.enable_pipeline(0, slot).unwrap();
    } else {
        server.enable_shared(0, slot).unwrap();
    }
    server
        .install(0, (env.endpoint(0).unwrap(), env.endpoint(1).unwrap()))
        .unwrap();
    let old = match server.next_event().unwrap() {
        Event::Request(r) => r,
        _ => panic!("request"),
    };
    assert!(!old.control && old.client == Some(0));
    if mode == 78 {
        for _ in 0..2 {
            let Event::Request(request) = server.next_event().unwrap() else {
                panic!("queued request");
            };
            server
                .reply(&request, &request.bytes[..request.len])
                .unwrap();
        }
    }
    Handle::bootstrap(1).unwrap().send(b"held").unwrap();
    assert!(matches!(
        server.next_event().unwrap(),
        Event::Disconnected { client: 0, .. }
    ));
    server.remove(0);
    Handle::bootstrap(1).unwrap().send(b"gone").unwrap();
    let mut bytes = [0; 64];
    let (_, input) = Handle::bootstrap(0)
        .unwrap()
        .receive_with(&mut bytes)
        .unwrap();
    Handle::bootstrap(1).unwrap().send(b"ok").unwrap();
    let (_, output) = Handle::bootstrap(0)
        .unwrap()
        .receive_with(&mut bytes)
        .unwrap();
    server
        .install(0, (input.unwrap(), output.unwrap()))
        .unwrap();
    Handle::bootstrap(1).unwrap().send(b"ok").unwrap();
    let next = match server.next_event().unwrap() {
        Event::Request(r) => r,
        _ => panic!("new request"),
    };
    assert_ne!(next.incarnation, old.incarnation);
    let object = Private::allocate(1).unwrap().freeze().unwrap();
    server.reply_memory(&old, b"stale", &object).unwrap();
    server.reply(&next, b"fresh").unwrap();
    0
}
