// Fair, bounded service control and explicitly admitted client connections.
// Client identity comes from the installed connection, never from message contents.
// owner.rs retains one lifecycle request for apps with no service-client slots.
use crate::{Error, abi, ipc::Handle, link::Link};
#[derive(Clone, Copy)]
pub struct Request {
    pub bytes: [u8; 64],
    pub len: usize,
    pub control: bool,
    pub client: Option<usize>,
    pub incarnation: u64,
    reply: Handle,
    shared_id: u64,
}
pub enum Event {
    Request(Request),
    Memory(Request, crate::memory::Object),
    Disconnected { client: usize, incarnation: u64 },
}
const MAX_CLIENTS: usize = abi::MAX_WAIT_HANDLES - 1; // Control also occupies a wait slot.
pub struct Server {
    control: (Handle, Handle),
    links: [Option<Link>; MAX_CLIENTS],
    clients: [Option<(Handle, Handle)>; MAX_CLIENTS],
    count: usize,
    cursor: usize,
    memory: bool,
    closed: [Option<u64>; MAX_CLIENTS],
    paused: u32,
    replies: Option<&'static mut Replies>,
    shared: [Option<&'static mut SharedSlot>; abi::MAX_WAIT_HANDLES],
}
impl Server {
    fn control_only(&self) -> bool {
        if self.shared.iter().any(|s| s.is_some()) || self.reply_paused() != 0 {
            return false;
        }
        for index in 0..self.count {
            if self.links[index].is_some()
                || self.clients[index].is_some()
                || self.closed[index].is_some()
            {
                return false;
            }
        }
        true
    }
    /// Apply backpressure while the service holds one deferred event per stream.
    /// Bit zero is control; bit n+1 is local client n. Resuming preserves FIFO.
    /// Keep a stream unpaused or supply an extra wake channel to next_event_or.
    pub fn pause(&mut self, streams: u32) {
        self.paused = streams;
    }
    /// Explicit opt-in: services must handle and release typed memory attachments.
    pub fn enable_memory(&mut self) {
        self.memory = true;
    }
    pub fn next_request(&mut self) -> Result<Request, Error> {
        loop {
            if let Event::Request(request) = self.next_event()? {
                return Ok(request);
            }
        }
    }
    /// Services retaining client-owned state must observe disconnects before reuse.
    pub fn next_event(&mut self) -> Result<Event, Error> {
        loop {
            if let Some(event) = self.next_event_for(&[], None)? {
                return Ok(event);
            }
        }
    }
    /// Also wake for caller-owned reply channels or a maintenance deadline.
    /// None asks the caller to inspect those channels; no message is consumed.
    pub fn next_event_or(
        &mut self,
        extra: &[Handle],
        deadline: Option<u64>,
    ) -> Result<Option<Event>, Error> {
        if extra.len() > abi::MAX_WAIT_HANDLES {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        let mut interests = [crate::event::Interest::Ipc(self.control.0); abi::MAX_WAIT_HANDLES];
        for (out, &handle) in interests.iter_mut().zip(extra) {
            *out = crate::event::Interest::Ipc(handle);
        }
        self.next_event_for(&interests[..extra.len()], deadline)
    }
    /// Caller-owned IPC/device/notification interests share one bounded wait.
    pub fn next_event_for(
        &mut self,
        extra: &[crate::event::Interest],
        deadline: Option<u64>,
    ) -> Result<Option<Event>, Error> {
        // A control-only service needs no clock grant (for example the display fixture).
        if extra.is_empty() && self.paused == 0 && deadline.is_none() && self.control_only() {
            let mut bytes = [0; 64];
            let (len, object) = if self.memory {
                self.control.0.receive_memory(&mut bytes)?
            } else {
                (self.control.0.receive(&mut bytes)?, None)
            };
            let request = Request {
                bytes,
                len,
                control: true,
                client: None,
                incarnation: self.control.0.raw(),
                reply: self.control.1,
                shared_id: 0,
            };
            return Ok(Some(match object {
                Some(object) => Event::Memory(request, object),
                None => Event::Request(request),
            }));
        }
        loop {
            self.flush_replies();
            self.expire_shared()?;
            if let Some(event) = self.take_disconnection() {
                return Ok(Some(event));
            }
            self.connect()?;
            if let Some(event) = self.poll()? {
                return Ok(Some(event));
            }
            if self.wait(extra, deadline)? {
                return Ok(None);
            }
        }
    }
    pub fn reply(&mut self, request: &Request, bytes: &[u8]) -> Result<(), Error> {
        if request.shared_id != 0 {
            return self.shared_reply(request, bytes, None);
        }
        if !request.control && self.replies.is_some() {
            return self.retained_reply(request, bytes);
        }
        match request.reply.send(bytes) {
            // A full client response queue loses only that reply. The service and other
            // client stay live; synchronous clients reconcile ambiguous commits.
            Err(Error(code)) if !request.control && code == abi::WOULD_BLOCK as i64 => Ok(()),
            Err(_) if !request.control => {
                self.disconnect(request.client.unwrap());
                Ok(())
            }
            result => result,
        }
    }
    /// Typed reply. Failure leaves the source object owned by the caller; the
    /// provider can send a small refusal if receiver funding prevents attachment.
    pub fn reply_memory(
        &mut self,
        request: &Request,
        bytes: &[u8],
        object: &crate::memory::Object,
    ) -> Result<(), Error> {
        if request.shared_id != 0 {
            return self.shared_reply(request, bytes, Some(object));
        }
        request.reply.send_memory(bytes, object)
    }
}

impl Server {
    /// Move a retained endpoint in a reply. Err preserves the caller's right.
    /// Ok consumes it, including shared failure after the grant was published;
    /// this is transfer custody, not proof that the receiver used the endpoint.
    pub fn reply_endpoint(
        &mut self,
        request: &Request,
        bytes: &[u8],
        endpoint: Handle,
    ) -> Result<(), Error> {
        if request.shared_id != 0 {
            return self.shared_endpoint(request, bytes, endpoint);
        }
        request.reply.send_with(bytes, endpoint)
    }
}

impl Server {
    /// Delegate a notification right. Err publishes no new grant; Ok may mean
    /// committed escrow followed by transport retirement. The source right stays
    /// owned by the caller in either case; never replay an exposed delegation.
    pub fn reply_notification(
        &mut self,
        request: &Request,
        bytes: &[u8],
        notification: crate::notification::Notification,
    ) -> Result<(), Error> {
        if request.shared_id != 0 {
            return self.shared_notification(request, bytes, notification);
        }
        notification.delegate(request.reply, bytes)
    }
}
