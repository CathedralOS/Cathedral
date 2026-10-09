//! Independent launch grants, sibling IPC continuity and joint teardown probes.
use cathedral_contracts::user as abi;
use cathedral_user_runtime::{
    Error,
    ipc::Handle,
    keyboard,
    task::{Launch, Outcome, Port},
    yield_now,
};

fn denied() {
    assert_eq!(keyboard::read(false), Err(Error(abi::DENIED as i64)));
    assert_eq!(keyboard::write(false, 0xf4), Err(Error(abi::DENIED as i64)));
}
fn exchange(pair: (Handle, Handle), value: u8) {
    pair.0.send(&[value]).unwrap();
    let mut bytes = [0; 64];
    assert_eq!(pair.1.receive(&mut bytes).unwrap(), 1);
    assert_eq!(bytes[0], value);
}
pub fn echo() -> u64 {
    denied();
    assert_eq!(Launch::at(0).unwrap_err(), Error(abi::DENIED as i64));
    let input = Handle::bootstrap(0).unwrap();
    let output = Handle::bootstrap(1).unwrap();
    loop {
        let mut bytes = [0; 64];
        let length = input.receive(&mut bytes).unwrap();
        output.send(&bytes[..length]).unwrap();
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
    for generation in 0..16 {
        let index = generation % 2;
        let sibling = 1 - index;
        exchange(pairs[index], 1);
        // Leave a reply queued across the other child's cancellation/replacement.
        pairs[sibling].0.send(b"q").unwrap();
        for _ in 0..8 {
            yield_now();
        }
        let old = children[index];
        let old_pair = pairs[index];
        old.cancel().unwrap();
        assert_eq!(old.wait().unwrap(), Outcome::Cancelled);
        children[index] = launches[index].spawn(generation as u64 + 1).unwrap();
        pairs[index] = ports[index].connect().unwrap();
        assert_ne!(old.raw(), children[index].raw());
        assert_eq!(old.cancel(), Err(Error(abi::BAD_HANDLE as i64)));
        assert_eq!(
            old_pair.0.send(b"stale"),
            Err(Error(abi::BAD_HANDLE as i64))
        );
        let mut reply = [0; 64];
        assert_eq!(pairs[sibling].1.receive(&mut reply).unwrap(), 1);
        assert_eq!(reply[0], b'q');
        exchange(pairs[index], 2);
        exchange(pairs[sibling], 3);
    }
    0 // Both remaining children must be cancelled and reclaimed by parent exit.
}
pub fn failed() -> u64 {
    denied();
    Launch::at(0).unwrap().spawn(0).unwrap();
    let pair = Port::at(0).unwrap().connect().unwrap();
    let failed = Launch::at(1).unwrap();
    for _ in 0..8 {
        assert_eq!(failed.spawn(0).unwrap_err(), Error(abi::NO_MEMORY as i64));
        exchange(pair, 7);
    }
    0
}
pub fn abandon() -> u64 {
    denied();
    Launch::at(0).unwrap().spawn(0).unwrap();
    Launch::at(1).unwrap().spawn(0).unwrap();
    exchange(Port::at(0).unwrap().connect().unwrap(), 9);
    let input = Port::at(1).unwrap().connect().unwrap();
    assert_eq!(input.1.receive(&mut [0; 64]).unwrap(), 5);
    for _ in 0..16 {
        yield_now();
    }
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
    Handle::bootstrap(1).unwrap().send(b"ready").unwrap();
    if cathedral_user_runtime::time::now().is_ok() {
        keyboard::read_until(cathedral_user_runtime::time::after(1000).unwrap()).unwrap();
    } else {
        keyboard::read(true).unwrap();
    }
    253 // Cancellation must wake/reap a provider blocked on its raw input queue.
}
