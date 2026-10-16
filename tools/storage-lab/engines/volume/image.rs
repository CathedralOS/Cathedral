//! Locked regular host images only. Creation refuses existing paths; opening
//! never creates, truncates or resizes. The volume adapter cannot resize the image.
use redb::StorageBackend;
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    path::Path,
    sync::Mutex,
};

#[derive(Debug)]
pub struct Image(Mutex<File>);
impl Image {
    pub fn create(path: &Path, bytes: u64) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(path)?;
        file.try_lock().map_err(io::Error::other)?;
        file.set_len(bytes)?;
        file.sync_all()?;
        Ok(Self(Mutex::new(file)))
    }
    pub fn open(path: &Path) -> io::Result<Self> {
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        if !file.metadata()?.is_file() {
            return Err(io::Error::other("only regular lab images are supported"));
        }
        file.try_lock().map_err(io::Error::other)?;
        Ok(Self(Mutex::new(file)))
    }
}
impl StorageBackend for Image {
    fn len(&self) -> io::Result<u64> {
        Ok(self.0.lock().unwrap().metadata()?.len())
    }
    fn read(&self, offset: u64, bytes: &mut [u8]) -> io::Result<()> {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start(offset))?;
        file.read_exact(bytes)
    }
    fn write(&self, offset: u64, bytes: &[u8]) -> io::Result<()> {
        let mut file = self.0.lock().unwrap();
        let image_bytes = file.metadata()?.len();
        if offset
            .checked_add(bytes.len() as u64)
            .is_none_or(|end| end > image_bytes)
        {
            return Err(io::Error::other("write outside fixed image"));
        }
        file.seek(SeekFrom::Start(offset))?;
        file.write_all(bytes)
    }
    fn sync_data(&self) -> io::Result<()> {
        self.0.lock().unwrap().sync_all()
    }
    fn set_len(&self, _: u64) -> io::Result<()> {
        Err(io::Error::other("fixed image cannot resize"))
    }
}
