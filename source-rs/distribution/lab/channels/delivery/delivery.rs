// Parent-side endpoint handoff, queued-message lifetime, quotas and peer closure.
use crate::{Error, Handle, abi, ipc, receive, time};
use cathedral_user_runtime::task::{Launch, Outcome, Port};

pub(super) fn exchange() {
    let child = Launch::bootstrap().unwrap().spawn(0).unwrap();
    let (control, events) = Port::bootstrap().unwrap().connect().unwrap();
    let (requests, worker_input) = ipc::channel().unwrap();
    let (worker_output, answers) = ipc::channel().unwrap();
    // The worker has no dynamic grants yet: these payload bits grant nothing.
    control.send(&worker_input.raw().to_le_bytes()).unwrap();
    receive(events);
    probes::send(control, worker_input);
    control.send_with(b"requests", worker_input).unwrap();
    assert_eq!(worker_input.close(), Err(Error(abi::BAD_HANDLE as i64)));
    receive(events);
    // Parent parks on an empty local channel, allowing the worker to block in
    // receive_with before the capability-bearing send completes that saved wait.
    assert_eq!(
        answers.receive_until(&mut [0; 64], time::after(10).unwrap()),
        Err(Error(abi::TIMED_OUT as i64))
    );
    control.send_with(b"answers", worker_output).unwrap();
    receive(events);
    requests.send(b"ordinary app work").unwrap();
    let (answer, cap) = receive(answers);
    assert!(cap.is_none());
    assert_eq!(&answer[..17], b"ordinary app work");
    assert_eq!(
        child.wait_until(time::after(300).unwrap()).unwrap(),
        Outcome::Returned(42)
    );
    assert_eq!(
        answers.receive(&mut [0; 64]),
        Err(Error(abi::PEER_CLOSED as i64))
    );
    assert_eq!(requests.send(b"late"), Err(Error(abi::PEER_CLOSED as i64)));
    requests.close().unwrap();
    answers.close().unwrap();
    assert_eq!(requests.send(b"stale"), Err(Error(abi::BAD_HANDLE as i64)));
}
pub(super) fn abandoned() {
    let child = Launch::bootstrap().unwrap().spawn(1).unwrap();
    let (control, events) = Port::bootstrap().unwrap().connect().unwrap();
    receive(events); // Worker is parked on its own private channel.
    let (send, recv) = ipc::channel().unwrap();
    control.send_with(b"not accepted", send).unwrap();
    child.cancel().unwrap();
    assert_eq!(child.wait().unwrap(), Outcome::Cancelled);
    assert_eq!(
        recv.receive(&mut [0; 64]),
        Err(Error(abi::PEER_CLOSED as i64))
    );
    recv.close().unwrap();
}
pub(super) fn quota() {
    use cathedral_user_runtime::notification::Notification;
    let before = ipc::budget().unwrap();
    let available = (before.channel_limit - before.charged_channels) as usize;
    let handles = (before.handle_limit - before.charged_handles) as usize;
    let limit = available.min(handles / 2 - 1);
    // The service fixture funds more objects than fit as endpoint pairs. Charge
    // the excess as notifications so this still proves object-quota refusal
    // with endpoint metadata spare, rather than hitting the handle ceiling first.
    let mut bells = [None; abi::NOTIFY_HANDLES];
    for bell in &mut bells[..available - limit] {
        let [wait, signal, control] = Notification::create().unwrap();
        // This quota probe needs lifetime custody only, not signal/close rights.
        // Releasing the final wait below refunds the retained object charge.
        signal.release().unwrap();
        control.release().unwrap();
        *bell = Some(wait);
    }
    let mut channels = [None; abi::MAX_DYNAMIC_HANDLES / 2];
    for slot in &mut channels[..limit] {
        *slot = Some(ipc::channel().unwrap());
    }
    let full = ipc::budget().unwrap();
    assert_eq!(full.charged_channels, full.channel_limit);
    assert!(full.charged_handles + 2 <= full.handle_limit);
    assert!(matches!(ipc::channel(), Err(Error(code)) if code == abi::NO_MEMORY as i64));
    let (send, recv) = channels[0].unwrap();
    send.send(b"held").unwrap();
    assert_eq!(send.send(b"full"), Err(Error(abi::WOULD_BLOCK as i64)));
    assert_eq!(recv.receive(&mut [0; 1]), Err(Error(abi::TOO_SMALL as i64)));
    assert_eq!(recv.receive(&mut [0; 64]).unwrap(), 4);
    for (send, recv) in channels.into_iter().flatten() {
        send.close().unwrap();
        recv.close().unwrap();
    }
    for wait in bells.into_iter().flatten() {
        wait.release().unwrap();
    }
    let after = ipc::budget().unwrap();
    assert_eq!(after.charged_channels, before.charged_channels);
    assert_eq!(after.charged_handles, before.charged_handles);
}

pub(super) fn closed_wait() {
    let child = Launch::bootstrap().unwrap().spawn(2).unwrap();
    let (control, events) = Port::bootstrap().unwrap().connect().unwrap();
    receive(events);
    assert_eq!(
        events.receive_until(&mut [0; 64], time::after(10).unwrap()),
        Err(Error(abi::TIMED_OUT as i64))
    );
    control.close().unwrap();
    assert_eq!(
        child.wait_until(time::after(300).unwrap()).unwrap(),
        Outcome::Returned(43)
    );
}
