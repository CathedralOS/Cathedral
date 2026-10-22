use super::backend::Backend;
use cathedral_block_device::{Device, SECTOR};
use cathedral_storage_volume::{
    BlockRegion,
    header::{DATA_START, Header},
    inspect,
};
use redb::StorageBackend;
use std::{
    sync::{Arc, Mutex},
    vec,
    vec::Vec,
};

#[derive(Clone)]
struct Disk(Arc<Mutex<State>>);
struct State {
    bytes: Vec<u8>,
    calls: usize,
    fail: usize,
}
impl Disk {
    fn call(&self) -> Result<std::sync::MutexGuard<'_, State>, u64> {
        let mut s = self.0.lock().unwrap();
        s.calls += 1;
        if s.calls == s.fail {
            return Err(77);
        }
        Ok(s)
    }
}
impl Device for Disk {
    fn sectors(&self) -> u64 {
        (self.0.lock().unwrap().bytes.len() / SECTOR) as u64
    }
    fn read(&mut self, sector: u64, out: &mut [u8; SECTOR]) -> Result<(), u64> {
        let s = self.call()?;
        let n = sector as usize * SECTOR;
        out.copy_from_slice(&s.bytes[n..n + SECTOR]);
        Ok(())
    }
    fn write(&mut self, sector: u64, data: &[u8; SECTOR]) -> Result<(), u64> {
        let mut s = self.call()?;
        let n = sector as usize * SECTOR;
        s.bytes[n..n + SECTOR].copy_from_slice(data);
        Ok(())
    }
    fn flush(&mut self) -> Result<(), u64> {
        drop(self.call()?);
        Ok(())
    }
}
fn fixture() -> (Backend<Disk>, Disk) {
    let header = Header {
        id: [1; 16],
        generation: 4,
        region_bytes: 4 * 1024 * 1024,
        length: 8192,
        ready: true,
    };
    let disk = Disk(Arc::new(Mutex::new(State {
        bytes: vec![0xa7; header.region_bytes as usize + 1024],
        calls: 0,
        fail: 0,
    })));
    (Backend::new(disk.clone(), 512, header), disk)
}
#[test]
fn growth_publishes_only_the_envelope_and_invalid_requests_do_no_io() {
    let (backend, mut disk) = fixture();
    for length in [0, u64::MAX] {
        assert!(backend.set_len(length).is_err());
    }
    assert!(backend.write(8191, &[1, 2]).is_err());
    assert!(backend.read(u64::MAX, &mut [0]).is_err());
    assert_eq!(disk.0.lock().unwrap().calls, 0);
    backend.set_len(16385).unwrap();
    assert_eq!(backend.len().unwrap(), 16385);
    // One flush of earlier engine writes, then each envelope copy written and flushed.
    assert_eq!(disk.0.lock().unwrap().calls, 1 + 2 * (8 + 1));
    let header = inspect(&mut BlockRegion::new(&mut disk, 512, 4 * 1024 * 1024).unwrap()).unwrap();
    assert_eq!(header.length, 16385);
    backend.set_len(8193).unwrap();
    assert!(backend.read(8193, &mut [0]).is_err());
    backend.set_len(16385).unwrap();
    let s = disk.0.lock().unwrap();
    let start = 512 + DATA_START as usize;
    // Engine bytes are never zeroed or otherwise touched by a length change.
    assert!(s.bytes[start..start + 16385].iter().all(|b| *b == 0xa7));
    assert!(
        s.bytes[..512]
            .iter()
            .chain(s.bytes[512 + 4 * 1024 * 1024..].iter())
            .all(|b| *b == 0xa7)
    );
}
#[test]
fn every_growth_io_failure_poison_stops_all_further_operations() {
    let (backend, disk) = fixture();
    backend.set_len(16385).unwrap();
    let calls = disk.0.lock().unwrap().calls;
    for fail in 1..=calls {
        let (backend, disk) = fixture();
        disk.0.lock().unwrap().fail = fail;
        assert!(backend.set_len(16385).is_err(), "call {fail}");
        assert!(backend.len().is_err());
        assert!(backend.read(0, &mut [0]).is_err());
        assert!(backend.write(0, &[1]).is_err());
        assert!(backend.sync_data().is_err());
        assert!(backend.set_len(8192).is_err());
        assert_eq!(disk.0.lock().unwrap().calls, fail);
    }
}

#[test]
fn failed_write_or_flush_retires_backend_after_temporary_region_is_gone() {
    for write in [true, false] {
        let (backend, disk) = fixture();
        disk.0.lock().unwrap().fail = 1;
        let result = if write {
            backend.write(0, &[7])
        } else {
            backend.sync_data()
        };
        assert!(result.is_err());
        assert!(backend.read(0, &mut [0]).is_err());
        assert!(backend.write(0, &[1]).is_err());
        assert!(backend.sync_data().is_err());
        assert_eq!(disk.0.lock().unwrap().calls, 1);
    }
}

#[test]
fn only_capacity_refusal_has_the_recoverable_marker() {
    let (backend, disk) = fixture();
    assert!(super::capacity_refusal(
        backend.set_len(u64::MAX).unwrap_err().into()
    ));
    assert!(!super::capacity_refusal(
        backend.set_len(0).unwrap_err().into()
    ));
    assert_eq!(disk.0.lock().unwrap().calls, 0);
    disk.0.lock().unwrap().fail = 1;
    assert!(!super::capacity_refusal(
        backend.set_len(16384).unwrap_err().into()
    ));
}
