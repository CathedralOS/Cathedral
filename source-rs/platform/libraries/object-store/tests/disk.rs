use super::*;
#[derive(Clone)]
pub(super) struct Disk {
    pub(super) stable: [[u8; 512]; 4],
    pub(super) cache: [[u8; 512]; 4],
    pub(super) operations: usize,
    pub(super) fail: usize,
    pub(super) tear: usize,
}
impl Disk {
    pub(super) fn blank() -> Self {
        Self {
            stable: [[0; 512]; 4],
            cache: [[0; 512]; 4],
            operations: 0,
            fail: usize::MAX,
            tear: 0,
        }
    }
    pub(super) fn power_loss(mut self) -> Self {
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
