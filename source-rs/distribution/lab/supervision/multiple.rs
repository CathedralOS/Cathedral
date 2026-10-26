//! Independent launch grants, sibling IPC continuity and joint teardown probes.
use cathedral_kernel_contracts::user as abi;
use cathedral_user_runtime::{
    Error,
    connection::Client,
    event,
    ipc::Handle,
    keyboard,
    server::{Server, SharedSlot},
    task::{Launch, Outcome, Port},
    time, yield_now,
};

fn denied() {
    assert_eq!(keyboard::read(false), Err(Error(abi::DENIED as i64)));
    assert_eq!(keyboard::write(false, 0xf4), Err(Error(abi::DENIED as i64)));
}
fn exchange(client: &mut Client, value: u8) {
    let (bytes, len) = client.call_retained(&[value]).unwrap();
    assert_eq!(&bytes[..len], &[value]);
}
pub fn echo() -> u64 {
    denied();
    assert_eq!(Launch::at(0).unwrap_err(), Error(abi::DENIED as i64));
    static mut CONTROL: SharedSlot = SharedSlot::empty();
    let slot = &raw mut CONTROL;
    let mut owner = Server::open_clients(0).unwrap();
    // SAFETY: One server in this process, exclusively owning this static slot.
    owner.enable_shared_control(unsafe { &mut *slot }).unwrap();
    loop {
        let request = owner.next_request().unwrap();
        owner
            .reply(&request, &request.bytes[..request.len])
            .unwrap();
    }
}
pub fn restart() -> u64 {
    denied();
    let launches = [Launch::at(0).unwrap(), Launch::at(1).unwrap()];
    let ports = [Port::at(0).unwrap(), Port::at(1).unwrap()];
    assert_ne!(launches[0].raw(), launches[1].raw());
    assert_eq!(Launch::at(2).unwrap_err(), Error(abi::BAD_HANDLE as i64));
    assert_eq!(Port::at(2).unwrap_err(), Error(abi::BAD_HANDLE as i64));
    let mut children = [launches[0].spawn(0).unwrap(), launches[1].spawn(0).unwrap()];
    let mut pairs = [ports[0].connect().unwrap(), ports[1].connect().unwrap()];
    let mut clients = pairs.map(|pair| Client::connect(pair, time::after(100).unwrap()).unwrap());
    for generation in 0..16 {
        let index = generation % 2;
        let sibling = 1 - index;
        exchange(&mut clients[index], 1);
        // Leave a reply queued across the other child's cancellation/replacement.
        clients[sibling].begin(b"q", None).unwrap();
        for _ in 0..8 {
            yield_now();
        }
        let old = children[index];
        let old_pair = pairs[index];
        old.cancel().unwrap();
        assert_eq!(old.wait().unwrap(), Outcome::Cancelled);
        clients[index].retire();
        children[index] = launches[index].spawn(generation as u64 + 1).unwrap();
        pairs[index] = ports[index].connect().unwrap();
        clients[index] = Client::connect(pairs[index], time::after(100).unwrap()).unwrap();
        assert_ne!(old.raw(), children[index].raw());
        assert_eq!(old.cancel(), Err(Error(abi::BAD_HANDLE as i64)));
        assert_eq!(
            old_pair.0.send(b"stale"),
            Err(Error(abi::BAD_HANDLE as i64))
        );
        // Collect exactly the call published before replacement, without replay.
        let deadline = time::after(100).unwrap();
        loop {
            if let Some((reply, len, object)) = clients[sibling].poll().unwrap() {
                assert!(object.is_none());
                assert_eq!(&reply[..len], b"q");
                break;
            }
            event::wait(&clients[sibling].interests().unwrap(), deadline).unwrap();
        }
        exchange(&mut clients[index], 2);
        exchange(&mut clients[sibling], 3);
    }
    cathedral_user_runtime::write(
        b"Cathedral supervision: shared sibling replies survived 16 restarts\n",
    )
    .unwrap();
    // Keep transports live until process exit. Rust drop would close them before
    // kernel parent-death cancellation, changing the lifecycle being tested.
    core::mem::forget(clients);
    0 // Both remaining children must be cancelled and reclaimed by parent exit.
}
pub fn failed() -> u64 {
    denied();
    Launch::at(0).unwrap().spawn(0).unwrap();
    let mut client = Client::connect(
        Port::at(0).unwrap().connect().unwrap(),
        time::after(100).unwrap(),
    )
    .unwrap();
    let failed = Launch::at(1).unwrap();
    for _ in 0..8 {
        assert_eq!(failed.spawn(0).unwrap_err(), Error(abi::NO_MEMORY as i64));
        exchange(&mut client, 7);
    }
    core::mem::forget(client); // Parent death owns transport and child teardown.
    0
}
pub fn abandon() -> u64 {
    denied();
    Launch::at(0).unwrap().spawn(0).unwrap();
    Launch::at(1).unwrap().spawn(0).unwrap();
    let mut client = Client::connect(
        Port::at(0).unwrap().connect().unwrap(),
        time::after(100).unwrap(),
    )
    .unwrap();
    exchange(&mut client, 9);
    let input = Port::at(1).unwrap().connect().unwrap();
    super::gates::wait(input.1);
    for _ in 0..16 {
        yield_now();
    }
    core::mem::forget(client); // Leave both providers alive for joint cancellation.
    0
}
pub fn keyboard_waiter() -> u64 {
    for command in [0xd1, 0xdd, 0xdf, 0xfe, 0xff] {
        assert_eq!(
            keyboard::write(true, command),
            Err(Error(abi::INVALID_ARGUMENT as i64))
        );
    }
    for _ in 0..65 {
        if keyboard::read(false) == Err(Error(abi::WOULD_BLOCK as i64)) {
            break;
        }
    }
    if cathedral_user_runtime::time::now().is_ok() {
        assert_eq!(keyboard::read_until(0), Err(Error(abi::TIMED_OUT as i64)));
        assert_eq!(
            keyboard::read_until(cathedral_user_runtime::time::after(3).unwrap()),
            Err(Error(abi::TIMED_OUT as i64))
        );
    } else {
        assert_eq!(keyboard::read_until(0), Err(Error(abi::DENIED as i64)));
    }
    super::gates::post(Handle::bootstrap(1).unwrap());
    if cathedral_user_runtime::time::now().is_ok() {
        keyboard::read_until(cathedral_user_runtime::time::after(1000).unwrap()).unwrap();
    } else {
        keyboard::read(true).unwrap();
    }
    253 // Cancellation must wake/reap a provider blocked on its raw input queue.
}
