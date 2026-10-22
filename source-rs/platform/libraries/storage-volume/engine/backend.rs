//! The engine owns its whole assigned region from format. Its layout length is
//! mirrored in the envelope because redb requires the backend length to match
//! the layout it stored; changing it writes nothing but the envelope. The engine
//! never reads space it has not written and rebuilds its allocators from its
//! trees after an unclean stop, so no range is ever zeroed at runtime.
//! Mutation failures retire the whole backend.
use alloc::string::ToString;
use cathedral_block_device::Device;
use cathedral_storage_volume::{
    BlockRegion, Error,
    header::{BLOCK, DATA_START, Header},
};
use redb::{StorageBackend, io};
use spinning_top::Spinlock;

struct State<D> {
    device: D,
    start: u64,
    header: Header,
    failed: bool,
}
pub struct Backend<D>(Spinlock<State<D>>);
impl<D> core::fmt::Debug for Backend<D> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("BoundedBlockBackend")
    }
}
impl<D: Device> Backend<D> {
    pub fn new(device: D, start: u64, header: Header) -> Self {
        Self(Spinlock::new(State {
            device,
            start,
            header,
            failed: false,
        }))
    }
}
fn error(e: Error) -> io::Error {
    io::Error::other(e.to_string())
}
impl<D: Device> State<D> {
    fn checked(&self, offset: u64, bytes: usize) -> Result<(), io::Error> {
        if self.failed {
            return Err(io::Error::other("backend failed; recover and reopen"));
        }
        if offset
            .checked_add(bytes as u64)
            .is_none_or(|end| end > self.header.length)
        {
            return Err(io::Error::other("engine IO exceeds logical extent"));
        }
        Ok(())
    }
    fn finish(&mut self, result: Result<(), Error>) -> Result<(), io::Error> {
        if result.is_err() {
            self.failed = true;
        }
        result.map_err(error)
    }
}
impl<D: Device + Send + 'static> StorageBackend for Backend<D> {
    fn len(&self) -> Result<u64, io::Error> {
        let state = self.0.lock();
        state.checked(0, 0)?;
        Ok(state.header.length)
    }
    fn read(&self, offset: u64, bytes: &mut [u8]) -> Result<(), io::Error> {
        let mut state = self.0.lock();
        state.checked(offset, bytes.len())?;
        let start = state.start;
        let capacity = state.header.region_bytes;
        BlockRegion::new(&mut state.device, start, capacity)
            .map_err(error)?
            .read(DATA_START + offset, bytes)
            .map_err(error)
    }
    fn write(&self, offset: u64, bytes: &[u8]) -> Result<(), io::Error> {
        let mut state = self.0.lock();
        state.checked(offset, bytes.len())?;
        let start = state.start;
        let capacity = state.header.region_bytes;
        let result = BlockRegion::new(&mut state.device, start, capacity)
            .and_then(|mut r| r.write(DATA_START + offset, bytes));
        state.finish(result)
    }
    fn sync_data(&self) -> Result<(), io::Error> {
        let mut state = self.0.lock();
        state.checked(0, 0)?;
        let start = state.start;
        let capacity = state.header.region_bytes;
        let result =
            BlockRegion::new(&mut state.device, start, capacity).and_then(|mut r| r.flush());
        state.finish(result)
    }
    /// Record the engine's new layout length. Earlier engine writes are flushed
    /// first, so a shorter layout never publishes ahead of the data it keeps.
    fn set_len(&self, length: u64) -> Result<(), io::Error> {
        let mut state = self.0.lock();
        state.checked(0, 0)?;
        if length > state.header.capacity() {
            // Exact marker reserved for this pre-I/O refusal, never device errors.
            return Err(io::Error::other(crate::CAPACITY_REFUSAL));
        }
        if length == 0 {
            return Err(io::Error::other("invalid zero engine extent"));
        }
        if length == state.header.length {
            return Ok(());
        }
        let mut header = state.header.clone();
        header.length = length;
        header.generation = header
            .generation
            .checked_add(1)
            .ok_or_else(|| io::Error::other("generation exhausted"))?;
        let start = state.start;
        let result = (|| {
            let mut region = BlockRegion::new(&mut state.device, start, header.region_bytes)?;
            region.flush()?;
            let encoded = header.encode();
            for slot in [header.generation % 2, 1 - header.generation % 2] {
                region.write(slot * BLOCK, &encoded)?;
                region.flush()?;
            }
            Ok(())
        })();
        state.finish(result)?;
        state.header = header;
        Ok(())
    }
}
