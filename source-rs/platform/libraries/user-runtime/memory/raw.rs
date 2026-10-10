//! Raw handles for ABI tests and runtimes implementing their own ownership.
use crate::{Error, abi, arch};
pub fn allocate(pages: usize) -> Result<u64, Error> {
    result(arch::call(abi::MEMORY_ALLOCATE, pages as u64, 0))
}
pub fn address(handle: u64) -> Result<u64, Error> {
    result(arch::call(abi::MEMORY_ADDRESS, handle, 0))
}
pub fn pages(handle: u64) -> Result<usize, Error> {
    result(arch::call(abi::MEMORY_PAGES, handle, 0)).map(|n| n as usize)
}
pub fn accept(handle: u64) -> Result<u64, Error> {
    result(arch::call(abi::MEMORY_MAP, handle, 0))
}
pub fn accept_from(handle: u64, link: u64) -> Result<u64, Error> {
    result(arch::call(abi::MEMORY_MAP, handle, link))
}
/// # Safety
/// End all write borrows of this region before sealing it.
pub unsafe fn seal(handle: u64, link: u64) -> Result<(), Error> {
    result(arch::call(abi::MEMORY_SEAL, handle, link)).map(|_| ())
}
/// # Safety
/// End every borrow of the caller's mapping before releasing it. Do not release
/// a handle owned by a live safe wrapper.
pub unsafe fn release(handle: u64) -> Result<(), Error> {
    result(arch::call(abi::MEMORY_RELEASE, handle, 0)).map(|_| ())
}
fn result(value: u64) -> Result<u64, Error> {
    if (value as i64) < 0 {
        Err(Error(value as i64))
    } else {
        Ok(value)
    }
}
