//! Private single-owner host file adapter for observing redb IO and denying growth.
//! Not a production locking implementation; every experiment owns a unique file.
use redb::StorageBackend;
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    path::Path,
    sync::{Arc, Mutex},
};

#[derive(Clone, Copy, Default, Debug)]
pub struct Counts {
    pub write_bytes: u64,
    pub writes: u64,
    pub syncs: u64,
}

#[derive(Debug)]
struct State {
    file: File,
    counts: Counts,
    limit: u64,
}
#[derive(Clone, Debug)]
pub struct Backend(Arc<Mutex<State>>);

impl Backend {
    pub fn open(path: &Path) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;
        Ok(Self(Arc::new(Mutex::new(State {
            file,
            counts: Counts::default(),
            limit: u64::MAX,
        }))))
    }
    pub fn reset(&self) {
        self.0.lock().unwrap().counts = Counts::default();
    }
    pub fn counts(&self) -> Counts {
        self.0.lock().unwrap().counts
    }
    pub fn limit(&self, bytes: u64) {
        self.0.lock().unwrap().limit = bytes;
    }
}

impl StorageBackend for Backend {
    fn len(&self) -> io::Result<u64> {
        Ok(self.0.lock().unwrap().file.metadata()?.len())
    }
    fn read(&self, offset: u64, out: &mut [u8]) -> io::Result<()> {
        let mut state = self.0.lock().unwrap();
        state.file.seek(SeekFrom::Start(offset))?;
        state.file.read_exact(out)
    }
    fn set_len(&self, len: u64) -> io::Result<()> {
        let state = self.0.lock().unwrap();
        if len > state.limit {
            return Err(io::Error::other("lab capacity exhausted"));
        }
        state.file.set_len(len)
    }
    fn sync_data(&self) -> io::Result<()> {
        let mut state = self.0.lock().unwrap();
        state.file.sync_all()?;
        state.counts.syncs += 1;
        Ok(())
    }
    fn write(&self, offset: u64, data: &[u8]) -> io::Result<()> {
        let mut state = self.0.lock().unwrap();
        if offset
            .checked_add(data.len() as u64)
            .is_none_or(|end| end > state.limit)
        {
            return Err(io::Error::other("lab capacity exhausted"));
        }
        state.file.seek(SeekFrom::Start(offset))?;
        state.file.write_all(data)?;
        state.counts.write_bytes += data.len() as u64;
        state.counts.writes += 1;
        Ok(())
    }
}
