//! Child-local object storage through the parent matrix. STORAGE_INFO supplies
//! a binding revision that scopes every later request and the realm the
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
/// Longer than the matrix chain's own deadline, which grows by a hop's margin
/// at each level, so a stalled provider is reported by a matrix's verdict
/// rather than by this client giving up first.
const DEADLINE_TICKS: u64 = 2 * matrix::STORAGE_DEADLINE_TICKS;
pub struct Scope {
    pair: (Handle, Handle),
    binding: u64,
    realm: u64,
    alive: bool,
    direct: bool,
}
impl Scope {
    pub fn new(pair: (Handle, Handle)) -> Self {
        Self {
            pair,
            binding: 0,
            realm: 0,
            alive: true,
            direct: false,
        }
    }
    /// A provider connection explicitly supplied to the storage root adapter.
    pub fn provider(pair: (Handle, Handle)) -> Self {
        let mut scope = Self::new(pair);
        scope.direct = true;
        scope
    }
    pub fn connected(&self) -> bool {
        self.alive
    }
    /// Acquire the currently granted binding. A revoked or replaced grant
    /// changes this number and every request scoped by the old one is denied.
    pub fn bind(&mut self) -> Result<u64, u64> {
        if self.direct {
            return Ok(1);
        }
        let (reply, _) = self.exchange([matrix::STORAGE_INFO, 0, 0, 0, 0, 0, 0, 0], None)?;
        let words = storage::unpack(&reply).ok_or(abi::IO_ERROR)?;
        if words[0] != 0 {
            return Err(words[0]);
        }
        self.binding = words[1];
        self.realm = words[2];
        Ok(self.binding)
    }
    /// The object this binding's realm is rooted at: the parent to list and
    /// to create under. Zero is the whole root. Valid after `bind`.
    pub fn realm(&self) -> u64 {
        self.realm
    }
    /// One scoped request with optional attachments in either direction.
    pub fn request(
        &mut self,
        request: storage::Request,
        attachment: Option<&Object>,
    ) -> Result<([u8; 64], Option<Object>), u64> {
        let mut words = request.words();
        if !self.direct {
            words[0] =
                (self.binding << matrix::STORAGE_SCOPE_BITS) | (matrix::STORAGE_BASE + request.op);
        }
        self.exchange(words, attachment)
    }
    /// A request whose reply has the common shape and no attachment.
    pub fn call(&mut self, request: storage::Request) -> Result<storage::Reply, u64> {
        let (reply, attachment) = self.request(request, None)?;
        drop(attachment);
        storage::Reply::decode(&reply)
    }
    /// Small immutable request payload, with custody released after completion.
    pub fn attached(&mut self, request: storage::Request, bytes: &[u8]) -> Result<[u8; 64], u64> {
        if bytes.len() > 4096 || request.len != bytes.len() as u64 {
            return Err(abi::INVALID_ARGUMENT);
        }
        let mut pages = crate::memory::Private::allocate(1).map_err(|e| e.0 as u64)?;
        pages.bytes_mut()[..bytes.len()].copy_from_slice(bytes);
        let object = pages.freeze().map_err(|e| e.0 as u64)?;
        let (reply, extra) = self.request(request, Some(&object))?;
        drop(extra);
        if storage::word(&reply, 0) != 0 {
            return Err(storage::word(&reply, 0));
        }
        Ok(reply)
    }
    pub fn lookup(&mut self, path: &[u8]) -> Result<u64, u64> {
        let bytes = self.attached(
            storage::Request::new(storage::LOOKUP, self.realm, 0, 0, path.len() as u64),
            path,
        )?;
        Ok(storage::Reply::decode(&bytes)?.object)
    }
    pub fn metadata(&mut self, object: u64) -> Result<storage::metadata::Metadata, u64> {
        let mut bytes = [0; 64];
        let mut revision = 0;
        for field in 0..2 {
            let (reply, extra) = self.request(
                storage::Request::new(storage::METADATA, object, 0, field, 0),
                None,
            )?;
            drop(extra);
            storage::Reply::decode(&reply)?;
            let observed = storage::word(&reply, 1);
            if field != 0 && observed != revision {
                return Err(abi::WOULD_BLOCK);
            }
            revision = observed;
            bytes[field as usize * 32..field as usize * 32 + 32].copy_from_slice(&reply[32..]);
        }
        storage::metadata::Metadata::decode(&bytes).ok_or(abi::IO_ERROR)
    }
    pub fn filter(&mut self, media_type: &[u8], after: u64, out: &mut [u64]) -> Result<usize, u64> {
        let reply = self.attached(
            storage::Request::new(
                storage::FILTER,
                self.realm,
                0,
                after,
                media_type.len() as u64,
            ),
            media_type,
        )?;
        let words = storage::unpack(&reply).ok_or(abi::IO_ERROR)?;
        let count = (words[1] as usize).min(out.len()).min(storage::LIST_IDS);
        out[..count].copy_from_slice(&words[2..2 + count]);
        Ok(count)
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
