//! Consume an immutable pixel lease. Validate the whole payload before scanout.
use super::{Surface, abi, wire};
use cathedral_user_runtime::{Error, memory::Shared};
impl Surface {
    pub(super) fn buffer(
        &mut self,
        x: u64,
        y: u64,
        width: u64,
        height: u64,
        handle: u64,
    ) -> [u64; 6] {
        let result = self.blit(x, y, width, height, handle);
        [
            result.err().map_or(0, |Error(code)| code as u64),
            0,
            0,
            0,
            0,
            0,
        ]
    }
    fn blit(&mut self, x: u64, y: u64, width: u64, height: u64, handle: u64) -> Result<(), Error> {
        // Even malformed geometry completes an authorized offer, so the caller
        // can reclaim it after the error reply. Foreign handles still fail closed.
        let region = Shared::accept(handle)?;
        let length = width.checked_mul(height).and_then(|n| n.checked_mul(4));
        if !wire::valid_rect(self.width, self.height, x, y, width, height)
            || length.is_none_or(|n| n > region.bytes().len() as u64)
        {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        let pixels = &region.bytes()[..length.unwrap() as usize];
        if pixels.chunks_exact(4).any(|pixel| pixel[3] != 0) {
            return Err(Error(abi::INVALID_ARGUMENT as i64));
        }
        for (index, bytes) in pixels.chunks_exact(4).enumerate() {
            let color = u32::from_le_bytes(bytes.try_into().unwrap());
            let column = x + index as u64 % width;
            let row = y + index as u64 / width;
            let pointer = (self.address + (row * self.stride + column) * 4) as *mut u32;
            // SAFETY: Exclusive device aperture and fully validated rectangle.
            unsafe { pointer.write_volatile(wire::pixel(self.format, u64::from(color))) };
        }
        // Drop unmaps the pinned read lease before main sends completion.
        Ok(())
    }
}
