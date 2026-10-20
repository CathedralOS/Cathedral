//! Stage device translations without owning the aperture or exposing partial mappings.
use super::*;

impl UserSpace {
    /// # Safety
    /// Inactive exclusively owned root, boot-approved reserved aperture. On error retire this space.
    pub unsafe fn map_framebuffer(
        &mut self,
        fb: cathedral_kernel_contracts::framebuffer::Framebuffer,
        frames: &mut impl FrameSource,
    ) -> Result<(), MemoryError> {
        // SAFETY: Initial boot completes the same engine synchronously.
        unsafe { self.map_framebuffer_pages(fb, &mut 0, usize::MAX, frames) }.map(|_| ())
    }

    /// Prepare at most budget device leaves; device backing is never owned RAM.
    /// # Safety
    /// Private inactive root, unchanged boot-approved aperture and exclusive
    /// device custody between calls. Cursor starts at zero; retire the entire
    /// space on failure. Publish no context until this returns true.
    pub unsafe fn map_framebuffer_pages(
        &mut self,
        fb: cathedral_kernel_contracts::framebuffer::Framebuffer,
        offset: &mut u64,
        budget: usize,
        frames: &mut impl FrameSource,
    ) -> Result<bool, MemoryError> {
        assert!(budget != 0);
        if !fb.valid() || !offset.is_multiple_of(PAGE) || *offset > fb.bytes {
            return Err(MemoryError::InvalidFrame);
        }
        let pages = ((fb.bytes - *offset) / PAGE) as usize;
        if pages == 0 {
            return Ok(true);
        }
        let bytes = pages.min(budget) as u64 * PAGE;
        let mut source = Tracked {
            frames,
            addresses: &mut self.owned,
            len: &mut self.len,
        };
        // SAFETY: Track every allocated table, never device backing; caller retains aperture custody.
        unsafe {
            self.mapping
                .as_mut()
                .unwrap()
                .map_framebuffer(fb, *offset, bytes, &mut source)?;
        }
        *offset += bytes;
        Ok(*offset == fb.bytes)
    }
}
