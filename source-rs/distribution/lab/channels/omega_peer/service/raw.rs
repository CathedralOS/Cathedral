//! Explicit v4 handoffs for malformed and separated-arrival probes.
use super::*;
use cathedral_user_runtime::{
    ipc::Handle,
    mailbox::Mailbox,
    memory::{
        Object,
        shared::{Reader, Writer},
    },
    notification::Notification,
};
pub(super) struct Peer {
    pub send: Handle,
    recv: Handle,
    mailbox: Mailbox,
    bell: [Notification; 3],
    signal: Notification,
}
impl Peer {
    pub fn open(send: Handle, recv: Handle) -> Self {
        let writer = Writer::allocate(1).unwrap();
        let bell = Notification::create().unwrap();
        send.send(wire::OPEN).unwrap();
        ack(recv);
        writer.delegate(send).unwrap();
        ack(recv);
        bell[1].delegate(send, b"").unwrap();
        ipc::wait_many(&[recv], time::after(300).unwrap()).unwrap();
        let reader = Reader::accept(recv).unwrap();
        send.send(wire::ACK).unwrap();
        ipc::wait_many(&[recv], time::after(300).unwrap()).unwrap();
        let (_, signal) = Notification::accept(recv, &mut [0; 64]).unwrap();
        let signal = signal.unwrap();
        send.send(wire::COMMIT).unwrap();
        ack(recv);
        Self {
            send,
            recv,
            mailbox: Mailbox::new(writer, reader).unwrap(),
            bell,
            signal,
        }
    }
    pub fn grant(&self, id: u64, object: &Object) {
        self.send.send_memory(&id.to_le_bytes(), object).unwrap();
    }
    pub fn frame(&mut self, id: u64, attachment: wire::Attachment) {
        let (bytes, n) =
            wire::encode(id, &matrix::encode([11, 0, 0, 0, 0, 0, 0, 0]), attachment).unwrap();
        self.mailbox.try_send(&bytes[..n]).unwrap();
        self.signal.signal(false).unwrap();
    }
    pub fn reply(&mut self, id: u64) {
        let deadline = time::after(300).unwrap();
        let mut object = None;
        let mut frame = false;
        while object.is_none() || !frame {
            if object.is_none() {
                let mut packet = [0; 64];
                match self.recv.receive_memory(&mut packet) {
                    Ok((n, received)) => {
                        assert_eq!(n, 8);
                        assert_eq!(&packet[..n], &id.to_le_bytes());
                        object = Some(received.expect("typed reply"));
                    }
                    Err(error) if error.0 == abi::WOULD_BLOCK as i64 => {}
                    Err(error) => panic!("reply grant: {error:?}"),
                }
            }
            if !frame {
                let mut bytes = [0; 80];
                if let Some(n) = self.mailbox.try_receive(&mut bytes).unwrap() {
                    let (got, payload, kind) = wire::decode(&bytes[..n]).unwrap();
                    assert_eq!(got, id);
                    assert_eq!(kind, wire::Attachment::Memory);
                    assert_eq!(payload, matrix::encode([11, 0, 0, 0, 0, 0, 0, 0]));
                    frame = true;
                }
            }
            assert!(!time::reached(time::now().unwrap(), deadline));
            if object.is_none() || !frame {
                let _ = cathedral_user_runtime::event::wait(&[], time::after(1).unwrap());
            }
        }
        assert!(object.unwrap().map().unwrap().iter().all(|&byte| byte == 0));
    }
    pub fn notification(&self) {
        self.bell[1]
            .delegate(self.send, &1u64.to_le_bytes())
            .unwrap();
    }
}
impl Drop for Peer {
    fn drop(&mut self) {
        self.bell[2].close_object().unwrap();
        for right in self.bell.into_iter().chain([self.signal]) {
            right.release().unwrap();
        }
        self.send.close().ok();
        self.recv.close().ok();
    }
}
fn ack(recv: Handle) {
    let mut bytes = [0; 64];
    let n = recv
        .receive_until(&mut bytes, time::after(300).unwrap())
        .unwrap();
    assert_eq!(&bytes[..n], wire::ACK);
}
