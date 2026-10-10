#![no_std]
#![forbid(unsafe_code)]
//! Private catalogs: validate root-local changes, commit a snapshot, recover complete roots.
use cathedral_contracts::{block::Device, storage::Record, user as abi};
pub mod catalog;
mod format;
mod legacy;
pub mod protocol;
mod recovery;
use catalog::{Catalog, Change};
pub struct Store<D> {
    disk: D,
    catalog: Catalog,
    slot: u64,
    healthy: bool,
}
impl<D: Device> Store<D> {
    pub fn open(mut disk: D) -> Result<Self, u64> {
        let (slot, record) = recovery::open(&mut disk)?;
        Ok(Self {
            disk,
            catalog: record,
            slot,
            healthy: true,
        })
    }
    pub fn needs_reopen(&self) -> bool {
        !self.healthy
    }
    pub fn root(&self, root: usize) -> Result<&catalog::Root, u64> {
        if !self.healthy {
            return Err(abi::IO_ERROR);
        }
        self.catalog.roots.get(root).ok_or(abi::DENIED)
    }
    pub fn read(&self, root: usize, object: u64) -> Result<Record, u64> {
        self.root(root)?.read(object)
    }
    /// Root revision is the compare-and-swap boundary, including deleted/recreated objects.
    /// Hook phases: before body, body durable, commit written, commit durable.
    pub fn transact(
        &mut self,
        root: usize,
        expected: u64,
        changes: &[Change],
        mut hook: impl FnMut(u8),
    ) -> Result<Record, u64> {
        let changed = self.root(root)?.changed(expected, changes)?;
        let mut record = self.catalog;
        record.roots[root] = changed;
        record.sequence = record.sequence.checked_add(1).ok_or(abi::IO_ERROR)?;
        let mut body = [0; 512];
        let mut commit = [0; 512];
        format::encode(&record, &mut body, &mut commit);
        let slot = 1 - self.slot;
        self.healthy = false;
        hook(1);
        self.disk.write(slot * 2, &body)?;
        self.disk.flush()?;
        hook(2);
        self.disk.write(slot * 2 + 1, &commit)?;
        hook(3);
        self.disk.flush()?;
        hook(4);
        self.slot = slot;
        self.catalog = record;
        self.healthy = true;
        Ok(changed.list())
    }
}
#[cfg(test)]
mod tests;
