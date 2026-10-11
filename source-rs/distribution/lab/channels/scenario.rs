//! Prove local names, checked move/copyout, worker death, quotas and slot reuse.
use super::*;
use cathedral_user_runtime::task::{Launch, Outcome, Port};
pub fn run() -> u64 {
    probes::creation();
    quota();
    for _ in 0..32 {
        exchange();
    }
    for _ in 0..16 {
        abandoned();
    }
    closed_wait();
    startup::run();
    tree::run();
    quota();
    cathedral_user_runtime::write(
        b"Cathedral IPC: channels, startup bundles, recursive admission and reclamation passed\n",
    )
    .unwrap();
    let (_, idle) = ipc::channel().unwrap();
    let _ = idle.receive(&mut [0; 64]);
    0
}
fn exchange() {
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
fn abandoned() {
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
fn quota() {
    let channels: [_; abi::MAX_CREATED_CHANNELS] =
        core::array::from_fn(|_| ipc::channel().unwrap());
    assert!(matches!(ipc::channel(), Err(Error(code)) if code == abi::NO_MEMORY as i64));
    let (send, recv) = channels[0];
    send.send(b"held").unwrap();
    assert_eq!(send.send(b"full"), Err(Error(abi::WOULD_BLOCK as i64)));
    assert_eq!(recv.receive(&mut [0; 1]), Err(Error(abi::TOO_SMALL as i64)));
    assert_eq!(recv.receive(&mut [0; 64]).unwrap(), 4);
    for (send, recv) in channels {
        send.close().unwrap();
        recv.close().unwrap();
    }
}

fn closed_wait() {
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
