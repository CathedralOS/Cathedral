use super::*;
#[derive(Clone)]
struct Disk {
    stable: [[u8; 512]; 4],
    cache: [[u8; 512]; 4],
    operations: usize,
    fail: usize,
    tear: usize,
}
impl Disk {
    fn blank() -> Self {
        Self {
            stable: [[0; 512]; 4],
            cache: [[0; 512]; 4],
            operations: 0,
            fail: usize::MAX,
            tear: 0,
        }
    }
    fn power_loss(mut self) -> Self {
        self.cache = self.stable;
        self.operations = 0;
        self.fail = usize::MAX;
        self
    }
}
impl Device for Disk {
    fn sectors(&self) -> u64 {
        4
    }
    fn read(&mut self, sector: u64, bytes: &mut [u8; 512]) -> Result<(), u64> {
        *bytes = self.cache[sector as usize];
        Ok(())
    }
    fn write(&mut self, sector: u64, bytes: &[u8; 512]) -> Result<(), u64> {
        self.operations += 1;
        if self.operations == self.fail {
            // A failed write can persist an arbitrary prefix of the target sector.
            self.stable[sector as usize][..self.tear].copy_from_slice(&bytes[..self.tear]);
            return Err(abi::IO_ERROR);
        }
        self.cache[sector as usize] = *bytes;
        Ok(())
    }
    fn flush(&mut self) -> Result<(), u64> {
        self.operations += 1;
        if self.operations == self.fail {
            for (stable, cached) in self.stable.iter_mut().zip(self.cache.iter()) {
                stable[..self.tear].copy_from_slice(&cached[..self.tear]);
            }
            return Err(abi::IO_ERROR);
        }
        self.stable = self.cache;
        Ok(())
    }
}
#[test]
fn acknowledged_versions_survive_reopen_and_stale_writers_cannot_overwrite() {
    let mut store = Store::open(Disk::blank()).unwrap();
    assert_eq!(store.read(2), Err(abi::DENIED));
    assert_eq!(store.replace(2, 0, b"foreign", |_| {}), Err(abi::DENIED));
    assert_eq!(
        store.replace(1, 0, &[0; 33], |_| {}),
        Err(abi::INVALID_ARGUMENT)
    );
    for generation in 0..12 {
        let record = store
            .replace(1, generation, &[generation as u8; 32], |_| {})
            .unwrap();
        store = Store::open(store.disk.power_loss()).unwrap();
        assert_eq!(store.read(1).unwrap(), record);
        assert_eq!(
            store.replace(1, generation, b"stale", |_| {}),
            Err(abi::WOULD_BLOCK)
        );
    }
}
#[test]
fn every_write_prefix_and_failed_flush_recovers_a_complete_committed_version() {
    let mut store = Store::open(Disk::blank()).unwrap();
    for generations in [0, 2] {
        for generation in 0..generations {
            store.replace(1, generation, b"old", |_| {}).unwrap();
        }
        let old = store.read(1).unwrap();
        let baseline = store.disk.clone().power_loss();
        for operation in 1..=4 {
            for prefix in 0..=512 {
                let mut store = Store::open(baseline.clone()).unwrap();
                store.disk.fail = operation;
                store.disk.tear = prefix;
                assert!(store.replace(1, generations, b"new", |_| {}).is_err());
                assert_eq!(store.read(1), Err(abi::IO_ERROR)); // Ambiguous instances cannot serve stale data.
                let recovered = Store::open(store.disk.power_loss())
                    .unwrap()
                    .read(1)
                    .unwrap();
                assert!(
                    recovered == old
                        || (recovered.generation == generations + 1
                            && recovered.payload() == b"new")
                );
                if operation <= 2 {
                    assert_eq!(recovered, old);
                }
            }
        }
    }
}
#[test]
fn durable_commit_with_lost_reply_is_observable_without_duplicate_write() {
    let mut store = Store::open(Disk::blank()).unwrap();
    let committed = store.replace(1, 0, b"saved", |_| {}).unwrap();
    let mut recovered = Store::open(store.disk.power_loss()).unwrap();
    assert_eq!(recovered.read(1).unwrap(), committed);
    assert_eq!(
        recovered.replace(1, 0, b"saved", |_| {}),
        Err(abi::WOULD_BLOCK)
    );
}
#[test]
fn corrupt_or_unknown_media_is_never_silently_formatted() {
    let mut disk = Disk::blank();
    disk.cache[0][0] = 0x55;
    disk.stable = disk.cache;
    let before = disk.stable;
    assert!(Store::open(&mut disk).is_err());
    assert_eq!(disk.stable, before);
    let mut store = Store::open(Disk::blank()).unwrap();
    store.replace(1, 0, b"kept", |_| {}).unwrap();
    // Corrupt both commit records; no valid root remains.
    store.disk.stable[1][0] ^= 1;
    store.disk.stable[3][0] ^= 1;
    assert!(Store::open(store.disk.power_loss()).is_err());
    assert_eq!(format::crc(b"123456789"), 0xcbf43926);
}

impl Device for &mut Disk {
    fn sectors(&self) -> u64 {
        4
    }
    fn read(&mut self, sector: u64, bytes: &mut [u8; 512]) -> Result<(), u64> {
        (**self).read(sector, bytes)
    }
    fn write(&mut self, sector: u64, bytes: &[u8; 512]) -> Result<(), u64> {
        (**self).write(sector, bytes)
    }
    fn flush(&mut self) -> Result<(), u64> {
        (**self).flush()
    }
}
#[test]
fn interrupted_initial_format_fails_closed_until_a_valid_baseline_exists() {
    for fail in 1..=4 {
        let mut disk = Disk::blank();
        disk.fail = fail;
        disk.tear = 256;
        assert!(Store::open(&mut disk).is_err());
        let disk = disk.power_loss();
        let reopened = Store::open(disk);
        // Before any acknowledgement or user data: incomplete format may fail
        // closed. A complete empty root is the only permitted recovered value.
        if let Ok(store) = reopened {
            assert_eq!(store.read(1).unwrap(), Record::default());
        }
    }
}
