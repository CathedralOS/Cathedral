//! Host file only supplies IO; the measured engine uses its actual no_std build.
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
    sync::Mutex,
};
#[derive(Debug)]
pub struct Backend(Mutex<File>);
impl Backend {
    pub fn open(path: &Path, create: bool) -> Self {
        Self(Mutex::new(
            OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(create)
                .open(path)
                .unwrap(),
        ))
    }
}
fn error(e: std::io::Error) -> redb::io::Error {
    redb::io::Error::other(e.to_string())
}
impl redb::StorageBackend for Backend {
    fn len(&self) -> Result<u64, redb::io::Error> {
        self.0
            .lock()
            .unwrap()
            .metadata()
            .map(|m| m.len())
            .map_err(error)
    }
    fn set_len(&self, n: u64) -> Result<(), redb::io::Error> {
        self.0.lock().unwrap().set_len(n).map_err(error)
    }
    fn read(&self, offset: u64, bytes: &mut [u8]) -> Result<(), redb::io::Error> {
        let mut f = self.0.lock().unwrap();
        f.seek(SeekFrom::Start(offset)).map_err(error)?;
        f.read_exact(bytes).map_err(error)
    }
    fn write(&self, offset: u64, bytes: &[u8]) -> Result<(), redb::io::Error> {
        let mut f = self.0.lock().unwrap();
        f.seek(SeekFrom::Start(offset)).map_err(error)?;
        f.write_all(bytes).map_err(error)
    }
    fn sync_data(&self) -> Result<(), redb::io::Error> {
        self.0.lock().unwrap().sync_all().map_err(error)
    }
}
