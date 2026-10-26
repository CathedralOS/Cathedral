//! Transport custody below the lifecycle decisions. One shared request survives
//! expiry until its exact reply is drained; fixture queues retain FIFO accounting.
use super::*;
use cathedral_user_runtime::connection::{Client, Connecting};

enum Link {
    Copied(Pair),
    Opening(Connecting<1>),
    Shared(Client),
    Failed,
}
pub struct Control {
    link: Link,
    queued: Option<wire::Packet>,
    late: u8,
}
impl Control {
    pub fn open(pair: Pair, shared: bool) -> Self {
        let link = if shared {
            match time::after(300) {
                Ok(until) => Client::begin_connect(pair, until).map_or(Link::Failed, Link::Opening),
                Err(_) => {
                    pair.0.close().ok();
                    pair.1.close().ok();
                    Link::Failed
                }
            }
        } else {
            Link::Copied(pair)
        };
        Self {
            link,
            queued: None,
            late: 0,
        }
    }
    pub fn progress(&mut self) {
        if let Link::Opening(opening) = &mut self.link {
            match opening.poll() {
                Ok(Some(client)) => {
                    self.link = Link::Shared(client);
                    cathedral_user_runtime::write(
                        b"Cathedral matrix: shared child control ready\n",
                    )
                    .unwrap();
                }
                Ok(None) => (),
                Err(_) => self.link = Link::Failed,
            }
        }
    }
    pub fn ask(&mut self, words: wire::Packet) -> Result<(), Error> {
        if self.queued.is_some() {
            return Err(Error(abi::BUSY as i64));
        }
        match &mut self.link {
            Link::Copied(pair) if self.late < 8 => pair.0.send(&wire::encode(words)),
            Link::Copied(_) => Err(Error(abi::TIMED_OUT as i64)),
            Link::Opening(_) => {
                self.queued = Some(words);
                Ok(())
            }
            Link::Shared(client) if self.late == 0 => client.begin(&wire::encode(words), None),
            Link::Shared(_) => Err(Error(abi::TIMED_OUT as i64)),
            Link::Failed => Err(Error(abi::PEER_CLOSED as i64)),
        }
    }
    pub fn has_late(&self) -> bool {
        self.late != 0
    }
    pub fn wait(&self, answers: bool) -> ([Option<Interest>; 2], Option<u64>) {
        match &self.link {
            Link::Opening(opening) => ([opening.interest().ok(), None], Some(opening.deadline())),
            Link::Shared(client) if answers && client.pending() => (
                client.interests().map_or([None; 2], |i| i.map(Some)),
                client.transfer_deadline(),
            ),
            Link::Copied(pair) if answers => ([Some(Interest::Ipc(pair.1)), None], None),
            _ => ([None; 2], None),
        }
    }
    pub fn receive(&mut self, now: u64, until: u64) -> Result<Option<wire::Packet>, Error> {
        if self.queued.is_some() {
            if time::reached(now, until) {
                return Ok(None);
            }
            if let Link::Shared(client) = &mut self.link {
                client.begin(&wire::encode(self.queued.take().unwrap()), None)?;
            }
        }
        self.poll()
    }
    fn poll(&mut self) -> Result<Option<wire::Packet>, Error> {
        let received = match &mut self.link {
            Link::Opening(_) => return Ok(None),
            Link::Shared(client) if !client.pending() => return Ok(None),
            Link::Shared(client) => match client.poll()? {
                Some((bytes, n, object)) if object.is_none() => Some((bytes, n)),
                Some(_) => {
                    client.retire();
                    return Err(Error(abi::IO_ERROR as i64));
                }
                None => None,
            },
            Link::Copied(pair) => {
                let mut bytes = [0; 64];
                match pair.1.receive_until(&mut bytes, time::now()?) {
                    Ok(n) => Some((bytes, n)),
                    Err(Error(c)) if c == abi::TIMED_OUT as i64 => None,
                    Err(error) => return Err(error),
                }
            }
            Link::Failed => return Err(Error(abi::PEER_CLOSED as i64)),
        };
        Ok(received.and_then(|(bytes, n)| {
            if self.late != 0 {
                #[cfg(feature = "trace-storage")]
                if let Some(words) = wire::decode(&bytes[..n]) {
                    time::storage_sample("matrix-control-late-drained", &words[..2]);
                }
                self.late -= 1;
                None
            } else {
                Some(wire::decode(&bytes[..n]).unwrap_or(wire::error(abi::IO_ERROR)))
            }
        }))
    }
    pub fn expire(&mut self) {
        if self.queued.take().is_none() {
            self.late += 1;
        }
    }
    pub fn drain(&mut self) {
        if self.late != 0 && self.poll().is_err() {
            self.late = 0;
            self.link = Link::Failed;
        }
    }
}
