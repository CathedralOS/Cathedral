// Retained connection custody. Setup transfers grants; ordinary frames live only
// in the mailbox. Endpoints mediate typed grants and lifetime, never payload fallback.
// opening.rs yields client setup; setup.rs yields acceptance; client.rs sequences calls.
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
    pipelined: bool,
}
enum Outbound<'a> {
    None,
    Memory(&'a Object),
    Endpoint(Handle),
    Notification(N),
}
impl Peer {
    pub(crate) fn wake(&self) -> N {
        self.bell.wait
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
            // Consumption frees the peer's publication slot even when this
            // request's application reply is deliberately deferred.
            if self.pipelined {
                let _ = self.signal.0.signal(false);
            }
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
    // One publication boundary also bounds code growth across client/server paths.
    #[inline(never)]
    pub(crate) fn send(
        &mut self,
        id: u64,
        bytes: &[u8],
        object: Option<&Object>,
    ) -> Result<(), SendFailure> {
        self.publish(id, bytes, object.map_or(Outbound::None, Outbound::Memory))
    }
    pub(crate) fn send_endpoint(
        &mut self,
        id: u64,
        bytes: &[u8],
        endpoint: Handle,
    ) -> Result<(), SendFailure> {
        self.publish(id, bytes, Outbound::Endpoint(endpoint))
    }
    pub(crate) fn send_notification(
        &mut self,
        id: u64,
        bytes: &[u8],
        notification: N,
    ) -> Result<(), SendFailure> {
        self.publish(id, bytes, Outbound::Notification(notification))
    }
    #[inline(never)]
    fn publish(
        &mut self,
        id: u64,
        bytes: &[u8],
        attachment: Outbound<'_>,
    ) -> Result<(), SendFailure> {
        let kind = match attachment {
            Outbound::None => wire::Attachment::None,
            Outbound::Memory(_) => wire::Attachment::Memory,
            Outbound::Endpoint(_) => wire::Attachment::Endpoint,
            Outbound::Notification(_) => wire::Attachment::Notification,
        };
        let (frame, n) = wire::encode(id, bytes, kind).ok_or(SendFailure {
            error: Error(abi::INVALID_ARGUMENT as i64),
            exposed: false,
        })?;
        self.mailbox.send_ready().map_err(|error| SendFailure {
            error,
            exposed: false,
        })?;
        let grant = match attachment {
            Outbound::None => Ok(()),
            Outbound::Memory(object) => self.lease.0.0.send_memory(&id.to_le_bytes(), object),
            Outbound::Endpoint(endpoint) => self.lease.0.0.send_with(&id.to_le_bytes(), endpoint),
            Outbound::Notification(notification) => {
                notification.delegate(self.lease.0.0, &id.to_le_bytes())
            }
        };
        grant.map_err(|error| SendFailure {
            error,
            exposed: false,
        })?;
        self.mailbox
            .try_send(&frame[..n])
            .map_err(|error| SendFailure {
                error,
                exposed: kind != wire::Attachment::None,
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
