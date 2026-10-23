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
/// End-to-end budget, preserved by every matrix. The root caps provider work
/// more tightly, leaving room for its failure verdict to return to the client.
const DEADLINE_TICKS: u64 = storage::EXCHANGE_TICKS;
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
        let deadline = time::after(DEADLINE_TICKS).map_err(|e| e.0 as u64)?;
        let (reply, _) = self.exchange([matrix::STORAGE_INFO, 0, 0, 0, 0, 0, 0, deadline], None)?;
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
        words[7] = storage::deadline(
            time::now().map_err(|e| e.0 as u64)?,
            words[7],
            DEADLINE_TICKS,
        )
        .ok_or(abi::TIMED_OUT)?;
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
    /// Atomic creation in this realm. An ambiguous transport failure retires
    /// this stream; reacquire and LOOKUP the name before deciding what to do.
    pub fn create_named(
        &mut self,
        metadata: storage::metadata::Metadata,
        contents: &[u8],
    ) -> Result<storage::Reply, u64> {
        let length = contents
            .len()
            .checked_add(storage::metadata::BYTES)
            .filter(|length| *length <= storage::MAX_BYTES)
            .ok_or(storage::TOO_LARGE)?;
        if metadata.name == [0; storage::metadata::FIELD]
            || storage::metadata::Metadata::decode(&metadata.encode()) != Some(metadata)
        {
            return Err(abi::INVALID_ARGUMENT);
        }
        let mut pages =
            crate::memory::Private::allocate(length.div_ceil(4096)).map_err(|e| e.0 as u64)?;
        pages.bytes_mut()[..storage::metadata::BYTES].copy_from_slice(&metadata.encode());
        pages.bytes_mut()[storage::metadata::BYTES..length].copy_from_slice(contents);
        let object = pages.freeze().map_err(|e| e.0 as u64)?;
        let (reply, extra) = self.request(
            storage::Request::new(storage::CREATE_NAMED, 0, 0, self.realm, length as u64),
            Some(&object),
        )?;
        drop(extra);
        storage::Reply::decode(&reply)
    }
    /// Both metadata fields, two round trips; refused if a rename lands between them.
    pub fn metadata(&mut self, object: u64) -> Result<storage::metadata::Metadata, u64> {
        let mut bytes = [0; 64];
        let (revision, name) = self.field(object, 0)?;
        let (observed, media_type) = self.field(object, 1)?;
        if observed != revision {
            return Err(abi::WOULD_BLOCK);
        }
        bytes[..32].copy_from_slice(&name);
        bytes[32..].copy_from_slice(&media_type);
        storage::metadata::Metadata::decode(&bytes).ok_or(abi::IO_ERROR)
    }
    /// The name alone, one round trip: for a caller that already knows the
    /// type, such as a launcher listing the executables a filter returned.
    pub fn name(&mut self, object: u64) -> Result<[u8; 32], u64> {
        self.field(object, 0).map(|(_, name)| name)
    }
    fn field(&mut self, object: u64, index: u64) -> Result<(u64, [u8; 32]), u64> {
        let (reply, extra) = self.request(
            storage::Request::new(storage::METADATA, object, 0, index, 0),
            None,
        )?;
        drop(extra);
        storage::Reply::decode(&reply)?;
        let mut field = [0; 32];
        field.copy_from_slice(&reply[32..]);
        Ok((storage::word(&reply, 1), field))
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
    /// One page naming the direct children after `after`: their numbers,
    /// revisions and metadata, decoded row by row from the sealed page.
    pub fn entries(&mut self, parent: u64, after: u64) -> Result<Listing, u64> {
        let (reply, page) = self.request(
            storage::Request::new(storage::ENTRIES, parent, 0, after, 0),
            None,
        )?;
        let words = storage::unpack(&reply).ok_or(abi::IO_ERROR)?;
        if words[0] != 0 {
            drop(page);
            return Err(words[0]);
        }
        let count = (words[1] as usize).min(storage::ENTRIES_PER_PAGE);
        if count != 0 && page.is_none() {
            return Err(abi::IO_ERROR);
        }
        Ok(Listing {
            page,
            count,
            last: words[2],
        })
    }
    /// The numbers of the realm's direct children carrying these names: one
    /// listing per page of children instead of one LOOKUP per name. A name
    /// nobody carries stays None.
    pub fn named(&mut self, names: &[&[u8]], out: &mut [Option<u64>]) -> Result<(), u64> {
        out.fill(None);
        let realm = self.realm;
        let mut after = 0;
        loop {
            let mut listing = self.entries(realm, after)?;
            for index in 0..listing.len() {
                let row = listing.row(index)?;
                let name = storage::metadata::text(&row.metadata.name);
                for (slot, wanted) in out.iter_mut().zip(names) {
                    if name == *wanted {
                        *slot = Some(row.object);
                    }
                }
            }
            after = listing.last;
            if !listing.is_full() || out.iter().all(Option::is_some) {
                return Ok(());
            }
        }
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
            let deadline = if words[7] == 0 {
                time::after(DEADLINE_TICKS)?
            } else {
                words[7]
            };
            ipc::wait_many(&[self.pair.1], deadline)?;
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
/// One page of an ENTRIES reply. Rows decode on demand, so a reader keeps no
/// page-sized copy of its own; `last` is the number to continue after.
pub struct Listing {
    page: Option<Object>,
    count: usize,
    pub last: u64,
}
impl Listing {
    pub fn len(&self) -> usize {
        self.count
    }
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    /// A full page may be followed by more children after `last`.
    pub fn is_full(&self) -> bool {
        self.count == storage::ENTRIES_PER_PAGE
    }
    pub fn row(&mut self, index: usize) -> Result<storage::Entry, u64> {
        if index >= self.count {
            return Err(abi::INVALID_ARGUMENT);
        }
        let page = self.page.as_mut().ok_or(abi::IO_ERROR)?;
        let bytes = page.map().map_err(|e| e.0 as u64)?;
        let start = index * storage::ENTRY_BYTES;
        storage::Entry::decode(&bytes[start..start + storage::ENTRY_BYTES]).ok_or(abi::IO_ERROR)
    }
}
