//! A deliberately incomplete/mismatched sender must not occupy service dispatch.
use super::*;
use cathedral_user_runtime::{
    mailbox::Mailbox,
    memory::{
        Private,
        shared::{Reader, Writer},
    },
};
pub(super) fn run(launch: Launch) {
    for mode in 65..=67 {
        let (send, input) = ipc::channel().unwrap();
        let (output, recv) = ipc::channel().unwrap();
        let child = launch.spawn_with(mode, &[input, output]).unwrap();
        let (control, events) = Port::bootstrap().unwrap().connect().unwrap();
        let writer = Writer::allocate(1).unwrap();
        let bell = N::create().unwrap();
        send.send(wire::OPEN).unwrap();
        ack(recv);
        writer.delegate(send).unwrap();
        ack(recv);
        bell[1].delegate(send, b"").unwrap();
        ipc::wait_many(&[recv], time::after(300).unwrap()).unwrap();
        let reader = Reader::accept(recv).unwrap();
        send.send(wire::ACK).unwrap();
        ipc::wait_many(&[recv], time::after(300).unwrap()).unwrap();
        let (_, signal) = N::accept(recv, &mut [0; 64]).unwrap();
        let signal = signal.unwrap();
        send.send(wire::COMMIT).unwrap();
        ack(recv);
        let mut mailbox = Mailbox::new(writer, reader).unwrap();
        if mode != 66 {
            let object = Private::allocate(1).unwrap().freeze().unwrap();
            send.send_memory(
                &(if mode == 67 { 2u64 } else { 1u64 }).to_le_bytes(),
                &object,
            )
            .unwrap();
        }
        if mode != 65 {
            let (frame, n) = wire::encode(1, b"never dispatch", true).unwrap();
            mailbox.try_send(&frame[..n]).unwrap();
        }
        signal.signal(false).unwrap();
        if mode != 67 {
            control.send(b"alive").unwrap();
            assert_eq!(&receive(events).0[..5], b"alive");
        }
        let mut bytes = [0; 64];
        let n = events
            .receive_until(&mut bytes, time::after(600).unwrap())
            .unwrap();
        assert_eq!(&bytes[..n], b"gone");
        assert_eq!(
            child.wait_until(time::after(300).unwrap()).unwrap(),
            Outcome::Returned(0)
        );
        for handle in bell.into_iter().chain([signal]) {
            handle.release().unwrap();
        }
        send.close().unwrap();
        recv.close().unwrap();
    }
}
fn ack(recv: Handle) {
    let mut bytes = [0; 64];
    let n = recv
        .receive_until(&mut bytes, time::after(300).unwrap())
        .unwrap();
    assert_eq!(&bytes[..n], wire::ACK);
}
