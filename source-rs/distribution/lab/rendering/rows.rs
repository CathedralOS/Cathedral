//! Exact software bounds for cooperating safe Rust. This is NOT a native-code sandbox.
use crate::Error;
pub struct Row<'a> {
    bytes: &'a mut [u8],
}
impl<'a> Row<'a> {
    pub fn new(bytes: &'a mut [u8]) -> Result<Self, Error> {
        if !bytes.len().is_multiple_of(4) {
            return Err(Error::Bounds);
        }
        Ok(Self { bytes })
    }
    /// Consumes parent write access; children cover disjoint ranges of the same allocation.
    /// ```compile_fail
    /// use cathedral_rendering_lab::rows::Row;
    /// let mut bytes = [0; 16];
    /// let mut parent = Row::new(&mut bytes).unwrap();
    /// let (mut left, mut right) = parent.split(2).unwrap();
    /// parent.put(0, 1).unwrap(); // Parent was moved, not cloned.
    /// left.put(0, 2).unwrap();
    /// ```
    pub fn split(self, pixels: usize) -> Result<(Self, Self), Error> {
        let bytes = pixels.checked_mul(4).ok_or(Error::Bounds)?;
        if bytes > self.bytes.len() {
            return Err(Error::Bounds);
        }
        let (left, right) = self.bytes.split_at_mut(bytes);
        Ok((Self { bytes: left }, Self { bytes: right }))
    }
    /// Transfer a narrower span; no reference to the surrendered remainder escapes.
    pub fn narrow(self, start: usize, count: usize) -> Result<Self, Error> {
        let start = start.checked_mul(4).ok_or(Error::Bounds)?;
        let end = count
            .checked_mul(4)
            .and_then(|n| start.checked_add(n))
            .ok_or(Error::Bounds)?;
        Ok(Self {
            bytes: self.bytes.get_mut(start..end).ok_or(Error::Bounds)?,
        })
    }
    pub fn put(&mut self, pixel: usize, color: u32) -> Result<(), Error> {
        let start = pixel.checked_mul(4).ok_or(Error::Bounds)?;
        let end = start.checked_add(4).ok_or(Error::Bounds)?;
        self.bytes
            .get_mut(start..end)
            .ok_or(Error::Bounds)?
            .copy_from_slice(&color.to_le_bytes());
        Ok(())
    }
}
