//! Nonblocking accept: one grant/ack step per dispatch, with bounded setup lifetime.
use super::*;
pub(crate) struct Accept {
    lease: Option<Lease>,
    writer: Option<Writer>,
    reader: Option<Reader>,
    bell: Option<Bell>,
    signal: Option<Signal>,
    phase: u8,
    pipelined: bool,
    pub deadline: u64,
}
impl Accept {
    // Called only after OPEN was read from the explicitly enabled endpoint.
    pub(crate) fn start(pair: Pair, deadline: u64, pipelined: bool) -> Result<Self, Error> {
        let lease = Lease(pair);
        let writer = Writer::allocate(1)?;
        let bell = Bell::new()?;
        pair.0.send(wire::ACK)?;
        Ok(Self {
            lease: Some(lease),
            writer: Some(writer),
            reader: None,
            bell: Some(bell),
            signal: None,
            phase: 0,
            pipelined,
            deadline,
        })
    }
    pub(crate) fn step(&mut self) -> Result<Option<Peer>, Error> {
        let pair = self.lease.as_ref().unwrap().0;
        let now = time::now()?;
        if time::reached(now, self.deadline) {
            return Err(Error(abi::TIMED_OUT as i64));
        }
        match ready(pair.1, now) {
            Err(Error(c)) if c == abi::TIMED_OUT as i64 => return Ok(None),
            result => result?,
        }
        match self.phase {
            0 => {
                self.reader = Some(Reader::accept(pair.1)?);
                pair.0.send(wire::ACK)?;
            }
            1 => {
                let (_, signal) = N::accept(pair.1, &mut [0; 64])?;
                self.signal = Some(Signal(signal.ok_or(Error(abi::IO_ERROR as i64))?));
                self.signal.as_ref().unwrap().0.signal(false)?;
                self.writer.as_ref().unwrap().delegate(pair.0)?;
            }
            2 => {
                ack(pair.1, now)?;
                self.bell.as_mut().unwrap().delegate(pair.0)?;
            }
            _ => {
                let mut bytes = [0; 64];
                let n = pair.1.receive_until(&mut bytes, now)?;
                if &bytes[..n] != wire::COMMIT {
                    return Err(Error(abi::IO_ERROR as i64));
                }
                let mailbox =
                    Mailbox::new(self.writer.take().unwrap(), self.reader.take().unwrap())?;
                pair.0.send(wire::ACK)?;
                return Ok(Some(Peer {
                    lease: self.lease.take().unwrap(),
                    mailbox,
                    bell: self.bell.take().unwrap(),
                    signal: self.signal.take().unwrap(),
                    incoming: Incoming::default(),
                    pipelined: self.pipelined,
                }));
            }
        }
        self.phase += 1;
        Ok(None)
    }
}
