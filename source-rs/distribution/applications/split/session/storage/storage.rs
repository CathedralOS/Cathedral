// One asynchronous exchange lets interaction continue while storage is busy.
// Platform Scopes authenticates/stamps; Split chooses two distinct child roots.
// An upstream regrant only restores these standing grants in the same realm.
use super::*;
use cathedral_matrix::world::storage::{Prepared, Scopes};
use cathedral_user_runtime::{ipc::Handle, memory::Object, server::Request};
const DEADLINE: u64 = wire::STORAGE_DEADLINE_TICKS + 200;
struct Pending {
    request: Request,
    words: wire::Packet,
    object: Option<Object>,
    refresh: bool,
    deadline: u64,
}
pub struct Storage {
    pair: Option<Pair>,
    realm: u64,
    epoch: u64,
    roots: [u64; 2],
    incarnations: [Option<u64>; 2],
    scopes: Scopes,
    pending: Option<Pending>,
}
impl Storage {
    pub fn new(pair: Pair, realm: u64, roots: [u64; 2]) -> Self {
        Self {
            pair: Some(pair),
            realm,
            epoch: 0,
            roots,
            incarnations: [None; 2],
            scopes: Scopes::empty(),
            pending: None,
        }
    }
    pub fn install(&mut self, slot: usize, incarnation: u64) {
        self.incarnations[slot] = Some(incarnation);
    }
    pub fn revoke(&mut self, slot: usize) {
        self.incarnations[slot] = None;
        self.scopes.revoke(slot);
    }
    pub fn wait(&self) -> Option<(Handle, u64)> {
        self.pending
            .as_ref()
            .map(|p| (self.pair.unwrap().1, p.deadline))
    }
    pub fn start(
        &mut self,
        request: Request,
        object: Option<Object>,
        server: &mut Server,
    ) -> Result<(), Error> {
        let Some(words) = wire::decode(&request.bytes[..request.len]) else {
            return server.reply(&request, &wire::encode(wire::error(abi::INVALID_ARGUMENT)));
        };
        let slot = request.client.unwrap() - 2;
        if self.incarnations[slot] != Some(request.incarnation) || self.pair.is_none() {
            return server.reply(&request, &wire::encode(wire::error(abi::DENIED)));
        }
        assert!(
            self.pending.is_none(),
            "storage streams must be paused during exchange"
        );
        self.pending = Some(Pending {
            request,
            words,
            object,
            refresh: true,
            deadline: time::after(DEADLINE)?,
        });
        match self
            .pair
            .unwrap()
            .0
            .send(&wire::encode([wire::STORAGE_INFO, 0, 0, 0, 0, 0, 0, 0]))
        {
            Ok(()) => Ok(()),
            Err(e) => self.retire(e, server),
        }
    }
    pub fn progress(&mut self, server: &mut Server) -> Result<(), Error> {
        let Some((receive, deadline)) = self.wait() else {
            return Ok(());
        };
        let (reply, object) = match upstream::exchange::poll(receive, deadline) {
            Ok(Some(reply)) => reply,
            Ok(None) => return Ok(()),
            Err(e) => return self.retire(e, server),
        };
        if !self.pending.as_ref().unwrap().refresh {
            return self.finish(reply, object, server);
        }
        drop(object);
        let epoch = if reply[0] == 0 && reply[2] == self.realm {
            reply[1]
        } else {
            0
        };
        if self.epoch != epoch {
            self.epoch = epoch;
            self.scopes.clear();
            if epoch != 0 {
                for slot in 0..2 {
                    if let Some(incarnation) = self.incarnations[slot] {
                        self.scopes
                            .grant(slot, incarnation, 0, self.roots[slot], 1)
                            .map_err(|c| Error(c as i64))?;
                    }
                }
            }
        }
        let pending = self.pending.as_mut().unwrap();
        let slot = pending.request.client.unwrap() - 2;
        let mut words =
            match self
                .scopes
                .prepare(slot, pending.request.incarnation, pending.words, self.realm)
            {
                Ok(Prepared::Reply(reply)) => return self.finish(reply, None, server),
                Ok(Prepared::Forward { words, .. }) => words,
                Err(code) => return self.finish(wire::error(code), None, server),
            };
        words[0] = words[0] & 0xffff | self.epoch << wire::STORAGE_SCOPE_BITS;
        pending.refresh = false;
        let send = self.pair.unwrap().0;
        let result = match pending.object.take() {
            Some(object) => send.send_memory(&wire::encode(words), &object),
            None => send.send(&wire::encode(words)),
        };
        match result {
            Ok(()) => Ok(()),
            Err(e) => self.retire(e, server),
        }
    }
    fn retire(&mut self, error: Error, server: &mut Server) -> Result<(), Error> {
        // Never consume a late reply as the answer to a subsequent request.
        if let Some((send, receive)) = self.pair.take() {
            let _ = send.close();
            let _ = receive.close();
        }
        self.scopes.clear();
        self.epoch = 0;
        self.finish(wire::error(error.0 as u64), None, server)
    }
    fn finish(
        &mut self,
        words: wire::Packet,
        object: Option<Object>,
        server: &mut Server,
    ) -> Result<(), Error> {
        let request = self.pending.take().unwrap().request;
        if let Some(object) = object {
            if server
                .reply_memory(&request, &wire::encode(words), &object)
                .is_err()
            {
                server.reply(&request, &wire::encode(wire::error(abi::BUSY)))?;
            }
            Ok(())
        } else {
            server.reply(&request, &wire::encode(words))
        }
    }
}
