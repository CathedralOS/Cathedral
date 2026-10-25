//! Open child control while its upstream acquisition still needs our service loop.
//! A failed link makes loss unknown; it never grants permission to kill the child.
use super::*;
use cathedral_user_runtime::connection::{Client, Connecting};

pub enum Control {
    Opening(Connecting<1>),
    Ready(Client),
    Failed,
}
impl Control {
    pub fn start(pair: Pair) -> Self {
        // begin_connect owns the pair on failure as well as success.
        let until = match time::after(300) {
            Ok(until) => until,
            Err(_) => {
                pair.0.close().ok();
                pair.1.close().ok();
                return Self::Failed;
            }
        };
        match Client::begin_connect(pair, until) {
            Ok(opening) => Self::Opening(opening),
            Err(_) => Self::Failed,
        }
    }
    pub fn progress(&mut self) {
        if let Self::Opening(opening) = self {
            match opening.poll() {
                Ok(Some(client)) => {
                    *self = Self::Ready(client);
                    cathedral_user_runtime::write(b"Cathedral Split: shared child control ready\n")
                        .unwrap();
                }
                Ok(None) => (),
                Err(_) => *self = Self::Failed,
            }
        }
    }
    pub fn wait(&self, pending: bool) -> ([Option<Interest>; 2], Option<u64>) {
        match self {
            Self::Opening(opening) => ([opening.interest().ok(), None], Some(opening.deadline())),
            Self::Ready(client) if pending => (
                client.interests().map_or([None; 2], |i| i.map(Some)),
                client.transfer_deadline(),
            ),
            _ => ([None; 2], None),
        }
    }
    pub fn ask(&mut self) -> Result<(), Error> {
        match self {
            Self::Ready(client) => {
                client.begin(&wire::encode([wire::LOSS, 0, 0, 0, 0, 0, 0, 0]), None)
            }
            _ => Err(Error(abi::BUSY as i64)),
        }
    }
    pub fn answer(&mut self) -> Result<Option<bool>, Error> {
        match self {
            Self::Ready(client) => client.poll().map(|r| {
                r.map(|(bytes, len, object)| {
                    object.is_none() && wire::decode(&bytes[..len]) == Some([0; 8])
                })
            }),
            _ => Err(Error(abi::PEER_CLOSED as i64)),
        }
    }
}
