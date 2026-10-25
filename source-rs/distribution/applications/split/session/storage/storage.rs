// One asynchronous exchange lets interaction continue while storage is busy.
// Platform Scopes authenticates and stamps; Split chooses two distinct child roots.
// An upstream regrant only restores these standing grants in the same realm.
use super::*;
use cathedral_matrix::world::storage::{Prepared, Proof, Scopes};
use cathedral_user_runtime::{
    connection::Client, event::Interest, ipc::Handle, memory::Object, server::Request,
    storage::Scope,
};
struct Pending {
    request: Request,
    words: wire::Packet,
    object: Option<Object>,
    refresh: bool,
    deadline: u64,
    validating: bool,
    folded: Option<(wire::Packet, Proof)>,
}
pub struct Storage {
    client: Option<Client>,
    realm: u64,
    epoch: u64,
    roots: [u64; 2],
    incarnations: [Option<u64>; 2],
    scopes: Scopes,
    pending: Option<Pending>,
}
impl Storage {
    pub fn idle(&self) -> bool {
        self.pending.is_none() && self.client.is_some()
    }
    pub fn idle_scope(&mut self) -> Option<Scope<'_>> {
        if self.pending.is_some() {
            return None;
        }
        self.client.as_mut().map(Scope::shared)
    }
    pub fn new(client: Client, realm: u64, roots: [u64; 2]) -> Self {
        Self {
            client: Some(client),
            realm,
            epoch: 0,
            roots,
            incarnations: [None; 2],
            scopes: Scopes::empty(),
            pending: None,
        }
    }
    pub fn reset_roots(&mut self, realm: u64, roots: [u64; 2]) {
        assert!(self.idle());
        self.realm = realm;
        self.roots = roots;
        self.epoch = 0;
        self.incarnations = [None; 2];
        self.scopes.clear();
    }
    pub fn install(&mut self, slot: usize, incarnation: u64) {
        self.incarnations[slot] = Some(incarnation);
    }
    pub fn revoke(&mut self, slot: usize) {
        self.incarnations[slot] = None;
        self.scopes.revoke(slot);
    }
    pub fn wait(&self) -> Option<(Handle, u64)> {
        self.pending.as_ref().map(|p| {
            (
                match self.client.as_ref().unwrap().interests().unwrap()[0] {
                    Interest::Ipc(h) => h,
                    _ => unreachable!(),
                },
                p.deadline,
            )
        })
    }
    pub fn wake(&self) -> Option<Interest> {
        self.pending.as_ref()?;
        self.client.as_ref()?.interests().ok().map(|i| i[1])
    }
    pub fn transfer_deadline(&self) -> Option<u64> {
        self.client.as_ref()?.transfer_deadline()
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
        if self.incarnations[slot] != Some(request.incarnation) || self.client.is_none() {
            return server.reply(&request, &wire::encode(wire::error(abi::DENIED)));
        }
        let Some(deadline) = cathedral_platform_contracts::storage::deadline(
            time::now()?,
            words[7],
            cathedral_platform_contracts::storage::EXCHANGE_TICKS,
        ) else {
            return server.reply(&request, &wire::encode(wire::error(abi::TIMED_OUT)));
        };
        assert!(
            self.pending.is_none(),
            "storage streams must be paused during exchange"
        );
        self.pending = Some(Pending {
            request,
            words,
            object,
            refresh: true,
            deadline,
            validating: false,
            folded: None,
        });
        match self.client.as_mut().unwrap().begin(
            &wire::encode([wire::STORAGE_INFO, 0, 0, 0, 0, 0, 0, deadline]),
            None,
        ) {
            Ok(()) => Ok(()),
            Err(e) => self.retire(e, server),
        }
    }
    pub fn progress(&mut self, server: &mut Server) -> Result<(), Error> {
        let Some((_, deadline)) = self.wait() else {
            return Ok(());
        };
        let response = (|| {
            if let Some((bytes, n, object)) = self.client.as_mut().unwrap().poll()? {
                return wire::decode(&bytes[..n])
                    .map(|w| Some((w, object)))
                    .ok_or(Error(abi::IO_ERROR as i64));
            }
            if time::reached(time::now()?, deadline) {
                return Err(Error(abi::TIMED_OUT as i64));
            }
            Ok(None)
        })();
        let (reply, object) = match response {
            Ok(Some(reply)) => reply,
            Ok(None) => return Ok(()),
            Err(e) => return self.retire(e, server),
        };
        if self.pending.as_ref().unwrap().validating {
            drop(object);
            if reply[0] != 0 {
                return self.finish(reply, None, server);
            }
            let pending = self.pending.as_mut().unwrap();
            let (words, proof) = pending.folded.take().unwrap();
            if let Err(code) = self.scopes.confirm(proof) {
                return self.finish(wire::error(code), None, server);
            }
            pending.validating = false;
            return self.send(words, server);
        }
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
        let words =
            match self
                .scopes
                .prepare(slot, pending.request.incarnation, pending.words, self.realm)
            {
                Ok(Prepared::Reply(reply)) => return self.finish(reply, None, server),
                Ok(Prepared::Forward { words, .. }) => words,
                Ok(Prepared::Validate {
                    check,
                    words,
                    proof,
                    ..
                }) => {
                    pending.validating = true;
                    pending.folded = Some((words, proof));
                    check
                }
                Err(code) => return self.finish(wire::error(code), None, server),
            };
        pending.refresh = false;
        self.send(words, server)
    }
    fn send(&mut self, mut words: wire::Packet, server: &mut Server) -> Result<(), Error> {
        let pending = self.pending.as_mut().unwrap();
        if time::reached(time::now()?, pending.deadline) {
            return self.finish(wire::error(abi::TIMED_OUT), None, server);
        }
        words[0] = words[0] & 0xffff | self.epoch << wire::STORAGE_SCOPE_BITS;
        words[7] = pending.deadline;
        let object = if pending.validating {
            None
        } else {
            pending.object.take()
        };
        let result = self
            .client
            .as_mut()
            .unwrap()
            .begin(&wire::encode(words), object.as_ref());
        match result {
            Ok(()) => Ok(()),
            Err(e) => self.retire(e, server),
        }
    }
    fn retire(&mut self, error: Error, server: &mut Server) -> Result<(), Error> {
        // Never consume a late reply as the answer to a subsequent request.
        self.client = None;
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
