//! Client setup advances one incoming handoff per poll. The caller may serve
//! unrelated requests between steps, including requests from this future peer.
use super::*;

pub struct Connecting<const WIDTH: usize> {
    opening: Option<Opening>,
    deadline: u64,
}
struct Opening {
    lease: Lease,
    writer: Option<Writer>,
    bell: Option<Bell>,
    reader: Option<Reader>,
    signal: Option<Signal>,
    phase: Phase,
}
enum Phase {
    Open,
    Writer,
    Reader,
    Signal,
    Commit,
}
impl<const WIDTH: usize> Connecting<WIDTH> {
    pub(super) fn start(pair: Pair, deadline: u64) -> Result<Self, Error> {
        let lease = Lease(pair);
        if WIDTH == 0 || WIDTH > wire::MAX_IN_FLIGHT {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        if time::reached(time::now()?, deadline) {
            return Err(Error(abi::TIMED_OUT as i64));
        }
        let writer = Writer::allocate(1)?;
        let bell = Bell::new()?;
        pair.0.send(if WIDTH > 1 {
            wire::OPEN_PIPELINE
        } else {
            wire::OPEN
        })?;
        Ok(Self {
            opening: Some(Opening {
                lease,
                writer: Some(writer),
                bell: Some(bell),
                reader: None,
                signal: None,
                phase: Phase::Open,
            }),
            deadline,
        })
    }
    pub fn deadline(&self) -> u64 {
        self.deadline
    }
    pub fn interest(&self) -> Result<Interest, Error> {
        self.opening
            .as_ref()
            .map(|o| Interest::Ipc(o.lease.0.1))
            .ok_or(Error(abi::PEER_CLOSED as i64))
    }
    /// None means setup still needs progress. Completion transfers custody once;
    /// any error retires it immediately, even if the caller retains this object.
    /// Dropping an unfinished connector releases local custody. Published grants
    /// keep their recipient lifetime; abandonment cannot recall exposed backing.
    pub fn poll(&mut self) -> Result<Option<Pipeline<WIDTH>>, Error> {
        let mut opening = self.opening.take().ok_or(Error(abi::PEER_CLOSED as i64))?;
        let now = time::now()?;
        if time::reached(now, self.deadline) {
            return Err(Error(abi::TIMED_OUT as i64));
        }
        match ready(opening.lease.0.1, now) {
            Err(Error(c)) if c == abi::TIMED_OUT as i64 => (),
            Err(error) => return Err(error),
            Ok(()) => {
                if opening.step(now)? {
                    let mailbox = Mailbox::new(
                        opening.writer.take().unwrap(),
                        opening.reader.take().unwrap(),
                    )?;
                    return Ok(Some(Pipeline::from_peer(Peer {
                        lease: opening.lease,
                        mailbox,
                        bell: opening.bell.take().unwrap(),
                        signal: opening.signal.take().unwrap(),
                        incoming: Incoming::default(),
                        pipelined: WIDTH > 1,
                    })));
                }
            }
        }
        self.opening = Some(opening);
        Ok(None)
    }
}
impl Opening {
    fn step(&mut self, now: u64) -> Result<bool, Error> {
        let pair = self.lease.0;
        self.phase = match self.phase {
            Phase::Open => {
                ack(pair.1, now)?;
                self.writer.as_ref().unwrap().delegate(pair.0)?;
                Phase::Writer
            }
            Phase::Writer => {
                ack(pair.1, now)?;
                self.bell.as_mut().unwrap().delegate(pair.0)?;
                Phase::Reader
            }
            Phase::Reader => {
                self.reader = Some(Reader::accept(pair.1)?);
                pair.0.send(wire::ACK)?;
                Phase::Signal
            }
            Phase::Signal => {
                let (_, signal) = N::accept(pair.1, &mut [0; 64])?;
                self.signal = Some(Signal(signal.ok_or(Error(abi::IO_ERROR as i64))?));
                self.signal.as_ref().unwrap().0.signal(false)?;
                pair.0.send(wire::COMMIT)?;
                Phase::Commit
            }
            Phase::Commit => {
                ack(pair.1, now)?;
                return Ok(true);
            }
        };
        Ok(false)
    }
}
