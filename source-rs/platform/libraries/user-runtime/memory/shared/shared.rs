// Live single-writer pages. Never lend Rust references into peer-mutable memory.
// Transfer installs only read authority; access copies aligned machine words.
use super::{Private, Region, raw};
use crate::{Error, abi, arch};
use cathedral_kernel_contracts::memory::PAGE_BYTES;
pub struct Writer(Region);
pub struct Reader(Region);
impl Private {
    pub fn share(self) -> Result<Writer, Error> {
        raw::result(arch::call(abi::MEMORY_SHARE_MUTABLE, self.0.handle, 0))?;
        Ok(Writer(self.0))
    }
}
impl Writer {
    pub fn allocate(pages: usize) -> Result<Self, Error> {
        Private::allocate(pages)?.share()
    }
    pub fn handle(&self) -> u64 {
        self.0.handle
    }
    pub fn words(&self) -> usize {
        self.0.bytes / 8
    }
    pub fn load(&self, index: usize) -> Result<u64, Error> {
        load(&self.0, index)
    }
    pub fn store(&mut self, index: usize, value: u64) -> Result<(), Error> {
        let address = address(&self.0, index)?;
        // SAFETY: This wrapper retains the producer's aligned writable mapping.
        unsafe { arch::shared::store(address, value) };
        Ok(())
    }
    pub fn unmap(&mut self) -> Result<(), Error> {
        unmap(&mut self.0)
    }
    pub fn map(&mut self) -> Result<(), Error> {
        map(&mut self.0)
    }
}
impl Reader {
    pub fn handle(&self) -> u64 {
        self.0.handle
    }
    pub fn words(&self) -> usize {
        self.0.bytes / 8
    }
    pub fn load(&self, index: usize) -> Result<u64, Error> {
        load(&self.0, index)
    }
    pub fn unmap(&mut self) -> Result<(), Error> {
        unmap(&mut self.0)
    }
    pub fn map(&mut self) -> Result<(), Error> {
        map(&mut self.0)
    }
    fn received(handle: u64) -> Result<Self, Error> {
        let mut reader = Self(Region {
            handle,
            address: 0,
            bytes: 0,
        });
        reader.0.bytes = raw::pages(handle)? * PAGE_BYTES;
        reader.map()?;
        Ok(reader)
    }
}
fn address(region: &Region, index: usize) -> Result<u64, Error> {
    if region.address == 0 {
        return Err(Error(abi::BAD_ADDRESS as i64));
    }
    if index >= region.bytes / 8 {
        return Err(Error(abi::INVALID_ARGUMENT as i64));
    }
    Ok(region.address + index as u64 * 8)
}
fn load(region: &Region, index: usize) -> Result<u64, Error> {
    let address = address(region, index)?;
    // SAFETY: Retained aligned readable mapping. No reference, slice or validity
    // assumption about concurrent peer bytes enters Rust; every u64 is valid.
    Ok(unsafe { arch::shared::load(address) })
}
fn map(region: &mut Region) -> Result<(), Error> {
    region.address = raw::result(arch::call(abi::MEMORY_MAP_SHARED, region.handle, 0))?;
    Ok(())
}
fn unmap(region: &mut Region) -> Result<(), Error> {
    raw::result(arch::call(abi::MEMORY_UNMAP_OBJECT, region.handle, 0))?;
    region.address = 0;
    Ok(())
}
