//! Same notification API over SVC on ARM64; no ISA logic in the wake protocol.
use crate::Handle;
use cathedral_user_runtime::notification::{Notification as N, Outcome};
pub(super) fn server(input: Handle, output: Handle) {
    let (_, wait) = N::accept(input, &mut [0; 64]).unwrap();
    output.send(b"wait installed").unwrap();
    let (_, signal) = N::accept(input, &mut [0; 64]).unwrap();
    output.send(b"signal installed").unwrap();
    let (wait, signal) = (wait.unwrap(), signal.unwrap());
    assert_eq!(wait.wait(None), Ok(Outcome::Signaled));
    for _ in 1..32 {
        assert_eq!(signal.signal_and_wait(wait, None), Ok(Outcome::Signaled));
    }
    signal.signal(false).unwrap();
    wait.release().unwrap();
    signal.release().unwrap();
}
pub(super) fn client(input: Handle, output: Handle) {
    let request = N::create().unwrap();
    let reply = N::create().unwrap();
    request[0].delegate(output, b"request").unwrap();
    input.receive(&mut [0; 64]).unwrap();
    reply[1].delegate(output, b"reply").unwrap();
    input.receive(&mut [0; 64]).unwrap();
    for _ in 0..32 {
        assert_eq!(
            request[1].signal_and_wait(reply[0], None),
            Ok(Outcome::Signaled)
        );
    }
    for grant in request.into_iter().chain(reply) {
        grant.release().unwrap();
    }
    cathedral_user_runtime::write(b"Cathedral ARM64: 32 notification round trips passed\n")
        .unwrap();
}
