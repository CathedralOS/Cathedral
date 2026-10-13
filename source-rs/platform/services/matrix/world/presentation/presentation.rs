// A connection owns one bounded unpublished batch. Payloads hold read authority;
// clearing a batch drops it without touching the previously published scene.
use cathedral_contracts::{matrix as wire, user as abi};

pub struct Batch<T> {
    pub incarnation: u64,
    pub epoch: u64,
    last: u64,
    revision: Option<u64>,
    failed: Option<u64>,
    pub entries: [Option<(wire::Packet, T)>; wire::PRESENT_LIMIT],
}
impl<T> Default for Batch<T> {
    fn default() -> Self {
        Self {
            incarnation: 0,
            epoch: 0,
            last: 0,
            revision: None,
            failed: None,
            entries: [const { None }; wire::PRESENT_LIMIT],
        }
    }
}
impl<T> Batch<T> {
    pub fn active(&self) -> bool {
        self.revision.is_some()
    }
    pub fn bind(&mut self, incarnation: u64, epoch: u64) {
        if (self.incarnation, self.epoch) != (incarnation, epoch) {
            *self = Self::default();
            self.incarnation = incarnation;
            self.epoch = epoch;
        }
    }
    pub fn begin(&mut self, revision: u64) -> Result<(), u64> {
        if self.revision == Some(revision) {
            return Ok(());
        }
        if self.active() {
            return Err(abi::BUSY);
        }
        if revision == 0 || revision <= self.last {
            return Err(abi::DENIED);
        }
        self.revision = Some(revision);
        self.failed = None;
        Ok(())
    }
    pub fn poison(&mut self, status: u64) {
        if self.active() {
            self.failed.get_or_insert(status);
        }
    }
    pub fn stage(&mut self, words: wire::Packet, payload: T) -> Result<(), u64> {
        let result = (|| {
            if !self.active() {
                return Err(abi::DENIED);
            }
            if let Some(status) = self.failed {
                return Err(status);
            }
            if self.entries.iter().flatten().any(|(p, _)| p[2] == words[2]) {
                return Err(abi::INVALID_ARGUMENT);
            }
            let entry = self
                .entries
                .iter_mut()
                .find(|e| e.is_none())
                .ok_or(abi::NO_MEMORY)?;
            *entry = Some((words, payload));
            Ok(())
        })();
        if let Err(status) = result {
            self.poison(status);
        }
        result
    }
    /// true requires publication; false is the latest successful commit's retry.
    pub fn prepare(&self, revision: u64) -> Result<bool, u64> {
        if !self.active() && revision != 0 && revision == self.last {
            return Ok(false);
        }
        if self.revision != Some(revision) {
            return Err(abi::DENIED);
        }
        if let Some(status) = self.failed {
            return Err(status);
        }
        Ok(true)
    }
    pub fn committed(&mut self, revision: u64) {
        assert_eq!(self.prepare(revision), Ok(true));
        self.last = revision;
        self.clear();
    }
    pub fn abort(&mut self, revision: u64) -> Result<(), u64> {
        if self.revision != Some(revision) {
            return Err(abi::DENIED);
        }
        self.clear();
        Ok(())
    }
    fn clear(&mut self) {
        self.entries = [const { None }; wire::PRESENT_LIMIT];
        self.revision = None;
        self.failed = None;
    }
}
