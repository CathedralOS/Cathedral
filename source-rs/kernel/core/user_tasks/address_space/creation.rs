//! Load private image and stack pages; CPU backends only encode and publish translations.
use super::*;
use cathedral_arch::memory::Permissions;
impl UserSpace {
    pub(super) unsafe fn build(
        &mut self,
        layout: &BootLayout,
        start: u64,
        end: u64,
        segments: &[UserSegment],
        contents: Contents<'_>,
        frames: &mut impl FrameSource,
    ) -> Result<(), MemoryError> {
        let pages = USER_STACK_PAGES
            + segments
                .iter()
                .map(|s| s.memory_size.div_ceil(PAGE) as usize)
                .sum::<usize>();
        self.backing
            .try_reserve_exact(pages)
            .map_err(|_| MemoryError::OutOfFrames)?;
        let mut source = Tracked {
            frames,
            addresses: &mut self.owned,
            len: &mut self.len,
        };
        // SAFETY: New inactive tree and permanent boot mappings; ledger captures failures too.
        self.mapping = Some(unsafe {
            UserMappings::create(
                layout,
                ImageRange {
                    base: start,
                    bytes: end - start,
                },
                &mut source,
            )?
        });
        let stack = UserSegment {
            address: USER_STACK,
            file_offset: 0,
            file_size: 0,
            memory_size: USER_STACK_PAGES as u64 * PAGE,
            writable: true,
            executable: false,
        };
        for segment in segments.iter().chain(core::iter::once(&stack)) {
            for offset in (0..segment.memory_size).step_by(PAGE as usize) {
                let physical = source.allocate().ok_or(MemoryError::OutOfFrames)?;
                let address = segment.address + offset;
                // SAFETY: Fresh exclusive backing; source custody includes a writable kernel alias.
                // Zero BSS/tails before copying file bytes; cache maintenance precedes publication.
                unsafe {
                    core::ptr::write_bytes(physical as *mut u8, 0, PAGE as usize);
                    if offset < segment.file_size {
                        let len = (segment.file_size - offset).min(PAGE) as usize;
                        contents.copy_page(segment.file_offset + offset, physical as *mut u8, len);
                    }
                    if segment.executable {
                        UserMappings::publish_code(physical);
                    }
                    self.mapping.as_mut().unwrap().map_page(
                        address,
                        physical,
                        Permissions {
                            writable: segment.writable,
                            executable: segment.executable,
                        },
                        &mut source,
                    )?;
                }
                self.backing.push(Backing {
                    address,
                    physical,
                    writable: segment.writable,
                });
                self.pages += 1;
            }
        }
        Ok(())
    }
}
