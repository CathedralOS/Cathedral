//! Root adapter supplies a grant; nested instances obtain the same shape via IPC.
//! A lost/invalid upstream grant empties the world before any child can query it.
use super::*;
use cathedral_user_runtime::{ipc::Handle, task::Environment, time};
mod rendering;
pub(super) struct Parent {
    pub upstream: bool,
    pair: Option<(Handle, Handle)>,
    pub grant: Grant,
    display: Option<(Handle, Handle)>,
    pub wake: Option<Handle>,
}
impl Parent {
    pub fn refresh_notified(&mut self, world: &mut World) -> Result<(), Error> {
        if let Some(wake) = self.wake {
            match wake.receive_until(&mut [0; 64], time::now()?) {
                Ok(_) => self.refresh(world)?,
                Err(Error(code)) if code == abi::TIMED_OUT as i64 => (),
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }
    pub fn new(environment: &Environment) -> Result<Self, Error> {
        if !matches!(environment.len(), 0 | 2 | 3 | 4) {
            return Err(Error(abi::DENIED as i64));
        }
        Ok(Self {
            upstream: matches!(environment.len(), 2 | 3),
            pair: matches!(environment.len(), 2 | 3).then(|| {
                (
                    environment.endpoint(0).unwrap(),
                    environment.endpoint(1).unwrap(),
                )
            }),
            display: (environment.len() == 4).then(|| {
                (
                    environment.endpoint(0).unwrap(),
                    environment.endpoint(1).unwrap(),
                )
            }),
            grant: Grant::default(),
            wake: (environment.len() == 3).then(|| environment.endpoint(2).unwrap()),
        })
    }
    pub fn refresh(&mut self, world: &mut World) -> Result<(), Error> {
        if !self.upstream {
            world.inherit(self.grant);
            return Ok(());
        }
        if self.pair.is_none() {
            world.inherit(Grant::default());
            return Ok(());
        }
        // Each round trip returns a consistent local grant and event. MORE avoids
        // spending another scheduling turn merely to discover an empty queue.
        for _ in 0..8 {
            let reply = match self.call([wire::POLL, 0, 0, 0, 0, 0, 0, 0]) {
                Ok(reply) => reply,
                Err(Error(code)) if code == abi::PEER_CLOSED as i64 => {
                    self.pair = None;
                    world.inherit(Grant::default());
                    return Ok(());
                }
                Err(error) => return Err(error),
            };
            self.grant = if reply[0] == 0 {
                Grant {
                    epoch: reply[1],
                    width: reply[2],
                    height: reply[3],
                    keys: reply[4],
                }
            } else {
                Grant::default()
            };
            world.inherit(self.grant);
            if !self.grant.valid() {
                return Ok(());
            }
            let event = wire::poll_event(reply);
            match event[2] {
                wire::NONE => (),
                wire::POINTER => world.pointer(event[3], event[4], event[5]),
                wire::KEY => {
                    #[cfg(feature = "trace-input")]
                    time::trace("matrix-key");
                    world.key(event[3], event[4]);
                }
                wire::CANCEL => world.cancel(),
                _ => {
                    world.inherit(Grant::default());
                    break;
                }
            }
            if reply[5] & wire::MORE == 0 {
                break;
            }
        }
        Ok(())
    }
    fn call(&self, words: wire::Packet) -> Result<wire::Packet, Error> {
        let (send, receive) = self.pair.unwrap();
        send.send(&wire::encode(words))?;
        let mut bytes = [0; 64];
        let len = receive.receive_until(&mut bytes, time::after(100)?)?;
        wire::decode(&bytes[..len]).ok_or(Error(abi::IO_ERROR as i64))
    }
}
