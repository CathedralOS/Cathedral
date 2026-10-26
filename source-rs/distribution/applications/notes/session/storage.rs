//! Each note is one private object in the binding's realm, which holds up to
//! MAX_NOTES of them. Saving publishes the whole encoded document; duplicating
//! asks the store to copy the confirmed object beside it, which shares its
//! content rather than rewriting it. The app never learns where its realm sits
//! in the root: a subtree and a whole root look alike.
//! IPC crosses into the parent matrix's service/parent/storage, then the
//! object provider's service entrance.
use super::*;
use cathedral_platform_contracts::storage as store;
use cathedral_user_runtime::{memory::Private, storage::Scope};
pub const MAX_NOTES: usize = 8;
pub struct Notes {
    scope: Option<Scope<'static>>,
    ids: [u64; MAX_NOTES],
    pub count: usize,
    pub current: usize,
    pub revision: u64,
    _style: u64,
}
impl Notes {
    pub fn new(scope: Option<Scope<'static>>, style: u64) -> Self {
        Self {
            scope,
            ids: [0; MAX_NOTES],
            count: 0,
            current: 0,
            revision: 0,
            _style: style,
        }
    }
}
fn traced<T>(style: u64, op: u64, call: impl FnOnce() -> T) -> T {
    #[cfg(feature = "trace-storage")]
    time::storage_trace("app-send", style, op);
    let result = call();
    #[cfg(feature = "trace-storage")]
    time::storage_trace("app-reply", style, op);
    let _ = (style, op);
    result
}
impl Notes {
    pub fn temporary(&self) -> bool {
        self.scope.is_none()
    }
    /// Acquire the current binding, enumerate the realm and read the current
    /// note. Missing authority is a refusal, never an empty successful reload.
    pub fn load(&mut self) -> Result<Document, u64> {
        let scope = self.scope.as_mut().ok_or(abi::DENIED)?;
        scope.bind()?;
        let realm = scope.realm();
        self.count = 0;
        let mut after = 0;
        loop {
            let request = store::Request::new(store::LIST, realm, 0, after, 0);
            let (reply, _) = traced(self._style, store::LIST, || scope.request(request, None))?;
            let words = store::unpack(&reply).ok_or(abi::IO_ERROR)?;
            if words[0] != 0 {
                return Err(words[0]);
            }
            let found = words[1] as usize;
            for &id in &words[2..2 + found.min(store::LIST_IDS)] {
                if self.count == MAX_NOTES {
                    break;
                }
                self.ids[self.count] = id;
                self.count += 1;
                after = id;
            }
            if found < store::LIST_IDS || self.count == MAX_NOTES {
                break;
            }
        }
        if self.count == 0 {
            self.current = 0;
            self.revision = 0;
            return Ok(Document::default());
        }
        self.current = self.current.min(self.count - 1);
        self.read()
    }
    fn read(&mut self) -> Result<Document, u64> {
        let scope = self.scope.as_mut().ok_or(abi::DENIED)?;
        let id = self.ids[self.current];
        let request = store::Request::new(store::HEAD, id, 0, 0, 0);
        let head = traced(self._style, store::HEAD, || scope.call(request))?;
        if head.length == 0 || head.length > cathedral_notes::document::ENCODED as u64 {
            return Err(abi::IO_ERROR);
        }
        let request = store::Request::new(store::BULK, id, head.revision, 0, head.length);
        let (reply, attachment) =
            traced(self._style, store::BULK, || scope.request(request, None))?;
        let reply = store::Reply::decode(&reply)?;
        let mut attachment = attachment.ok_or(abi::IO_ERROR)?;
        let bytes = attachment.map().map_err(|e| e.0 as u64)?;
        let doc = Document::decode(&bytes[..reply.extra as usize]).ok_or(abi::IO_ERROR)?;
        self.revision = reply.revision;
        Ok(doc)
    }
    pub fn save(&mut self, doc: &Document) -> Result<(), u64> {
        #[cfg(feature = "trace-storage")]
        time::storage_trace("note-save-begin", self._style, 0);
        let result = self.publish(doc);
        #[cfg(feature = "trace-storage")]
        time::storage_trace("note-save-end", self._style, 0);
        result
    }
    fn publish(&mut self, doc: &Document) -> Result<(), u64> {
        let scope = self.scope.as_mut().ok_or(abi::DENIED)?;
        let (bytes, len) = doc.encode();
        let mut pages = Private::allocate(1).map_err(|e| e.0 as u64)?;
        pages.bytes_mut()[..len].copy_from_slice(&bytes[..len]);
        let object = pages.freeze().map_err(|e| e.0 as u64)?;
        // A first note is created in the realm; later saves name the note.
        let request = if self.count == 0 {
            store::Request::new(store::PUBLISH, 0, 0, scope.realm(), len as u64)
        } else {
            store::Request::new(
                store::PUBLISH,
                self.ids[self.current],
                self.revision,
                0,
                len as u64,
            )
        };
        let (reply, _) = traced(self._style, store::PUBLISH, || {
            scope.request(request, Some(&object))
        })?;
        let reply = store::Reply::decode(&reply)?;
        if request.object == 0 {
            self.ids[0] = reply.object;
            self.count = 1;
            self.current = 0;
        }
        self.revision = reply.revision;
        Ok(())
    }
    /// A new note beside the current one, sharing its confirmed content.
    pub fn duplicate(&mut self) -> Result<Document, u64> {
        let scope = self.scope.as_mut().ok_or(abi::DENIED)?;
        if self.count == 0 || self.count == MAX_NOTES {
            return Err(abi::NO_MEMORY);
        }
        let request = store::Request::new(
            store::COPY,
            self.ids[self.current],
            self.revision,
            scope.realm(),
            0,
        );
        let reply = traced(self._style, store::COPY, || scope.call(request))?;
        self.ids[self.count] = reply.object;
        self.current = self.count;
        self.count += 1;
        self.read()
    }
    pub fn next(&mut self) -> Result<Document, u64> {
        if self.count == 0 {
            return Ok(Document::default());
        }
        self.current = (self.current + 1) % self.count;
        self.read()
    }
}
