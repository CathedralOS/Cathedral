// Process address-space ownership: validate, construct with a frame ledger, and retire.
// creation loads images; regions borrows page objects; access checks copies against live backing.
use cathedral_arch::{
    BootLayout, FrameSource, ImageRange, MemoryError, UserMappings, memory::PAGE,
};
use frames::Tracked;
pub const USER_CODE: u64 = 0x0000_0080_0000_0000;
pub const USER_DATA: u64 = USER_CODE + PAGE;
pub const USER_IMAGE_END: u64 = USER_CODE + 1024 * 1024;
pub const USER_STACK: u64 = USER_CODE + 2 * 1024 * 1024;
pub const USER_STACK_PAGES: usize = 8;
pub const USER_STACK_TOP: u64 = USER_STACK + USER_STACK_PAGES as u64 * PAGE;
pub const MAX_IMAGE_PAGES: usize = 240;
const MAX_BACKING: usize =
    MAX_IMAGE_PAGES + USER_STACK_PAGES + 2 * cathedral_kernel_contracts::memory::MAX_PAGES;
// Owned image/stack frames and translation tables; page objects retain their own ledger.
// This bounds bookkeeping, not funding: every allocation still charges its account.
const MAX_FRAMES: usize = cathedral_kernel_contracts::user::MAX_ADMISSION_FRAMES;

#[derive(Clone, Copy, Default)]
struct Backing {
    address: u64,
    physical: u64,
    writable: bool,
}

pub struct UserSpace {
    mapping: Option<UserMappings>,
    owned: alloc::vec::Vec<u64>,
    len: usize,
    backing: alloc::vec::Vec<Backing>,
    pages: usize,
}
impl UserSpace {
    pub fn root(&self) -> u64 {
        self.mapping.as_ref().unwrap().root()
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
        frames: &mut impl FrameSource,
    ) -> Result<Self, MemoryError> {
        if code.is_empty() || code.len() > PAGE as usize {
            return Err(MemoryError::InvalidFrame);
        }
        let segments = [
            UserSegment {
                address: USER_CODE,
                file_offset: 0,
                file_size: code.len() as u64,
                memory_size: PAGE,
                writable: false,
                executable: true,
            },
            UserSegment {
                address: USER_DATA,
                file_offset: 0,
                file_size: 0,
                memory_size: PAGE,
                writable: true,
                executable: false,
            },
        ];
        // SAFETY: Same caller obligations; raw probes use two fixed disjoint pages.
        unsafe { Self::from_segments(layout, image, &segments, Contents::Bytes(code), frames) }
    }

    /// Build a bounded image from page-disjoint, immutable segment descriptions.
    /// # Safety
    /// Same machine/ownership obligations as create. Segment bytes remain live
    /// for this call; kernel image and layout retain their boot-time mappings.
    pub unsafe fn from_segments(
        layout: &BootLayout,
        image: ImageRange,
        segments: &[UserSegment],
        contents: Contents<'_>,
        frames: &mut impl FrameSource,
    ) -> Result<Self, MemoryError> {
        // SAFETY: Same inactive-root and immutable-source obligations.
        let mut space = unsafe { Self::begin(layout, image, segments, frames)? };
        let mut cursor = LoadCursor::default();
        if let Err(error) =
            // SAFETY: Serialized wrapper for initial boot/fixtures; same batched page engine.
            unsafe { space.load_pages(segments, contents, &mut cursor, usize::MAX, frames) }
        {
            // SAFETY: Private root, never runnable, including partially built tables.
            unsafe {
                space.release(frames);
            }
            return Err(error);
        }
        Ok(space)
    }

    /// Start a private address space without installing any user image/stack pages.
    /// # Safety
    /// Same exclusive inactive-root and frame custody as from_segments.
    pub unsafe fn begin(
        layout: &BootLayout,
        image: ImageRange,
        segments: &[UserSegment],
        frames: &mut impl FrameSource,
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
            mapping: None,
            owned: alloc::vec::Vec::new(),
            len: 0,
            backing: alloc::vec::Vec::new(),
            pages: 0,
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
    /// Inactive exclusively owned root, boot-approved reserved aperture. On error retire this space.
    pub unsafe fn map_framebuffer(
        &mut self,
        fb: cathedral_kernel_contracts::framebuffer::Framebuffer,
        frames: &mut impl FrameSource,
    ) -> Result<(), MemoryError> {
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
                .map_framebuffer(fb, &mut source)
        }
    }
    /// # Safety
    /// Kernel root active, IRQs off. Task can never resume; no saved references
    /// or mappings outside this retired root refer to these owned frames.
    pub unsafe fn release(mut self, frames: &mut impl FrameSource) {
        // SAFETY: Whole-operation wrapper for construction rollback and initial boot.
        unsafe {
            self.release_pages(frames, usize::MAX);
        }
    }

    /// Release at most budget frames from a permanently retired root, root last.
    /// No mapping or checked-copy operation may use this space after the first step.
    /// # Safety
    /// Same custody as release. All borrowed page-object mappings must be removed
    /// first. No context can resume this tree, including between steps.
    pub unsafe fn release_pages(&mut self, frames: &mut impl FrameSource, budget: usize) -> bool {
        if let Some(mapping) = &self.mapping {
            mapping.assert_inactive();
        }
        for _ in 0..budget.min(self.len) {
            self.len -= 1;
            let address = self.owned[self.len];
            // SAFETY: Inactive root; translation root switch flushed its non-global translations.
            unsafe {
                frames.release(address);
            }
        }
        self.len == 0
    }
}
