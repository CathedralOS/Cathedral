//! Shared mailbox payloads and notifications through SVC, using the same runtime.
use crate::Handle;
use cathedral_user_runtime::{
    mailbox::Mailbox,
    memory::shared::{Reader, Writer},
    notification::{Notification as N, Outcome},
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
            assert_eq!(reply[0].wait(None), Ok(Outcome::Signaled));
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
