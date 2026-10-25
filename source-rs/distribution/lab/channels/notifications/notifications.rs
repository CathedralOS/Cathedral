// Real traps and independent processes: establish rights once, then exchange
// remembered notifications with no per-turn payload delivery or handle creation.
use super::*;
use cathedral_user_runtime::{
    notification::{self, Notification as N, Outcome as Wake},
    task::{Launch, Outcome, Port},
};
const ROUNDS: usize = 64;
pub(super) fn run() {
    let baseline = notification::budget().unwrap();
    assert_eq!(syscall::call(abi::NOTIFY_CREATE, 1, 0, 0), abi::BAD_ADDRESS);
    assert_eq!(notification::budget().unwrap(), baseline);
    let [wait, signal, control] = N::create().unwrap();
    assert_eq!(wait.signal(false), Err(Error(abi::DENIED as i64)));
    assert_eq!(signal.close_object(), Err(Error(abi::DENIED as i64)));
    signal.signal(false).unwrap();
    signal.signal(true).unwrap();
    assert_eq!(wait.wait(Some(time::now().unwrap())), Ok(Wake::Signaled));
    assert_eq!(wait.wait(Some(time::now().unwrap())), Ok(Wake::TimedOut));
    assert_eq!(
        signal.signal_and_wait(signal, None),
        Err(Error(abi::DENIED as i64))
    );
    assert_eq!(wait.wait(Some(time::now().unwrap())), Ok(Wake::TimedOut));
    let replies = N::create().unwrap();
    let reply = replies[0];
    assert_eq!(
        signal.signal_and_wait(reply, Some(time::now().unwrap())),
        Ok(Wake::TimedOut)
    );
    assert_eq!(wait.wait(None), Ok(Wake::Signaled));
    wait.release().unwrap();
    signal.release().unwrap();
    control.release().unwrap();
    for right in replies {
        right.release().unwrap();
    }
    ping_pong();
    broadcast(false);
    broadcast(true);
    for mode in 11..=14 {
        terminal(mode);
    }
    creator_death();
    assert_eq!(notification::budget().unwrap(), baseline);
    cathedral_user_runtime::write(b"Cathedral notifications: ping-pong, early signals, cancellation, closure, withdrawal and retained funding passed\n").unwrap();
}
fn introduce(control: Handle, events: Handle, handles: &[N]) {
    assert_eq!(
        syscall::call(abi::NOTIFY_SEND, control.raw(), 1, 0),
        abi::BAD_ADDRESS
    );
    control.send(&handles[0].raw().to_le_bytes()).unwrap();
    receive(events); // The raw number did not install a notification.
    for &handle in handles {
        handle.delegate(control, b"right").unwrap();
        receive(events);
    }
}
fn ping_pong() {
    let request = N::create().unwrap();
    let reply = N::create().unwrap();
    let child = Launch::bootstrap().unwrap().spawn(10).unwrap();
    let (control, events) = Port::bootstrap().unwrap().connect().unwrap();
    introduce(control, events, &[request[0], reply[1]]);
    for _ in 0..ROUNDS {
        assert_eq!(
            request[1].signal_and_wait(reply[0], Some(time::after(300).unwrap())),
            Ok(Wake::Signaled)
        );
    }
    assert_eq!(
        child.wait_until(time::after(300).unwrap()).unwrap(),
        Outcome::Returned(0)
    );
    for right in request.into_iter().chain(reply) {
        right.release().unwrap();
    }
}
fn terminal(mode: u64) {
    let request = N::create().unwrap();
    let ready = N::create().unwrap();
    let child = Launch::bootstrap().unwrap().spawn(mode).unwrap();
    let (control, events) = Port::bootstrap().unwrap().connect().unwrap();
    introduce(control, events, &[request[0], ready[1]]);
    // The worker signals ready and registers its wait in one serialized syscall.
    assert_eq!(
        ready[0].wait(Some(time::after(300).unwrap())),
        Ok(Wake::Signaled)
    );
    match mode {
        11 => request[2].cancel_waits().unwrap(),
        12 => request[2].withdraw_waits().unwrap(),
        13 => request[2].close_object().unwrap(),
        _ => child.cancel().unwrap(),
    }
    let expected = if mode == 14 {
        Outcome::Cancelled
    } else {
        Outcome::Returned(0)
    };
    assert_eq!(
        child.wait_until(time::after(300).unwrap()).unwrap(),
        expected
    );
    if mode == 14 {
        request[1].signal(false).unwrap();
        assert_eq!(request[0].wait(None), Ok(Wake::Signaled));
    }
    for right in request.into_iter().chain(ready) {
        right.release().unwrap();
    }
}
fn creator_death() {
    let child = Launch::bootstrap().unwrap().spawn(15).unwrap();
    let (control, events) = Port::bootstrap().unwrap().connect().unwrap();
    let (_, right) = N::accept(events, &mut [0; 64]).unwrap();
    let right = right.unwrap();
    control.send(b"accepted").unwrap();
    assert_eq!(child.wait().unwrap(), Outcome::Returned(0));
    right.signal(false).unwrap();
    let next = Launch::bootstrap().unwrap().spawn(16).unwrap();
    let (control, events) = Port::bootstrap().unwrap().connect().unwrap();
    receive(events); // New incarnation observed its predecessor's retained charge.
    right.release().unwrap();
    assert_eq!(right.signal(false), Err(Error(abi::BAD_HANDLE as i64)));
    control.send(b"released").unwrap();
    assert_eq!(next.wait().unwrap(), Outcome::Returned(0));
}

fn broadcast(all: bool) {
    let request = N::create().unwrap();
    let ready = N::create().unwrap();
    let peers: [_; 2] = core::array::from_fn(|index| {
        let task = Launch::at(index as u64).unwrap().spawn(17).unwrap();
        let (control, events) = Port::at(index as u64).unwrap().connect().unwrap();
        introduce(control, events, &[request[0], ready[1]]);
        assert_eq!(
            ready[0].wait(Some(time::after(300).unwrap())),
            Ok(Wake::Signaled)
        );
        (task, events)
    });
    request[1].signal(all).unwrap();
    let selected = ipc::wait_two(peers[0].1, peers[1].1, time::after(300).unwrap()).unwrap();
    receive(peers[selected].1);
    let other = 1 - selected;
    if !all {
        assert_eq!(
            peers[other]
                .1
                .receive_until(&mut [0; 64], time::now().unwrap()),
            Err(Error(abi::TIMED_OUT as i64))
        );
        request[1].signal(true).unwrap();
    }
    receive(peers[other].1);
    for (task, _) in peers {
        assert_eq!(task.wait().unwrap(), Outcome::Returned(0));
    }
    assert_eq!(
        request[0].wait(Some(time::now().unwrap())),
        Ok(Wake::TimedOut)
    );
    for right in request.into_iter().chain(ready) {
        right.release().unwrap();
    }
}
