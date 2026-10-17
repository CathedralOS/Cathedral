#![no_std]
#![forbid(unsafe_code)]
//! Existing private-catalog semantics over atomic content publication. One root
//! snapshot per private content object; one provider owns the cache and writer.
//! This experiment retains only the current snapshot, without changing matrix IPC.
mod snapshot;
use cathedral_database_objects::{Error, Store};
use cathedral_kernel_contracts::user as abi;
use cathedral_object_store::{
    catalog::{Change, Root},
    protocol::Backend,
};
use cathedral_platform_contracts::storage::{self as wire, Record};
pub struct Catalog<'a> {
    store: &'a Store<'a>,
    roots: [Root; wire::ROOTS],
    healthy: bool,
}
impl<'a> Catalog<'a> {
    pub fn open(store: &'a Store<'a>) -> Result<Self, Error> {
        let mut roots = [Root::default(); wire::ROOTS];
        for (root, value) in roots.iter_mut().enumerate() {
            let head = store.read(root as u8, 0, 0, &mut [])?;
            if head.revision == 0 {
                continue;
            }
            if head.len != snapshot::SIZE {
                return Err(Error::Corrupt);
            }
            let mut bytes = [0; snapshot::SIZE];
            store.read(root as u8, head.revision, 0, &mut bytes)?;
            *value = snapshot::decode(&bytes)?;
            if value.generation != head.revision {
                return Err(Error::Corrupt);
            }
        }
        Ok(Self {
            store,
            roots,
            healthy: true,
        })
    }
    pub fn needs_reopen(&self) -> bool {
        !self.healthy
    }
    fn collect(&self, root: usize) -> Result<(), Error> {
        for _ in 0..64 {
            if !self.store.collect(root as u8)?.more {
                return Ok(());
            }
        }
        Err(Error::Capacity)
    }
    fn error(&mut self, error: Error) -> u64 {
        match error {
            Error::Conflict => abi::WOULD_BLOCK,
            Error::Capacity => abi::NO_MEMORY,
            _ => {
                self.healthy = false;
                abi::IO_ERROR
            }
        }
    }
}
impl Backend for Catalog<'_> {
    fn root(&self, root: usize) -> Result<&Root, u64> {
        if !self.healthy {
            return Err(abi::IO_ERROR);
        }
        self.roots.get(root).ok_or(abi::DENIED)
    }
    fn transact(
        &mut self,
        root: usize,
        expected: u64,
        changes: &[Change],
        mut hook: impl FnMut(u8),
    ) -> Result<Record, u64> {
        let next = self.root(root)?.changed(expected, changes)?;
        // Finish bounded old garbage before allocating the next content revision.
        // Current head remains intact even if maintenance fails or power is lost.
        self.collect(root).map_err(|e| self.error(e))?;
        hook(1);
        self.store
            .publish(root as u8, expected, &snapshot::encode(&next))
            .map_err(|e| self.error(e))?;
        self.roots[root] = next;
        hook(4);
        Ok(next.list())
    }
}
#[cfg(all(test, feature = "host"))]
extern crate std;
#[cfg(all(test, feature = "host"))]
mod tests;
