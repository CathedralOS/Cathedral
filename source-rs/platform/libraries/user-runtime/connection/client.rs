//! Bounded outstanding calls, exact correlation and retirement after ambiguous failure.
use super::*;
pub type Response = ([u8; 64], usize, Option<Object>);
pub type Client = Pipeline<1>;
/// Explicitly bounded independent calls over one connection. Poll replies by
/// transport ID; application payloads do not select the recipient of a reply.
pub struct Pipeline<const WIDTH: usize> {
    peer: Option<Peer>,
    next: u64,
    pending: [u64; WIDTH],
}
impl<const WIDTH: usize> Pipeline<WIDTH> {
    /// Takes ownership of the supplied send/receive pair, including on failure.
    pub fn connect(pair: Pair, deadline: u64) -> Result<Self, Error> {
        let lease = Lease(pair);
        if WIDTH == 0 || WIDTH > wire::MAX_IN_FLIGHT {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        let writer = Writer::allocate(1)?;
        let mut bell = Bell::new()?;
        pair.0.send(if WIDTH > 1 {
            wire::OPEN_PIPELINE
        } else {
            wire::OPEN
        })?;
        ack(pair.1, deadline)?;
        writer.delegate(pair.0)?;
        ack(pair.1, deadline)?;
        bell.delegate(pair.0)?;
        ready(pair.1, deadline)?;
        let reader = Reader::accept(pair.1)?;
        pair.0.send(wire::ACK)?;
        ready(pair.1, deadline)?;
        let (_, signal) = N::accept(pair.1, &mut [0; 64])?;
        let signal = Signal(signal.ok_or(Error(abi::IO_ERROR as i64))?);
        signal.0.signal(false)?; // Validate installed signal authority; harmless early hint.
        pair.0.send(wire::COMMIT)?;
        ack(pair.1, deadline)?;
        Ok(Self {
            peer: Some(Peer {
                lease,
                mailbox: Mailbox::new(writer, reader)?,
                bell,
                signal,
                incoming: Incoming::default(),
                pipelined: WIDTH > 1,
            }),
            next: 1,
            pending: [0; WIDTH],
        })
    }
    /// Any error after submission retires this connection; caller must reconcile
    /// possible service effects. A deadline never authorizes mutation replay.
    pub fn call(&mut self, bytes: &[u8], deadline: u64) -> Result<([u8; 64], usize), Error> {
        self.plain(bytes, Some(deadline))
    }
    /// Retain the same request until a reply or terminal transport failure.
    /// Slow service work does not authorize a retry or disconnect.
    pub fn call_retained(&mut self, bytes: &[u8]) -> Result<([u8; 64], usize), Error> {
        self.plain(bytes, None)
    }
    /// Both directions may carry one immutable object. A successful reply owns
    /// its installed read grant; dropping it releases that custody.
    pub fn call_memory(
        &mut self,
        bytes: &[u8],
        object: Option<&Object>,
        deadline: u64,
    ) -> Result<([u8; 64], usize, Option<Object>), Error> {
        self.exchange(bytes, object, Some(deadline))
    }
    fn plain(&mut self, bytes: &[u8], deadline: Option<u64>) -> Result<([u8; 64], usize), Error> {
        let (bytes, n, object) = self.exchange(bytes, None, deadline)?;
        if object.is_some() {
            self.retire();
            return Err(Error(abi::IO_ERROR as i64));
        }
        Ok((bytes, n))
    }
    /// Publish one request without waiting for the service. BUSY leaves the
    /// original request intact; a transport failure retires without replay.
    pub fn begin(&mut self, bytes: &[u8], object: Option<&Object>) -> Result<(), Error> {
        if self.pending() {
            return Err(Error(abi::BUSY as i64));
        }
        self.submit(bytes, object).map(|_| ())
    }
    /// Submit independently of older application replies. BUSY means the local
    /// in-flight window is full; WOULD_BLOCK means the peer has not consumed the
    /// previous frame. Neither publishes, consumes an ID, nor retires prior work.
    pub fn submit(&mut self, bytes: &[u8], object: Option<&Object>) -> Result<u64, Error> {
        if bytes.len() > 64 {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        let slot = self
            .pending
            .iter()
            .position(|id| *id == 0)
            .ok_or(Error(abi::BUSY as i64))?;
        let id = self.next;
        let Some(next) = id.checked_add(1) else {
            self.retire();
            return Err(Error(abi::IO_ERROR as i64));
        };
        let peer = self.peer.as_mut().ok_or(Error(abi::PEER_CLOSED as i64))?;
        match peer.send(id, bytes, object) {
            Ok(()) => {
                self.next = next;
                self.pending[slot] = id;
                Ok(id)
            }
            Err(failure) => {
                if failure.exposed || failure.error.0 != abi::WOULD_BLOCK as i64 {
                    self.retire();
                }
                Err(failure.error)
            }
        }
    }
    /// Legacy single-call convenience. Multiple outstanding calls require poll_reply.
    pub fn poll(&mut self) -> Result<Option<Response>, Error> {
        if self.pending.iter().filter(|id| **id != 0).count() != 1 {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        self.poll_reply()
            .map(|reply| reply.map(|(_, response)| response))
    }
    /// Replies may complete out of order, but must match an outstanding ID once.
    pub fn poll_reply(&mut self) -> Result<Option<(u64, Response)>, Error> {
        let Some(reply) = self.poll_message()? else {
            return Ok(None);
        };
        if reply.endpoint.is_some() || reply.notification.is_some() {
            self.retire();
            return Err(Error(abi::IO_ERROR as i64));
        }
        Ok(Some((reply.id, (reply.bytes, reply.len, reply.object))))
    }
    fn poll_message(&mut self) -> Result<Option<Message>, Error> {
        if !self.pending() {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        let result = (|| {
            let peer = self.peer.as_mut().ok_or(Error(abi::PEER_CLOSED as i64))?;
            let received = peer.receive()?;
            peer.incoming.validate_replies(&self.pending)?;
            let Some(reply) = received else {
                return Ok(None);
            };
            let slot = self
                .pending
                .iter()
                .position(|id| *id == reply.id)
                .ok_or(Error(abi::IO_ERROR as i64))?;
            self.pending[slot] = 0;
            Ok(Some(reply))
        })();
        if result.is_err() {
            self.retire();
        }
        result
    }
    /// Include both interests in the caller's service loop. Readiness consumes
    /// nothing; poll remains responsible for correlation and completion.
    pub fn interests(&self) -> Result<[Interest; 2], Error> {
        let peer = self.peer.as_ref().ok_or(Error(abi::PEER_CLOSED as i64))?;
        Ok([
            Interest::Ipc(peer.lease.0.1),
            Interest::Notification(peer.wake()),
        ])
    }
    pub fn pending(&self) -> bool {
        self.pending.iter().any(|id| *id != 0)
    }
    pub fn transfer_deadline(&self) -> Option<u64> {
        self.peer.as_ref().and_then(Peer::deadline)
    }
    pub fn call_retained_memory(
        &mut self,
        bytes: &[u8],
        object: Option<&Object>,
    ) -> Result<Response, Error> {
        self.exchange(bytes, object, None)
    }
    fn exchange(
        &mut self,
        bytes: &[u8],
        object: Option<&Object>,
        deadline: Option<u64>,
    ) -> Result<Response, Error> {
        let reply = self.exchange_message(bytes, object, deadline)?;
        if reply.endpoint.is_some() || reply.notification.is_some() {
            self.retire();
            return Err(Error(abi::IO_ERROR as i64));
        }
        Ok((reply.bytes, reply.len, reply.object))
    }
    /// Accept endpoint custody explicitly. Application payload stays in shared
    /// backing; an unclaimed or rejected endpoint is released by its guard.
    pub fn call_retained_endpoint(
        &mut self,
        bytes: &[u8],
    ) -> Result<([u8; 64], usize, Option<Endpoint>), Error> {
        let reply = self.exchange_message(bytes, None, None)?;
        if reply.object.is_some() || reply.notification.is_some() {
            self.retire();
            return Err(Error(abi::IO_ERROR as i64));
        }
        Ok((reply.bytes, reply.len, reply.endpoint))
    }
    /// Accept only notification custody (or a plain refusal) on this call.
    /// The provider retains its source right; the received guard owns an
    /// independently installed grant with the same authority and object lifetime.
    pub fn call_retained_notification(
        &mut self,
        bytes: &[u8],
    ) -> Result<([u8; 64], usize, Option<NotificationGrant>), Error> {
        let reply = self.exchange_message(bytes, None, None)?;
        if reply.object.is_some() || reply.endpoint.is_some() {
            self.retire();
            return Err(Error(abi::IO_ERROR as i64));
        }
        Ok((reply.bytes, reply.len, reply.notification))
    }
    fn exchange_message(
        &mut self,
        bytes: &[u8],
        object: Option<&Object>,
        deadline: Option<u64>,
    ) -> Result<Message, Error> {
        self.begin(bytes, object)?;
        let result = (|| {
            let mut wake_error = None;
            let diagnostic = time::after(100)?;
            let mut delayed = false;
            loop {
                if let Some(reply) = self.poll_message()? {
                    if delayed {
                        time::storage_sample("shared-call-resumed", &[reply.id]);
                    }
                    return Ok(reply);
                }
                if let Some(error) = wake_error {
                    return Err(error);
                }
                let now = time::now()?;
                if deadline.is_none() && !delayed && time::reached(now, diagnostic) {
                    time::storage_sample("shared-call-delayed", &[self.next - 1]);
                    delayed = true;
                }
                if deadline.is_some_and(|d| time::reached(now, d)) {
                    return Err(Error(abi::TIMED_OUT as i64));
                }
                let until = match deadline {
                    Some(d) => d,
                    None => time::after(100)?,
                };
                wake_error = match self.peer.as_ref().unwrap().wait(until) {
                    Err(Error(c)) if deadline.is_none() && c == abi::TIMED_OUT as i64 => None,
                    result => result.err(),
                };
            }
        })();
        if result.is_err() {
            self.retire();
        }
        result
    }
    pub fn retire(&mut self) {
        self.peer = None;
        self.pending.fill(0);
    }
}
