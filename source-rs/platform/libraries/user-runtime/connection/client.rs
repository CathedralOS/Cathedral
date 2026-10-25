//! One outstanding call, exact correlation, retirement after ambiguous failure.
use super::*;
pub type Response = ([u8; 64], usize, Option<Object>);
pub struct Client {
    peer: Option<Peer>,
    next: u64,
    pending: Option<u64>,
}
impl Client {
    /// Takes ownership of the supplied send/receive pair, including on failure.
    pub fn connect(pair: Pair, deadline: u64) -> Result<Self, Error> {
        let lease = Lease(pair);
        let writer = Writer::allocate(1)?;
        let mut bell = Bell::new()?;
        pair.0.send(wire::OPEN)?;
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
            }),
            next: 1,
            pending: None,
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
        if self.pending.is_some() {
            return Err(Error(abi::BUSY as i64));
        }
        if bytes.len() > 64 {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        let id = self.next;
        let result = (|| {
            self.next = id.checked_add(1).ok_or(Error(abi::IO_ERROR as i64))?;
            self.peer
                .as_mut()
                .ok_or(Error(abi::PEER_CLOSED as i64))?
                .send(id, bytes, object)
                .map_err(|e| e.error)?;
            self.pending = Some(id);
            Ok(())
        })();
        if result.is_err() {
            self.retire();
        }
        result
    }
    /// Consume only the exact outstanding reply. None leaves its custody intact.
    pub fn poll(&mut self) -> Result<Option<Response>, Error> {
        let id = self.pending.ok_or(Error(abi::INVALID_ARGUMENT as i64))?;
        let result = (|| {
            let peer = self.peer.as_mut().ok_or(Error(abi::PEER_CLOSED as i64))?;
            let Some(reply) = peer.receive()? else {
                return Ok(None);
            };
            if reply.id != id {
                return Err(Error(abi::IO_ERROR as i64));
            }
            self.pending = None;
            Ok(Some((reply.bytes, reply.len, reply.object)))
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
        self.pending.is_some()
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
        self.begin(bytes, object)?;
        let result = (|| {
            let mut wake_error = None;
            loop {
                if let Some(reply) = self.poll()? {
                    return Ok(reply);
                }
                if let Some(error) = wake_error {
                    return Err(error);
                }
                let now = time::now()?;
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
        self.pending = None;
    }
}
