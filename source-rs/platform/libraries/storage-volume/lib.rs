#![no_std]
#![forbid(unsafe_code)]
//! Experimental volume boundary. Inspect the envelope through a bounded block
//! region; header.rs owns the shared codec, region.rs owns byte/sector translation.
//! This is not a database engine or a partition/authority discovery mechanism.
pub mod header;
mod region;
use cathedral_block_device::Device;
use header::{BLOCK, Header};
pub use region::BlockRegion;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid(&'static str),
    Device(u64),
    Failed,
}
impl core::fmt::Display for Error {
    fn fmt(&self, out: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Invalid(reason) => out.write_str(reason),
            Self::Device(code) => write!(out, "block device error {code}"),
            Self::Failed => out.write_str("failed region; device recovery and reopen required"),
        }
    }
}
impl core::error::Error for Error {}

/// Inspect only: no engine initialization, repair, or writes on any path.
/// Formatting state is returned to the caller; it does not imply engine validity.
pub fn inspect<D: Device>(region: &mut BlockRegion<'_, D>) -> Result<Header, Error> {
    let mut copies = [None, None];
    let mut bytes = [0; BLOCK as usize];
    for (slot, copy) in copies.iter_mut().enumerate() {
        region.read(slot as u64 * BLOCK, &mut bytes)?;
        *copy = Header::decode(&bytes)?;
    }
    let header = header::select(&copies)?;
    if header.region_bytes != region.bytes() {
        return Err(Error::Invalid("envelope does not match assigned region"));
    }
    Ok(header)
}

#[cfg(test)]
extern crate std;
#[cfg(test)]
mod tests;
