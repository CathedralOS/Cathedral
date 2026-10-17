//! Child-local object storage through the parent matrix. STORAGE_INFO supplies
//! a binding revision that scopes every later request and the home object the
//! binding is rooted at; the parent substitutes its own upstream scope at each
//! hop. Replies carry no correlation identity, so one failed exchange retires
//! the connection rather than risking a late reply being read as the answer to
//! a newer request.
use crate::{
    Error, abi,
    ipc::{self, Handle},
    memory::Object,
    time,
};
use cathedral_platform_contracts::{matrix, storage};
const DEADLINE_TICKS: u64 = 300;
pub struct Scope {
    pair: (Handle, Handle),
    binding: u64,
    home: u64,
    alive: bool,
}
impl Scope {
    pub fn new(pair: (Handle, Handle)) -> Self {
        Self {
            pair,
            binding: 0,
            home: 0,
            alive: true,
        }
    }
    pub fn connected(&self) -> bool {
        self.alive
    }
    /// Acquire the currently granted binding. A revoked or replaced grant
    /// changes this number and every request scoped by the old one is denied.
    pub fn bind(&mut self) -> Result<u64, u64> {
        let (reply, _) = self.exchange([matrix::STORAGE_INFO, 0, 0, 0, 0, 0, 0, 0], None)?;
        let words = storage::unpack(&reply).ok_or(abi::IO_ERROR)?;
        if words[0] != 0 {
            return Err(words[0]);
        }
        self.binding = words[1];
        self.home = words[2];
        Ok(self.binding)
    }
    /// The object this binding is rooted at: the parent to list and to create
    /// under. Zero is the whole root. Valid after `bind`.
    pub fn home(&self) -> u64 {
        self.home
    }
    /// One scoped request with optional attachments in either direction.
    pub fn request(
        &mut self,
        request: storage::Request,
        attachment: Option<&Object>,
    ) -> Result<([u8; 64], Option<Object>), u64> {
        let mut words = request.words();
        words[0] =
            (self.binding << matrix::STORAGE_SCOPE_BITS) | (matrix::STORAGE_BASE + request.op);
        self.exchange(words, attachment)
    }
    /// A request whose reply has the common shape and no attachment.
    pub fn call(&mut self, request: storage::Request) -> Result<storage::Reply, u64> {
        let (reply, attachment) = self.request(request, None)?;
        drop(attachment);
        storage::Reply::decode(&reply)
    }
    fn exchange(
        &mut self,
        words: [u64; 8],
        attachment: Option<&Object>,
    ) -> Result<([u8; 64], Option<Object>), u64> {
        if !self.alive {
            return Err(abi::PEER_CLOSED);
        }
        let result = (|| {
            let bytes = storage::pack(words);
            match attachment {
                Some(object) => self.pair.0.send_memory(&bytes, object)?,
                None => self.pair.0.send(&bytes)?,
            }
            ipc::wait_many(&[self.pair.1], time::after(DEADLINE_TICKS)?)?;
            let mut reply = [0; 64];
            let (len, object) = self.pair.1.try_receive_memory(&mut reply)?;
            if len != 64 {
                return Err(Error(abi::IO_ERROR as i64));
            }
            Ok((reply, object))
        })();
        match result {
            Ok(reply) => Ok(reply),
            Err(error) => {
                self.alive = false;
                Err(error.0 as u64)
            }
        }
    }
}
