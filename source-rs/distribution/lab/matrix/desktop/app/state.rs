//! App schema over its matrix-local catalog capability. Persist before presenting;
//! revoked bindings cannot save, and a replacement reads only committed state.
use super::*;
use cathedral_platform_contracts::storage as store;
pub(super) struct State {
    binding: u64,
    pub generation: u64,
    style: u64,
    pub alternate: bool,
    pub error: u64,
    volatile: bool,
    transport_alive: core::cell::Cell<bool>,
}
impl State {
    pub fn open(parent: Pair, style: u64) -> Self {
        let mut state = Self {
            binding: 0,
            generation: 0,
            style,
            alternate: false,
            error: 0,
            // Only the explicitly diskless sibling probe may change volatile state.
            volatile: style == 2,
            transport_alive: core::cell::Cell::new(true),
        };
        let _ = state.reload(parent);
        state
    }
    /// Explicit reconciliation: acquire only the currently granted local binding,
    /// read first, and create only after observing absence and the current revision.
    /// Never replay an uncertain toggle or replace confirmed state on failed reads.
    pub fn reload(&mut self, parent: Pair) -> Result<(), u64> {
        let result = self.load(parent);
        self.error = result.err().unwrap_or(0);
        result
    }
    fn load(&mut self, parent: Pair) -> Result<(), u64> {
        let info = self.exchange(parent, [wire::STORAGE_INFO, 0, 0, 0, 0, 0, 0, 0])?;
        if info[0] == abi::DENIED && self.volatile {
            return Ok(());
        }
        if info[0] != 0 {
            return Err(info[0]);
        }
        self.binding = info[1];
        self.volatile = false;
        let record = match self.request(parent, store::READ, 2, 0, &[]) {
            Ok(record) => record,
            Err(store::NOT_FOUND) => {
                let generation = self.request(parent, store::LIST, 0, 0, &[])?.generation;
                let created =
                    self.request(parent, store::CREATE, 2, generation, &self.payload(false));
                // Even an error may hide a committed CREATE. A successful readback
                // settles the outcome; otherwise leave it unresolved for the owner/user.
                match self.request(parent, store::READ, 2, 0, &[]) {
                    Ok(record) => record,
                    Err(read_error) => return Err(created.err().unwrap_or(read_error)),
                }
            }
            Err(error) => return Err(error),
        };
        if record.payload() != self.payload(false) && record.payload() != self.payload(true) {
            return Err(abi::IO_ERROR);
        }
        self.generation = record.generation;
        self.alternate = record.bytes[5] != 0;
        Ok(())
    }
    pub fn toggle(&mut self, parent: Pair) -> Result<(), u64> {
        // An unresolved operation must be reconciled explicitly before mutation.
        if self.error != 0 {
            return Err(abi::DENIED);
        }
        let alternate = !self.alternate;
        if !self.volatile {
            match self.request(
                parent,
                store::REPLACE,
                2,
                self.generation,
                &self.payload(alternate),
            ) {
                Ok(record) => self.generation = record.generation,
                Err(error) => {
                    self.error = error;
                    return Err(error);
                }
            }
        }
        self.alternate = alternate;
        Ok(())
    }
    pub fn stage(&self, parent: Pair) {
        assert_ne!(self.binding, 0);
        self.request(parent, store::BEGIN, 0, self.generation, &[])
            .unwrap();
        self.request(
            parent,
            store::STAGE_REPLACE,
            2,
            0,
            &self.payload(!self.alternate),
        )
        .unwrap();
    }
    pub fn color(&self) -> u64 {
        if self.alternate {
            0xd18b35
        } else if self.style == 0 {
            LEFT
        } else {
            RIGHT
        }
    }
    fn payload(&self, alternate: bool) -> [u8; 6] {
        [
            b'M',
            b'C',
            b'0',
            b'1',
            self.style as u8,
            u8::from(alternate),
        ]
    }
    fn exchange(&self, parent: Pair, words: wire::Packet) -> Result<wire::Packet, u64> {
        if !self.transport_alive.get() {
            return Err(abi::PEER_CLOSED);
        }
        let mut bytes = [0; 64];
        let result = parent
            .0
            .send(&wire::encode(words))
            .and_then(|()| parent.1.receive_until(&mut bytes, time::after(300)?))
            .map_err(|e| e.0 as u64)
            .and_then(|n| wire::decode(&bytes[..n]).ok_or(abi::IO_ERROR));
        // No correlation ID exists on this stream. After transport failure, never
        // mistake a late response for a new request. Replacing the app is required.
        if result.is_err() {
            self.transport_alive.set(false);
        }
        result
    }
    fn request(
        &self,
        parent: Pair,
        op: u64,
        object: u64,
        generation: u64,
        payload: &[u8],
    ) -> Result<store::Record, u64> {
        let opcode = (self.binding << wire::STORAGE_SCOPE_BITS) | (wire::STORAGE_BASE + op);
        let bytes = store::request(opcode, object, generation, payload).unwrap();
        #[cfg(feature = "trace-storage")]
        time::storage_trace("app-send", self.style, op);
        let result = self
            .exchange(parent, wire::decode(&bytes).unwrap())
            .and_then(|reply| store::Record::decode(&wire::encode(reply)));
        #[cfg(feature = "trace-storage")]
        time::storage_trace("app-reply", self.style, op);
        result
    }
}
