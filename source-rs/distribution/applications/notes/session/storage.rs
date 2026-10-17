//! Two app records, one catalog transaction. IPC crosses into the parent matrix's
//! service/parent/storage, then the database provider's catalog entrance.
use super::*;
use cathedral_platform_contracts::storage as store;
pub struct Catalog {
    pair: Pair,
    binding: u64,
    pub generation: u64,
    exists: bool,
    alive: bool,
    _style: u64,
}
impl Catalog {
    pub fn new(pair: Pair, style: u64) -> Self {
        Self {
            pair,
            binding: 0,
            generation: 0,
            exists: false,
            alive: true,
            _style: style,
        }
    }
    fn exchange(&mut self, words: wire::Packet) -> Result<wire::Packet, u64> {
        if !self.alive {
            return Err(abi::PEER_CLOSED);
        }
        let mut bytes = [0; 64];
        let result = self
            .pair
            .0
            .send(&wire::encode(words))
            .and_then(|()| self.pair.1.receive_until(&mut bytes, time::after(300)?))
            .map_err(|e| e.0 as u64)
            .and_then(|n| wire::decode(&bytes[..n]).ok_or(abi::IO_ERROR));
        if result.is_err() {
            self.alive = false;
        } // No uncorrelated stream reuse.
        result
    }
    fn request(
        &mut self,
        op: u64,
        object: u64,
        expected: u64,
        payload: &[u8],
    ) -> Result<store::Record, u64> {
        let opcode = self.binding << wire::STORAGE_SCOPE_BITS | (wire::STORAGE_BASE + op);
        let words =
            wire::decode(&store::request(opcode, object, expected, payload).unwrap()).unwrap();
        #[cfg(feature = "trace-storage")]
        time::storage_trace("app-send", self._style, op);
        let result = self
            .exchange(words)
            .and_then(|reply| store::Record::decode(&wire::encode(reply)));
        #[cfg(feature = "trace-storage")]
        time::storage_trace("app-reply", self._style, op);
        result
    }
    pub fn load(&mut self) -> Result<Document, u64> {
        let info = self.exchange([wire::STORAGE_INFO, 0, 0, 0, 0, 0, 0, 0])?;
        if info[0] != 0 {
            return Err(info[0]);
        }
        self.binding = info[1];
        // Staging is ours on this connection. Reconciliation abandons any old draft.
        self.request(store::ABORT, 0, 0, &[])?;
        let listing = self.request(store::LIST, 0, 0, &[])?;
        let first = self.request(store::READ, 1, 0, &[]);
        let second = self.request(store::READ, 2, 0, &[]);
        let doc = match (first, second) {
            (Err(store::NOT_FOUND), Err(store::NOT_FOUND)) if listing.length == 0 => {
                self.exists = false;
                Document::default()
            }
            (Ok(a), Ok(b))
                if a.generation == listing.generation
                    && b.generation == a.generation
                    && (a.length == 32 || b.length == 0) =>
            {
                let mut bytes = [0; 64];
                bytes[..a.length].copy_from_slice(a.payload());
                bytes[a.length..a.length + b.length].copy_from_slice(b.payload());
                let doc = Document::decode(&bytes[..a.length + b.length]).ok_or(abi::IO_ERROR)?;
                self.exists = true;
                doc
            }
            (Err(error), _) | (_, Err(error)) => return Err(error),
            _ => return Err(abi::IO_ERROR),
        };
        self.generation = listing.generation;
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
        self.request(store::BEGIN, 0, self.generation, &[])?;
        let (bytes, len) = doc.encode();
        let split = len.min(32);
        let op = if self.exists {
            store::STAGE_REPLACE
        } else {
            store::STAGE_CREATE
        };
        self.request(op, 1, 0, &bytes[..split])?;
        self.request(op, 2, 0, &bytes[split..len])?;
        let committed = self.request(store::COMMIT, 0, 0, &[])?;
        self.generation = committed.generation;
        self.exists = true;
        Ok(())
    }
}
