//! Transport custody below the lifecycle decisions. One shared request survives
//! expiry until its exact reply is drained; repeated asks cannot queue work.
use super::*;
use cathedral_user_runtime::connection::{Client, Connecting};

enum Link {
    Opening(Connecting<1>),
    Shared(Client),
    Failed,
}
pub struct Control {
    link: Link,
    queued: Option<wire::Packet>,
    late: bool,
}
impl Control {
    /// Fault fixture: abandon only the established control transport. The child
    /// task and its separately granted service connections remain owned and live.
    #[cfg(feature = "control-failure-probe")]
    pub fn cut_ready(&mut self) -> bool {
        if matches!(self.link, Link::Shared(_)) {
            self.link = Link::Failed;
            true
        } else {
            false
        }
    }
    pub fn open(pair: Pair) -> Self {
        let link = match time::after(300) {
            Ok(until) => Client::begin_connect(pair, until).map_or_else(
                |error| {
                    time::storage_sample("matrix-control-begin-refused", &[error.0 as u64]);
                    Link::Failed
                },
                Link::Opening,
            ),
            Err(_) => {
                pair.0.close().ok();
                pair.1.close().ok();
                Link::Failed
            }
        };
        Self {
            link,
            queued: None,
            late: false,
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
                Err(error) => {
                    time::storage_sample("matrix-control-open-refused", &[error.0 as u64]);
                    self.link = Link::Failed;
                }
            }
        }
    }
    pub fn ask(&mut self, words: wire::Packet) -> Result<(), Error> {
        if self.queued.is_some() {
            return Err(Error(abi::BUSY as i64));
        }
        match &mut self.link {
            Link::Opening(_) => {
                self.queued = Some(words);
                Ok(())
            }
            Link::Shared(client) if !self.late => client.begin(&wire::encode(words), None),
            Link::Shared(_) => Err(Error(abi::TIMED_OUT as i64)),
            Link::Failed => Err(Error(abi::PEER_CLOSED as i64)),
        }
    }
    pub fn has_late(&self) -> bool {
        self.late
    }
    pub fn wait(&self, answers: bool) -> ([Option<Interest>; 2], Option<u64>) {
        match &self.link {
            Link::Opening(opening) => ([opening.interest().ok(), None], Some(opening.deadline())),
            Link::Shared(client) if answers && client.pending() => (
                client.interests().map_or([None; 2], |i| i.map(Some)),
                client.transfer_deadline(),
            ),
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
            Link::Failed => return Err(Error(abi::PEER_CLOSED as i64)),
        };
        Ok(received.and_then(|(bytes, n)| {
            if self.late {
                #[cfg(feature = "trace-storage")]
                if let Some(words) = wire::decode(&bytes[..n]) {
                    time::storage_sample("matrix-control-late-drained", &words[..2]);
                }
                self.late = false;
                None
            } else {
                Some(wire::decode(&bytes[..n]).unwrap_or(wire::error(abi::IO_ERROR)))
            }
        }))
    }
    pub fn expire(&mut self) {
        if self.queued.take().is_none() {
            self.late = true;
        }
    }
    pub fn drain(&mut self) {
        if self.late && self.poll().is_err() {
            self.late = false;
            self.link = Link::Failed;
        }
    }
}
