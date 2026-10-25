// Retained connection custody. Setup transfers grants; ordinary frames live only
// in the mailbox. Original endpoints remain a lifetime/abort lane, never a fallback.
// client.rs sequences calls; setup.rs advances server acceptance between dispatches.
use crate::{
    Error, abi,
    event::{self, Interest},
    ipc::Handle,
    mailbox::Mailbox,
    memory::shared::{Reader, Writer},
    notification::{Notification as N, Outcome},
    time,
};
use cathedral_platform_contracts::connection as wire;
type Pair = (Handle, Handle);
struct Lease(Pair);
impl Drop for Lease {
    fn drop(&mut self) {
        self.0.0.close().ok();
        self.0.1.close().ok();
    }
}
struct Bell([N; 3]);
impl Bell {
    fn new() -> Result<Self, Error> {
        N::create().map(Self)
    }
}
impl Drop for Bell {
    fn drop(&mut self) {
        self.0[2].close_object().ok();
        for h in self.0 {
            h.release().ok();
        }
    }
}
struct Signal(N);
impl Drop for Signal {
    fn drop(&mut self) {
        self.0.release().ok();
    }
}
pub(crate) struct Peer {
    lease: Lease,
    mailbox: Mailbox,
    bell: Bell,
    signal: Signal,
}
impl Peer {
    pub(crate) fn wake(&self) -> N {
        self.bell.0[0]
    }
    pub(crate) fn receive(&mut self) -> Result<Option<(u64, [u8; 64], usize)>, Error> {
        // Clear the old hint before testing publication; a later signal remains pending.
        let hint = self.check();
        let mut frame = [0; wire::FRAME_BYTES];
        let Some(n) = self.mailbox.try_receive(&mut frame)? else {
            hint?;
            return Ok(None);
        };
        let (id, payload) = wire::decode(&frame[..n]).ok_or(Error(abi::IO_ERROR as i64))?;
        let mut bytes = [0; 64];
        bytes[..payload.len()].copy_from_slice(payload);
        Ok(Some((id, bytes, payload.len())))
    }
    pub(crate) fn check(&self) -> Result<(), Error> {
        match self.wake().wait(Some(time::now()?))? {
            Outcome::Signaled | Outcome::TimedOut => (),
            _ => return Err(Error(abi::PEER_CLOSED as i64)),
        }
        match self.lease.0.1.receive_until(&mut [0; 64], time::now()?) {
            Err(Error(c)) if c == abi::TIMED_OUT as i64 => Ok(()),
            Err(e) => Err(e),
            Ok(_) => Err(Error(abi::IO_ERROR as i64)),
        }
    }
    pub(crate) fn send(&mut self, id: u64, bytes: &[u8]) -> Result<(), Error> {
        let (frame, n) = wire::encode(id, bytes).ok_or(Error(abi::INVALID_ARGUMENT as i64))?;
        self.mailbox.try_send(&frame[..n])?;
        self.signal.0.signal(false) // Failure may follow publication: retire, never retry.
    }
    fn wait(&self, deadline: u64) -> Result<(), Error> {
        event::wait(
            &[
                Interest::Notification(self.wake()),
                Interest::Ipc(self.lease.0.1),
            ],
            deadline,
        )
        .map(|_| ())
    }
}
fn ready(receive: Handle, deadline: u64) -> Result<(), Error> {
    event::wait(&[Interest::Ipc(receive)], deadline).map(|_| ())
}
fn ack(receive: Handle, deadline: u64) -> Result<(), Error> {
    let mut bytes = [0; 64];
    let n = receive.receive_until(&mut bytes, deadline)?;
    if &bytes[..n] == wire::ACK {
        Ok(())
    } else {
        Err(Error(abi::IO_ERROR as i64))
    }
}
