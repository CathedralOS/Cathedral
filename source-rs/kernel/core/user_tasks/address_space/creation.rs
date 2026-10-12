//! Load private image and stack pages; CPU backends only encode and publish translations.
use super::*;
use cathedral_arch::memory::Permissions;
impl UserSpace {
    pub(super) unsafe fn build(
        &mut self,
        layout: &BootLayout,
        start: u64,
        end: u64,
        segments: &[UserSegment<'_>],
        frames: &mut impl FrameSource,
    ) -> Result<(), MemoryError> {
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
            bytes: &[],
            memory_size: 4 * PAGE,
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
                    if (offset as usize) < segment.bytes.len() {
                        let bytes = &segment.bytes[offset as usize..];
                        core::ptr::copy_nonoverlapping(
                            bytes.as_ptr(),
                            physical as *mut u8,
                            bytes.len().min(PAGE as usize),
                        );
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
                self.backing[self.pages] = Backing { address, physical };
                if segment.writable {
                    self.writable |= 1 << self.pages;
                }
                self.pages += 1;
            }
        }
        Ok(())
    }
}
