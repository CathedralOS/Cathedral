//! Explicit assigned range, independent of partition discovery. All translations
//! are checked before IO; this layer never resizes the underlying device.
use super::header::{self, BLOCK, Header};
use redb::StorageBackend;
use std::{io, sync::Arc};

pub fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[derive(Clone, Debug)]
pub struct Region {
    device: Arc<dyn StorageBackend>,
    start: u64,
    pub bytes: u64,
}
impl Region {
    pub fn new(device: Arc<dyn StorageBackend>, start: u64, bytes: u64) -> io::Result<Self> {
        header::geometry(bytes).map_err(io::Error::other)?;
        let device_bytes = device.len()?;
        if !start.is_multiple_of(BLOCK)
            || start
                .checked_add(bytes)
                .is_none_or(|end| end > device_bytes)
        {
            return Err(invalid("assigned region outside device or unaligned"));
        }
        Ok(Self {
            device,
            start,
            bytes,
        })
    }
    fn offset(&self, offset: u64, bytes: usize) -> io::Result<u64> {
        if offset
            .checked_add(bytes as u64)
            .is_none_or(|end| end > self.bytes)
        {
            return Err(invalid("IO exceeds assigned region"));
        }
        self.start
            .checked_add(offset)
            .ok_or_else(|| invalid("physical offset overflow"))
    }
    pub fn read(&self, offset: u64, bytes: &mut [u8]) -> io::Result<()> {
        self.device.read(self.offset(offset, bytes.len())?, bytes)
    }
    pub fn write(&self, offset: u64, bytes: &[u8]) -> io::Result<()> {
        self.device.write(self.offset(offset, bytes.len())?, bytes)
    }
    pub fn sync(&self) -> io::Result<()> {
        self.device.sync_data()
    }
    pub fn blank(&self) -> io::Result<()> {
        let mut bytes = [0; 64 * 1024];
        for offset in (0..self.bytes).step_by(bytes.len()) {
            let count = (self.bytes - offset).min(bytes.len() as u64) as usize;
            self.read(offset, &mut bytes[..count])?;
            if bytes[..count].iter().any(|b| *b != 0) {
                return Err(invalid("format refuses nonblank assigned region"));
            }
        }
        Ok(())
    }
    pub fn headers(&self) -> io::Result<Header> {
        let mut copies = [None, None];
        for (i, copy) in copies.iter_mut().enumerate() {
            let mut bytes = [0; BLOCK as usize];
            self.read(i as u64 * BLOCK, &mut bytes)?;
            *copy = Header::decode(&bytes).map_err(io::Error::other)?;
        }
        let header = header::select(&copies).map_err(io::Error::other)?;
        if header.region_bytes != self.bytes {
            return Err(invalid("envelope does not match assigned region"));
        }
        Ok(header)
    }
    pub fn publish_header(&self, header: &Header) -> io::Result<()> {
        let bytes = header.encode();
        // Publish the alternate first, then restore redundancy before returning.
        for slot in [header.generation % 2, 1 - header.generation % 2] {
            self.write(slot * BLOCK, &bytes)?;
            self.sync()?;
        }
        Ok(())
    }
}
