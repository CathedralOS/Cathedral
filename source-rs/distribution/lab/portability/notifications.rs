//! Shared mailbox payloads and notifications through SVC, using the same runtime.
use crate::{Error, Handle, abi};
use cathedral_user_runtime::{
    event::{self, Interest},
    mailbox::Mailbox,
    memory::shared::{Reader, Writer},
    notification::{Notification as N, Outcome},
    time,
};
pub(super) fn server(input: Handle, output: Handle) {
    let reader = Reader::accept(input).unwrap();
    let writer = Writer::allocate(1).unwrap();
    writer.delegate(output).unwrap();
    let mut mailbox = Mailbox::new(writer, reader).unwrap();
    let (_, wait) = N::accept(input, &mut [0; 64]).unwrap();
    output.send(b"wait installed").unwrap();
    let (_, signal) = N::accept(input, &mut [0; 64]).unwrap();
    output.send(b"signal installed").unwrap();
    let (wait, signal) = (wait.unwrap(), signal.unwrap());
    for round in 0..32 {
        let mut bytes = [0; 256];
        loop {
            if let Some(n) = mailbox.try_receive(&mut bytes).unwrap() {
                assert_eq!(n, 256);
                assert_eq!(bytes, [round; 256]);
                break;
            }
            assert_eq!(wait.wait(None), Ok(Outcome::Signaled));
        }
        mailbox.try_send(&bytes).unwrap();
        signal.signal(false).unwrap();
    }
    wait.release().unwrap();
    signal.release().unwrap();
}
pub(super) fn client(input: Handle, output: Handle) {
    let writer = Writer::allocate(1).unwrap();
    writer.delegate(output).unwrap();
    let reader = Reader::accept(input).unwrap();
    let mut mailbox = Mailbox::new(writer, reader).unwrap();
    let request = N::create().unwrap();
    let reply = N::create().unwrap();
    request[0].delegate(output, b"request").unwrap();
    input.receive(&mut [0; 64]).unwrap();
    reply[1].delegate(output, b"reply").unwrap();
    input.receive(&mut [0; 64]).unwrap();
    // Cross the entire mixed-wait ABI through SVC, including the final slot.
    // Invalid trailing authority must win over an already-ready first interest.
    let mut interests = [Interest::Ipc(input); abi::MAX_WAIT_HANDLES];
    interests[abi::MAX_WAIT_HANDLES - 1] = Interest::Notification(reply[0]);
    reply[1].signal(false).unwrap();
    assert_eq!(
        event::wait(&interests, time::after(300).unwrap()),
        Ok(abi::MAX_WAIT_HANDLES - 1)
    );
    interests[0] = Interest::Notification(reply[0]);
    interests[abi::MAX_WAIT_HANDLES - 1] = Interest::Ipc(Handle::from_raw(0));
    assert_eq!(
        event::wait(&interests, time::now().unwrap()),
        Err(Error(abi::BAD_HANDLE as i64))
    );
    assert_eq!(
        reply[0].wait(Some(time::now().unwrap())),
        Ok(Outcome::Signaled)
    );
    cathedral_user_runtime::write(b"Cathedral ARM64: final wait slot and invalid tail passed\n")
        .unwrap();
    for round in 0..32 {
        mailbox.try_send(&[round; 256]).unwrap();
        request[1].signal(false).unwrap();
        let mut bytes = [0; 256];
        loop {
            if let Some(n) = mailbox.try_receive(&mut bytes).unwrap() {
                assert_eq!(n, 256);
                assert_eq!(bytes, [round; 256]);
                break;
            }
            assert_eq!(
                event::wait(
                    &[Interest::Notification(reply[0]), Interest::Ipc(input)],
                    time::after(300).unwrap()
                ),
                Ok(0)
            );
            assert_eq!(
                reply[0].wait(Some(time::now().unwrap())),
                Ok(Outcome::Signaled)
            );
        }
    }
    for grant in request.into_iter().chain(reply) {
        grant.release().unwrap();
    }
    cathedral_user_runtime::write(b"Cathedral ARM64: 32 notification round trips passed\n")
        .unwrap();
    cathedral_user_runtime::write(
        b"Cathedral ARM64: 32 shared mailbox payload round trips passed\n",
    )
    .unwrap();
}
