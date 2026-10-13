//! Sealed object custody is independent of local mapping. IPC copies read rights;
//! forwarding does not require mapping or copying any pixel/sample bytes.
use super::raw::result;
use super::*;
use crate::{abi, arch};
pub struct Object(Region);
impl Private {
    pub fn freeze(self) -> Result<Object, Error> {
        result(arch::call(abi::MEMORY_FREEZE, self.0.handle, 0))?;
        Ok(Object(self.0))
    }
}
impl Object {
    pub(crate) fn received(handle: u64) -> Result<Self, Error> {
        // Own the handle before a fallible query so every error still releases it.
        let mut object = Self(Region {
            handle,
            address: 0,
            bytes: 0,
        });
        object.0.bytes = raw::pages(handle)? * PAGE_BYTES;
        Ok(object)
    }
    pub fn handle(&self) -> u64 {
        self.0.handle
    }
    pub fn len(&self) -> usize {
        self.0.bytes
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn map(&mut self) -> Result<&[u8], Error> {
        if self.0.address == 0 {
            self.0.address = result(arch::call(abi::MEMORY_MAP_OBJECT, self.0.handle, 0))?;
        }
        Ok(self.bytes().unwrap())
    }
    pub fn bytes(&self) -> Option<&[u8]> {
        if self.0.address == 0 {
            return None;
        }
        // SAFETY: Sealed backing pinned by this unique wrapper; unmap/drop require all borrows ended.
        Some(unsafe { core::slice::from_raw_parts(self.0.address as *const u8, self.0.bytes) })
    }
    pub fn unmap(&mut self) -> Result<(), Error> {
        result(arch::call(abi::MEMORY_UNMAP_OBJECT, self.0.handle, 0))?;
        self.0.address = 0;
        Ok(())
    }
}
