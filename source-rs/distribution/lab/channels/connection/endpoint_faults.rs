//! Refusal before endpoint publication preserves custody; failure after moves it.
use super::*;
use cathedral_user_runtime::{
    mailbox::Mailbox,
    memory::shared::{Reader, Writer},
};
pub(super) fn run(launch: Launch) {
    for mode in [87, 88] {
        let baseline = resources::budget().unwrap();
        let physical = resources::physical_budget().unwrap();
        let (send, input) = ipc::channel().unwrap();
        let (output, recv) = ipc::channel().unwrap();
        let child = launch.spawn_with(mode, &[input, output]).unwrap();
        let (control, events) = Port::bootstrap().unwrap().connect().unwrap();
        let writer = Writer::allocate(1).unwrap();
        let bell = N::create().unwrap();
        send.send(if mode == 88 {
            wire::OPEN_PIPELINE
        } else {
            wire::OPEN
        })
        .unwrap();
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
        if mode == 87 {
            // Server can offer the endpoint and publish its frame, but its final
            // signal fails. The moved right must not be reused by the service.
            bell[2].close_object().unwrap();
            publish(&mut mailbox, signal, 1, b"use");
            let (n, endpoint) = recv.receive_with(&mut [0; 64]).unwrap();
            assert_eq!(n, 8);
            let endpoint = endpoint.unwrap();
            let mut frame = [0; wire::FRAME_BYTES];
            let n = mailbox.try_receive(&mut frame).unwrap().unwrap();
            assert_eq!(
                wire::decode(&frame[..n]),
                Some((1, b"endpoint".as_slice(), wire::Attachment::Endpoint))
            );
            endpoint.send(b"proof").unwrap();
            endpoint.close().unwrap();
        } else {
            publish(&mut mailbox, signal, 1, b"plain");
            assert_eq!(&receive(events).0[..5], b"first");
            // The first response is still unconsumed. The second transfer must
            // refuse before the endpoint changes hands, then queue a plain error.
            publish(&mut mailbox, signal, 2, b"blocked");
            assert_eq!(&receive(events).0[..7], b"refused");
            for id in 1..=2 {
                let mut frame = [0; wire::FRAME_BYTES];
                let n = loop {
                    if let Some(n) = mailbox.try_receive(&mut frame).unwrap() {
                        break n;
                    }
                    assert_eq!(
                        bell[0].wait(Some(time::after(300).unwrap())),
                        Ok(Wake::Signaled)
                    );
                };
                let (received, bytes, kind) = wire::decode(&frame[..n]).unwrap();
                assert_eq!(received, id);
                assert_eq!(kind, wire::Attachment::None);
                assert_eq!(
                    bytes,
                    if id == 1 {
                        b"plain".as_slice()
                    } else {
                        b"capacity"
                    }
                );
                signal.signal(false).unwrap();
            }
            control.send(b"done").unwrap();
        }
        assert_eq!(
            child.wait_until(time::after(300).unwrap()).unwrap(),
            Outcome::Returned(0)
        );
        drop(mailbox);
        for handle in bell.into_iter().chain([signal]) {
            handle.release().unwrap();
        }
        send.close().unwrap();
        recv.close().unwrap();
        assert_eq!(resources::budget().unwrap(), baseline);
        assert_eq!(resources::physical_budget().unwrap(), physical);
    }
}
fn ack(recv: Handle) {
    let mut bytes = [0; 64];
    let n = recv
        .receive_until(&mut bytes, time::after(300).unwrap())
        .unwrap();
    assert_eq!(&bytes[..n], wire::ACK);
}
fn publish(mailbox: &mut Mailbox, signal: N, id: u64, bytes: &[u8]) {
    let (frame, n) = wire::encode(id, bytes, wire::Attachment::None).unwrap();
    mailbox.try_send(&frame[..n]).unwrap();
    signal.signal(false).unwrap();
}
