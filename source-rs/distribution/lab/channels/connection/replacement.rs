//! Preselected empty slots survive removal without reviving old reply authority.
use super::*;
use cathedral_user_runtime::{
    memory::Private,
    server::{Event, Server, SharedSlot},
};
pub(super) fn run(launch: Launch) {
    let (send, input) = ipc::channel().unwrap();
    let (output, recv) = ipc::channel().unwrap();
    let child = launch.spawn_with(73, &[input, output]).unwrap();
    let (control, events) = Port::bootstrap().unwrap().connect().unwrap();
    let mut old = Client::connect((send, recv), time::after(300).unwrap()).unwrap();
    old.begin(b"hold", None).unwrap();
    // The worker reports that it holds the exact request before disconnection.
    assert_eq!(&receive(events).0[..4], b"held");
    old.retire();
    assert_eq!(&receive(events).0[..4], b"gone");
    let (send, input) = ipc::channel().unwrap();
    let (output, recv) = ipc::channel().unwrap();
    control.send_with(b"input", input).unwrap();
    assert_eq!(&receive(events).0[..2], b"ok");
    control.send_with(b"output", output).unwrap();
    assert_eq!(&receive(events).0[..2], b"ok");
    let mut next = Client::connect((send, recv), time::after(300).unwrap()).unwrap();
    assert_eq!(
        &next.call(b"new", time::after(300).unwrap()).unwrap().0[..5],
        b"fresh"
    );
    next.retire();
    assert_eq!(
        child.wait_until(time::after(300).unwrap()).unwrap(),
        Outcome::Returned(0)
    );
    cathedral_user_runtime::write(b"Cathedral shared replacement: prearmed empty slot, remove/reinstall and stale grant rejection passed\n").unwrap();
}
pub(super) fn worker(env: Environment) -> u64 {
    let mut server = Server::open().unwrap();
    static mut SLOT: SharedSlot = SharedSlot::empty();
    let address = &raw mut SLOT;
    // SAFETY: Single-entry process lends this slot once, across both occupants.
    server.enable_shared(0, unsafe { &mut *address }).unwrap();
    server
        .install(0, (env.endpoint(0).unwrap(), env.endpoint(1).unwrap()))
        .unwrap();
    let old = match server.next_event().unwrap() {
        Event::Request(r) => r,
        _ => panic!("request"),
    };
    assert!(!old.control && old.client == Some(0));
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
