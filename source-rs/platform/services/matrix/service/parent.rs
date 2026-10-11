//! Root adapter supplies a grant; nested instances obtain the same shape via IPC.
//! A lost/invalid upstream grant empties the world before any child can query it.
use super::*;
use cathedral_user_runtime::{ipc::Handle, link::Link, time};
pub(super) struct Parent {
    pub upstream: bool,
    pair: Option<(Handle, Handle)>,
    pub grant: Grant,
}
impl Parent {
    pub fn new(upstream: bool) -> Self {
        Self {
            upstream,
            pair: None,
            grant: Grant::default(),
        }
    }
    pub fn refresh(&mut self, world: &mut World) -> Result<(), Error> {
        if !self.upstream {
            world.inherit(self.grant);
            return Ok(());
        }
        if self.pair.is_none() {
            match Link::at(0)?.connect() {
                Ok(pair) => self.pair = Some(pair),
                Err(Error(code)) if code == abi::PEER_CLOSED as i64 => {
                    world.inherit(Grant::default());
                    return Ok(());
                }
                Err(error) => return Err(error),
            }
        }
        let view = match self.call([wire::VIEW, 0, 0, 0, 0, 0, 0, 0]) {
            Ok(reply) => reply,
            Err(Error(code)) if code == abi::PEER_CLOSED as i64 => {
                self.pair = None;
                world.inherit(Grant::default());
                return Ok(());
            }
            // A timeout may leave a late reply: retire rather than reuse that stream.
            Err(error) => return Err(error),
        };
        self.grant = if view[0] == 0 && view[5..] == [0; 3] {
            Grant {
                epoch: view[1],
                width: view[2],
                height: view[3],
                keys: view[4],
            }
        } else {
            Grant::default()
        };
        world.inherit(self.grant);
        if !self.grant.valid() {
            return Ok(());
        }
        // Bounded drain. Ancestors use exactly this same child protocol.
        for _ in 0..8 {
            let event = self.call([wire::NEXT, self.grant.epoch, 0, 0, 0, 0, 0, 0])?;
            if event[0] != 0 || event[1] != self.grant.epoch {
                world.inherit(Grant::default());
                break;
            }
            match event[2] {
                wire::NONE => break,
                wire::POINTER => world.pointer(event[3], event[4], event[5]),
                wire::KEY => world.key(event[3], event[4]),
                wire::CANCEL => world.cancel(),
                _ => {
                    world.inherit(Grant::default());
                    break;
                }
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
