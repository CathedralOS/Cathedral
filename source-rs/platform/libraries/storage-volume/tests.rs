use super::*;
use cathedral_block_device::SECTOR;
use std::{vec, vec::Vec};

struct Disk {
    visible: Vec<u8>,
    durable: Vec<u8>,
    calls: usize,
    fail: usize,
}
impl Disk {
    fn new() -> Self {
        let bytes: Vec<_> = (0..8192).map(|n| (n % 251) as u8).collect();
        Self {
            visible: bytes.clone(),
            durable: bytes,
            calls: 0,
            fail: 0,
        }
    }
    fn call(&mut self) -> Result<(), u64> {
        self.calls += 1;
        if self.calls == self.fail {
            Err(77)
        } else {
            Ok(())
        }
    }
}
impl Device for Disk {
    fn sectors(&self) -> u64 {
        (self.visible.len() / SECTOR) as u64
    }
    fn read(&mut self, sector: u64, out: &mut [u8; SECTOR]) -> Result<(), u64> {
        self.call()?;
        let offset = sector as usize * SECTOR;
        out.copy_from_slice(&self.visible[offset..offset + SECTOR]);
        Ok(())
    }
    fn write(&mut self, sector: u64, bytes: &[u8; SECTOR]) -> Result<(), u64> {
        let offset = sector as usize * SECTOR;
        if let Err(error) = self.call() {
            // A reported failure can already have changed part of the sector.
            self.visible[offset..offset + 17].copy_from_slice(&bytes[..17]);
            return Err(error);
        }
        self.visible[offset..offset + SECTOR].copy_from_slice(bytes);
        Ok(())
    }
    fn flush(&mut self) -> Result<(), u64> {
        if let Err(error) = self.call() {
            self.durable[..2048].copy_from_slice(&self.visible[..2048]);
            return Err(error);
        }
        self.durable.clone_from(&self.visible);
        Ok(())
    }
}

#[test]
fn arbitrary_byte_ranges_preserve_sectors_and_neighbours_until_flush() {
    for offset in [0, 1, 509, 511, 512, 1023, 4095] {
        for count in [0, 1, 2, 511, 512, 513, 1031] {
            if offset + count > 4096 {
                continue;
            }
            let mut disk = Disk::new();
            let before = disk.durable.clone();
            let replacement = vec![0x87; count];
            let mut expected = before.clone();
            expected[512 + offset..512 + offset + count].copy_from_slice(&replacement);
            {
                let mut region = BlockRegion::new(&mut disk, 512, 4096).unwrap();
                region.write(offset as u64, &replacement).unwrap();
                let mut read = vec![0; 4096];
                region.read(0, &mut read).unwrap();
                assert_eq!(read, expected[512..4608]);
            }
            assert_eq!(disk.durable, before, "write falsely promised durability");
            assert_eq!(disk.visible, expected);
            BlockRegion::new(&mut disk, 512, 4096)
                .unwrap()
                .flush()
                .unwrap();
            assert_eq!(disk.durable, expected);
        }
    }
}

#[test]
fn invalid_geometry_and_entire_io_are_rejected_before_device_calls() {
    let mut disk = Disk::new();
    for (start, bytes) in [
        (1, 512),
        (0, 513),
        (0, 0),
        (8192, 512),
        (u64::MAX - 511, 512),
    ] {
        assert!(BlockRegion::new(&mut disk, start, bytes).is_err());
    }
    {
        let mut region = BlockRegion::new(&mut disk, 512, 4096).unwrap();
        for (start, count) in [(4095, 2), (4096, 1), (u64::MAX, 2), (4097, 0)] {
            assert!(region.write(start, &vec![0; count]).is_err());
            assert!(region.read(start, &mut vec![0; count]).is_err());
        }
        region.read(4096, &mut []).unwrap();
        region.write(4096, &[]).unwrap();
    }
    assert_eq!(disk.calls, 0);
}

#[test]
fn every_partial_write_and_flush_failure_requires_device_recovery() {
    // offset 509, length 1031: read/write, write, write, read/write, flush.
    for fail in 1..=7 {
        let mut disk = Disk::new();
        disk.fail = fail;
        let mut region = BlockRegion::new(&mut disk, 512, 4096).unwrap();
        let written = region.write(509, &[8; 1031]);
        let result = if fail == 7 {
            assert!(written.is_ok());
            region.flush()
        } else {
            written
        };
        assert_eq!(result, Err(Error::Device(77)), "failure {fail}");
        assert_eq!(region.flush(), Err(Error::Failed));
        assert_eq!(region.write(0, &[1]), Err(Error::Failed));
        assert_eq!(region.read(0, &mut [0]), Err(Error::Failed));
        assert_eq!(disk.calls, fail, "continued IO after uncertainty");
    }
}

#[test]
fn envelope_inspection_uses_shared_codec_without_writes_or_repairs() {
    let mut disk = Disk::new();
    let size = 4 * 1024 * 1024;
    disk.visible = vec![0; size + 1024];
    let expected = Header {
        id: [1; 16],
        generation: 4,
        region_bytes: size as u64,
        length: 8192,
        ready: true,
    };
    for slot in 0..2 {
        let start = 512 + slot * BLOCK as usize;
        disk.visible[start..start + BLOCK as usize].copy_from_slice(&expected.encode());
    }
    let before = disk.visible.clone();
    let got = inspect(&mut BlockRegion::new(&mut disk, 512, size as u64).unwrap()).unwrap();
    assert_eq!(got, expected);
    assert_eq!(disk.calls, 16);
    assert_eq!(disk.visible, before);
    disk.visible[512] ^= 1;
    assert_eq!(
        inspect(&mut BlockRegion::new(&mut disk, 512, size as u64).unwrap()).unwrap(),
        expected
    );
    disk.visible[512 + BLOCK as usize] ^= 1;
    assert!(inspect(&mut BlockRegion::new(&mut disk, 512, size as u64).unwrap()).is_err());
}

#[test]
fn contiguous_device_range_is_checked_before_fallback_io() {
    let mut disk = Disk::new();
    let mut blocks = [[0; SECTOR]; 2];
    for sector in [15, u64::MAX] {
        assert_eq!(
            disk.read_many(sector, &mut blocks),
            Err(cathedral_block_device::OUT_OF_RANGE)
        );
        assert_eq!(
            disk.write_many(sector, &blocks),
            Err(cathedral_block_device::OUT_OF_RANGE)
        );
    }
    assert_eq!(disk.calls, 0);
    disk.read_many(14, &mut blocks).unwrap();
    assert_eq!(blocks.as_flattened(), &disk.visible[14 * SECTOR..]);
}

#[test]
fn aligned_region_dispatches_batches_and_retires_after_partial_batch_failure() {
    struct BatchDisk {
        calls: usize,
    }
    impl Device for BatchDisk {
        fn sectors(&self) -> u64 {
            16
        }
        fn read(&mut self, _: u64, _: &mut [u8; SECTOR]) -> Result<(), u64> {
            panic!("unbatched")
        }
        fn write(&mut self, _: u64, _: &[u8; SECTOR]) -> Result<(), u64> {
            panic!("unbatched")
        }
        fn read_many(&mut self, sector: u64, out: &mut [[u8; SECTOR]]) -> Result<(), u64> {
            assert_eq!((sector, out.len()), (2, 3));
            self.calls += 1;
            out.fill([42; SECTOR]);
            Ok(())
        }
        fn write_many(&mut self, sector: u64, bytes: &[[u8; SECTOR]]) -> Result<(), u64> {
            assert_eq!((sector, bytes.len()), (2, 3));
            self.calls += 1;
            Err(77) // Some sectors may already have reached the device.
        }
        fn flush(&mut self) -> Result<(), u64> {
            panic!("continued uncertain IO")
        }
    }
    let mut disk = BatchDisk { calls: 0 };
    let mut region = BlockRegion::new(&mut disk, 512, 4096).unwrap();
    let mut bytes = [0; 3 * SECTOR];
    region.read(512, &mut bytes).unwrap();
    assert_eq!(bytes, [42; 3 * SECTOR]);
    assert_eq!(region.write(512, &bytes), Err(Error::Device(77)));
    assert_eq!(region.flush(), Err(Error::Failed));
    assert_eq!(region.read(512, &mut bytes), Err(Error::Failed));
    assert_eq!(disk.calls, 2);
}
