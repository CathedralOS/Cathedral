// Own private EL0 image/stack frames; borrow the supervisor-only boot branch.
// Construction rolls back through the same ledger used for process retirement.
use super::memory::tables;
use super::{BootLayout, MemoryError, StackFrames};
use crate::image::{UserSegment, valid_segments};

const PAGE: u64 = 4096;
pub const USER_CODE: u64 = 0x0000_0080_0000_0000;
pub const USER_DATA: u64 = USER_CODE + PAGE;
pub const USER_IMAGE_END: u64 = USER_CODE + 1024 * 1024;
pub const USER_STACK: u64 = USER_CODE + 2 * 1024 * 1024;
pub const USER_STACK_TOP: u64 = USER_STACK + 4 * PAGE;
pub const MAX_IMAGE_PAGES: usize = 64;
const MAX_BACKING: usize = MAX_IMAGE_PAGES + 4 + 2 * cathedral_contracts::memory::MAX_PAGES;
const _: () = assert!(MAX_BACKING <= u128::BITS as usize);
const MAX_FRAMES: usize = 128;

#[derive(Clone, Copy, Default)]
struct Backing {
    address: u64,
    physical: u64,
}

#[derive(Clone, Copy)]
pub struct ImageRange {
    pub base: u64,
    pub bytes: u64,
}

pub struct UserSpace {
    root: u64,
    owned: [u64; MAX_FRAMES],
    len: usize,
    backing: [Backing; MAX_BACKING],
    pages: usize,
    writable: u128,
}
impl UserSpace {
    pub fn root(&self) -> u64 {
        self.root
    }
    pub fn frame_count(&self) -> usize {
        self.len
    }

    /// # Safety
    /// Kernel root active, IRQs off, exclusive frame custody. The supplied image
    /// and BootLayout are live supervisor resources from this boot. No mappings
    /// change until the resulting task and its entry paths have been retired.
    pub unsafe fn create(
        layout: &BootLayout,
        image: ImageRange,
        code: &[u8],
        frames: &mut impl StackFrames,
    ) -> Result<Self, MemoryError> {
        if code.is_empty() || code.len() > PAGE as usize {
            return Err(MemoryError::InvalidFrame);
        }
        let segments = [
            UserSegment {
                address: USER_CODE,
                bytes: code,
                memory_size: PAGE,
                writable: false,
                executable: true,
            },
            UserSegment {
                address: USER_DATA,
                bytes: &[],
                memory_size: PAGE,
                writable: true,
                executable: false,
            },
        ];
        // SAFETY: Same caller obligations; raw probes use two fixed disjoint pages.
        unsafe { Self::from_segments(layout, image, &segments, frames) }
    }

    /// Build a bounded image from page-disjoint, immutable segment descriptions.
    /// # Safety
    /// Same machine/ownership obligations as create. Segment bytes remain live
    /// for this call; kernel image and layout retain their boot-time mappings.
    pub unsafe fn from_segments(
        layout: &BootLayout,
        image: ImageRange,
        segments: &[UserSegment<'_>],
        frames: &mut impl StackFrames,
    ) -> Result<Self, MemoryError> {
        if image.bytes == 0 || !image.base.is_multiple_of(PAGE) || !valid_segments(segments) {
            return Err(MemoryError::InvalidFrame);
        }
        let end = image
            .base
            .checked_add(image.bytes)
            .and_then(|end| end.checked_add(PAGE - 1))
            .ok_or(MemoryError::InvalidFrame)?
            & !(PAGE - 1);
        // Keep the low identity-loaded image disjoint from the user branch.
        if end > USER_CODE {
            return Err(MemoryError::OccupiedVirtualRange);
        }
        let mut space = Self {
            root: 0,
            owned: [0; MAX_FRAMES],
            len: 0,
            backing: [Backing::default(); MAX_BACKING],
            pages: 0,
            writable: 0,
        };
        // SAFETY: New frames are exclusively owned and identity-mapped in kernel translation root.
        let result = unsafe { space.build(layout, image.base, end, segments, frames) };
        if let Err(error) = result {
            // SAFETY: This root was never activated, including partially built tables.
            unsafe {
                space.release(frames);
            }
            return Err(error);
        }
        Ok(space)
    }

    /// # Safety
    /// Kernel root active, IRQs off. Task can never resume; no saved references
    /// or mappings outside this retired root refer to these owned frames.
    pub unsafe fn release(self, frames: &mut impl StackFrames) {
        assert_ne!(tables::active(), self.root);
        for address in self.owned[..self.len].iter().rev() {
            // SAFETY: Inactive root; translation root switch flushed its non-global translations.
            unsafe {
                frames.release(*address);
            }
        }
    }
}
