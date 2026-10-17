//! App schema over its matrix-local object capability. Persist before
//! presenting; revoked bindings cannot save, and a replacement reads only
//! committed state. The color lives in one object under the binding's home;
//! its number is discovered by listing the home, never assumed.
use super::*;
use cathedral_platform_contracts::storage as store;
use cathedral_user_runtime::{memory::Private, storage::Scope};
pub(crate) struct State {
    scope: Scope,
    object: u64,
    pub revision: u64,
    style: u64,
    pub alternate: bool,
    pub error: u64,
    volatile: bool,
}
impl State {
    pub fn open(pair: Pair, style: u64) -> Self {
        let mut state = Self {
            scope: Scope::new(pair),
            object: 0,
            revision: 0,
            style,
            alternate: false,
            error: 0,
            // Only the explicitly diskless sibling probe may change volatile state.
            volatile: style == 2,
        };
        let _ = state.reload();
        state
    }
    /// Explicit reconciliation: acquire only the currently granted binding,
    /// read first, and create only after observing an empty home. Never replay
    /// an uncertain toggle or replace confirmed state on failed reads.
    pub fn reload(&mut self) -> Result<(), u64> {
        let result = self.load();
        self.error = result.err().unwrap_or(0);
        result
    }
    fn load(&mut self) -> Result<(), u64> {
        match self.scope.bind() {
            Ok(_) => (),
            Err(abi::DENIED) if self.volatile => return Ok(()),
            Err(error) => return Err(error),
        }
        self.volatile = false;
        let home = self.scope.home();
        let listing = self.exchange(store::Request::new(store::LIST, home, 0, 0, 0), None)?;
        let words = store::unpack(&listing).ok_or(abi::IO_ERROR)?;
        if words[0] != 0 {
            return Err(words[0]);
        }
        if words[1] == 0 {
            // Even an error may hide a committed creation. A successful listing
            // afterwards settles the outcome; otherwise leave it unresolved.
            let created = self.publish(0, 0, false);
            let listing = self.exchange(store::Request::new(store::LIST, home, 0, 0, 0), None)?;
            let words = store::unpack(&listing).ok_or(abi::IO_ERROR)?;
            if words[0] != 0 || words[1] == 0 {
                return Err(created.err().unwrap_or(abi::IO_ERROR));
            }
            self.object = words[2];
        } else {
            self.object = words[2];
        }
        let reply = self.exchange(store::Request::new(store::READ, self.object, 0, 0, 6), None)?;
        let record = store::Reply::decode(&reply)?;
        let payload = &reply[32..38];
        if record.length != 6 || (payload != self.payload(false) && payload != self.payload(true)) {
            return Err(abi::IO_ERROR);
        }
        self.revision = record.revision;
        self.alternate = payload[5] != 0;
        Ok(())
    }
    pub fn toggle(&mut self) -> Result<(), u64> {
        // An unresolved operation must be reconciled explicitly before mutation.
        if self.error != 0 {
            return Err(abi::DENIED);
        }
        let alternate = !self.alternate;
        if !self.volatile
            && let Err(error) = self.publish(self.object, self.revision, alternate)
        {
            self.error = error;
            return Err(error);
        }
        self.alternate = alternate;
        Ok(())
    }
    /// Object zero creates the color under the home; otherwise the object is
    /// replaced at its expected revision.
    fn publish(&mut self, object: u64, expected: u64, alternate: bool) -> Result<(), u64> {
        let mut pages = Private::allocate(1).map_err(|e| e.0 as u64)?;
        pages.bytes_mut()[..6].copy_from_slice(&self.payload(alternate));
        let attachment = pages.freeze().map_err(|e| e.0 as u64)?;
        let parent = if object == 0 { self.scope.home() } else { 0 };
        let reply = self.exchange(
            store::Request::new(store::PUBLISH, object, expected, parent, 6),
            Some(&attachment),
        )?;
        let reply = store::Reply::decode(&reply)?;
        self.object = reply.object;
        self.revision = reply.revision;
        Ok(())
    }
    /// Every exchange goes through here so the storage trace sees each one.
    fn exchange(
        &mut self,
        request: store::Request,
        attachment: Option<&cathedral_user_runtime::memory::Object>,
    ) -> Result<[u8; 64], u64> {
        #[cfg(feature = "trace-storage")]
        time::storage_trace("app-send", self.style, request.op);
        let result = self
            .scope
            .request(request, attachment)
            .map(|(reply, _)| reply);
        #[cfg(feature = "trace-storage")]
        time::storage_trace("app-reply", self.style, request.op);
        result
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
}
