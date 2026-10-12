//! No introduction messages: a child starts with the parent's chosen connections.
mod probes;
use super::*;
use cathedral_user_runtime::task::{Environment, Launch, Outcome};
pub fn run() {
    let launch = Launch::bootstrap().unwrap();
    for _ in 0..16 {
        exchange(launch, false);
    }
    for _ in 0..8 {
        exchange(launch, true);
    }
    let (s0, r0) = ipc::channel().unwrap();
    let (s1, r1) = ipc::channel().unwrap();
    let child = launch.spawn_with(5, &[s0, r0, s1, r1]).unwrap();
    assert_eq!(child.wait().unwrap(), Outcome::Returned(45));
    let child = launch.spawn_with(6, &[]).unwrap();
    assert_eq!(child.wait().unwrap(), Outcome::Returned(46));
    cathedral_user_runtime::write(b"Cathedral IPC: initial environments available at main; failed bundles retain all grants\n").unwrap();
}
fn exchange(launch: Launch, cancel: bool) {
    let (requests, input) = ipc::channel().unwrap();
    let (output, answers) = ipc::channel().unwrap();
    // This extra authority is deliberately absent from the initial bundle.
    let (private_send, private_recv) = ipc::channel().unwrap();
    probes::check(launch, input);
    assert_eq!(
        launch.spawn_with(3, &[input, input]).unwrap_err(),
        Error(abi::INVALID_ARGUMENT as i64)
    );
    assert_eq!(
        launch
            .spawn_with(3, &[input, Handle::from_raw(0)])
            .unwrap_err(),
        Error(abi::BAD_HANDLE as i64)
    );
    requests.send(b"pending").unwrap();
    assert_eq!(
        launch.spawn_with(3, &[input, output]).unwrap_err(),
        Error(abi::BUSY as i64)
    );
    assert_eq!(input.receive(&mut [0; 64]).unwrap(), 7);
    // A valid tail after several rejected requests proves no partial move/launch.
    let child = launch
        .spawn_with(if cancel { 4 } else { 3 }, &[input, output])
        .unwrap();
    assert_eq!(input.close(), Err(Error(abi::BAD_HANDLE as i64)));
    assert_eq!(output.close(), Err(Error(abi::BAD_HANDLE as i64)));
    assert_eq!(
        launch.spawn_with(3, &[private_recv]).unwrap_err(),
        Error(abi::BUSY as i64)
    );
    private_send.send(b"private").unwrap();
    assert_eq!(private_recv.receive(&mut [0; 64]).unwrap(), 7);
    if cancel {
        assert_eq!(&receive(answers).0[..5], b"ready");
        child.cancel().unwrap();
        assert_eq!(child.wait().unwrap(), Outcome::Cancelled);
    } else {
        requests.send(b"from initial environment").unwrap();
        assert_eq!(&receive(answers).0[..24], b"from initial environment");
        assert_eq!(
            child.wait_until(time::after(300).unwrap()).unwrap(),
            Outcome::Returned(44)
        );
    }
    assert_eq!(requests.send(b"dead"), Err(Error(abi::PEER_CLOSED as i64)));
    assert_eq!(
        answers.receive(&mut [0; 64]),
        Err(Error(abi::PEER_CLOSED as i64))
    );
    for endpoint in [requests, answers, private_send, private_recv] {
        endpoint.close().unwrap();
    }
}
pub fn worker(mode: u64, environment: Environment) -> u64 {
    // The entry macro loaded only this process's startup snapshot before calling us.
    if mode == 6 {
        assert!(environment.is_empty());
        return 46;
    }
    if mode == 5 {
        assert_eq!(environment.len(), 4);
        for i in [0, 2] {
            environment.endpoint(i).unwrap().send(b"both ends").unwrap();
            assert_eq!(
                environment
                    .endpoint(i + 1)
                    .unwrap()
                    .receive(&mut [0; 64])
                    .unwrap(),
                9
            );
        }
        return 45;
    }
    assert_eq!(environment.len(), 2);
    assert!(environment.endpoint(2).is_none());
    assert!(Handle::bootstrap(2).is_err()); // No dynamic/parent grant enumeration.
    assert!(Launch::bootstrap().is_err());
    let input = environment.endpoint(0).unwrap();
    let output = environment.endpoint(1).unwrap();
    if mode == 4 {
        output.send(b"ready").unwrap();
        let _ = input.receive(&mut [0; 64]);
        return 254;
    }
    let mut bytes = [0; 64];
    let len = input
        .receive_until(&mut bytes, time::after(300).unwrap())
        .unwrap();
    input.close().unwrap();
    let again = Environment::load().unwrap().endpoint(0).unwrap();
    assert_eq!(again.raw(), input.raw());
    assert_eq!(
        again.receive(&mut [0; 64]),
        Err(Error(abi::BAD_HANDLE as i64))
    );
    output.send(&bytes[..len]).unwrap();
    44
}
