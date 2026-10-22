//! File-length semantics inside fixed capacity. The engine's layout length is
//! durably mirrored in the envelope; the underlying image size never changes and
//! no range is zeroed, because the engine never reads space it has not written.
use super::{
    header::{DATA_START, Header},
    region::{Region, invalid},
};
use redb::StorageBackend;
use std::{
    io,
    sync::{Arc, Mutex},
};

#[derive(Debug)]
struct State {
    header: Header,
    failed: bool,
}
#[derive(Clone, Debug)]
pub struct Backend {
    region: Region,
    state: Arc<Mutex<State>>,
}
impl Backend {
    pub fn new(region: Region, header: Header) -> Self {
        Self {
            region,
            state: Arc::new(Mutex::new(State {
                header,
                failed: false,
            })),
        }
    }
    fn healthy(state: &State) -> io::Result<()> {
        if state.failed {
            Err(io::Error::other("volume backend failed; reopen required"))
        } else {
            Ok(())
        }
    }
    fn checked(state: &State, offset: u64, bytes: usize) -> io::Result<()> {
        Self::healthy(state)?;
        if offset
            .checked_add(bytes as u64)
            .is_none_or(|end| end > state.header.length)
        {
            return Err(invalid("engine IO exceeds logical extent"));
        }
        Ok(())
    }
    fn finish(state: &mut State, result: io::Result<()>) -> io::Result<()> {
        if result.is_err() {
            state.failed = true;
        }
        result
    }
    pub fn mark_ready(&self) -> io::Result<()> {
        let mut state = self.state.lock().unwrap();
        Self::healthy(&state)?;
        if state.header.length == 0 {
            return Err(invalid("empty engine cannot become ready"));
        }
        let mut header = state.header.clone();
        header.ready = true;
        header.generation = header
            .generation
            .checked_add(1)
            .ok_or_else(|| invalid("generation exhausted"))?;
        Self::finish(&mut state, self.region.publish_header(&header))?;
        state.header = header;
        Ok(())
    }
}
impl StorageBackend for Backend {
    fn len(&self) -> io::Result<u64> {
        let s = self.state.lock().unwrap();
        Self::healthy(&s)?;
        Ok(s.header.length)
    }
    fn read(&self, offset: u64, bytes: &mut [u8]) -> io::Result<()> {
        let state = self.state.lock().unwrap();
        Self::checked(&state, offset, bytes.len())?;
        self.region.read(DATA_START + offset, bytes)
    }
    fn write(&self, offset: u64, bytes: &[u8]) -> io::Result<()> {
        let mut state = self.state.lock().unwrap();
        Self::checked(&state, offset, bytes.len())?;
        Self::finish(&mut state, self.region.write(DATA_START + offset, bytes))
    }
    fn sync_data(&self) -> io::Result<()> {
        let mut state = self.state.lock().unwrap();
        Self::healthy(&state)?;
        Self::finish(&mut state, self.region.sync())
    }
    fn set_len(&self, length: u64) -> io::Result<()> {
        let mut state = self.state.lock().unwrap();
        Self::healthy(&state)?;
        if length > state.header.capacity() {
            return Err(io::Error::other(format!(
                "volume capacity exhausted: requested {length}, capacity {}",
                state.header.capacity()
            )));
        }
        if length == state.header.length {
            return Ok(());
        }
        if state.header.ready && length == 0 {
            return Err(invalid("refuse emptying a ready engine"));
        }
        let mut header = state.header.clone();
        header.length = length;
        header.generation = header
            .generation
            .checked_add(1)
            .ok_or_else(|| invalid("generation exhausted"))?;
        let result = (|| {
            // Earlier engine writes reach the device before the length that keeps them.
            self.region.sync()?;
            self.region.publish_header(&header)
        })();
        Self::finish(&mut state, result)?;
        state.header = header;
        Ok(())
    }
}
