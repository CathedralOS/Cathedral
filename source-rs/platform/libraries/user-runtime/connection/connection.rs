// Retained connection custody. Setup transfers grants; ordinary frames live only
// in the mailbox. Endpoints mediate typed grants and lifetime, never payload fallback.
// client.rs sequences calls; setup.rs advances server acceptance between dispatches.
use crate::{
    Error, abi,
    event::{self, Interest},
    ipc::Handle,
    mailbox::Mailbox,
    memory::{
        Object,
        shared::{Reader, Writer},
    },
    notification::{Notification as N, Outcome},
    time,
};
use cathedral_platform_contracts::connection as wire;
use custody::{Bell, Lease, Signal};
use incoming::{Incoming, Message};
type Pair = (Handle, Handle);
pub(crate) struct Peer {
    lease: Lease,
    mailbox: Mailbox,
    bell: Bell,
    signal: Signal,
    incoming: Incoming,
}
impl Peer {
    pub(crate) fn wake(&self) -> N {
        self.bell.0[0]
    }
    pub(crate) fn receive(&mut self) -> Result<Option<Message>, Error> {
        // Drain before checking publication. A complete final frame/grant pair
        // may outlive the peer, but malformed typed handoffs never become replies.
        let hint = self.hint();
        let transfer = self.incoming.transfer(self.lease.0.1);
        if let Err(error) = &transfer
            && error.0 != abi::PEER_CLOSED as i64
        {
            return Err(Error(error.0));
        }
        if let Some(message) = self.incoming.receive(&mut self.mailbox)? {
            return Ok(Some(message));
        }
        transfer?;
        hint?;
        Ok(None)
    }
    fn hint(&self) -> Result<(), Error> {
        match self.wake().wait(Some(time::now()?))? {
            Outcome::Signaled | Outcome::TimedOut => Ok(()),
            _ => Err(Error(abi::PEER_CLOSED as i64)),
        }
    }
    pub(crate) fn check(&mut self) -> Result<(), Error> {
        self.hint()?;
        self.incoming.transfer(self.lease.0.1)
    }
    pub(crate) fn deadline(&self) -> Option<u64> {
        self.incoming.deadline
    }
    pub(crate) fn send(
        &mut self,
        id: u64,
        bytes: &[u8],
        object: Option<&Object>,
    ) -> Result<(), SendFailure> {
        let (frame, n) = wire::encode(id, bytes, object.is_some()).ok_or(SendFailure {
            error: Error(abi::INVALID_ARGUMENT as i64),
            exposed: false,
        })?;
        if let Some(object) = object {
            self.lease
                .0
                .0
                .send_memory(&id.to_le_bytes(), object)
                .map_err(|error| SendFailure {
                    error,
                    exposed: false,
                })?;
        }
        self.mailbox
            .try_send(&frame[..n])
            .map_err(|error| SendFailure {
                error,
                exposed: object.is_some(),
            })?;
        self.signal.0.signal(false).map_err(|error| SendFailure {
            error,
            exposed: true,
        })
    }
    fn wait(&self, mut deadline: u64) -> Result<(), Error> {
        if let Some(half) = self.deadline()
            && time::reached(deadline, half)
        {
            deadline = half;
        }
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
pub(crate) struct SendFailure {
    pub error: Error,
    pub exposed: bool,
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
