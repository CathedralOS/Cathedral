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
        Ok(())
    }

    /// Copy a bounded number of image/stack pages into a private inactive root.
    /// # Safety
    /// Same frame/root custody as from_segments. Segments and contents describe
    /// the same immutable image on every call. Retire the entire root on error.
    pub unsafe fn load_pages(
        &mut self,
        segments: &[UserSegment],
        contents: Contents<'_>,
        cursor: &mut LoadCursor,
        budget: usize,
        frames: &mut impl FrameSource,
    ) -> Result<bool, MemoryError> {
        let stack = UserSegment {
            address: USER_STACK,
            file_offset: 0,
            file_size: 0,
            memory_size: USER_STACK_PAGES as u64 * PAGE,
            writable: true,
            executable: false,
        };
        let mut source = Tracked {
            frames,
            addresses: &mut self.owned,
            len: &mut self.len,
        };
        for _ in 0..budget {
            let Some((segment, offset)) = cursor.next(segments, &stack) else {
                return Ok(true);
            };
            let physical = source.allocate().ok_or(MemoryError::OutOfFrames)?;
            let address = segment.address + offset;
            // SAFETY: Fresh exclusive backing; validated file ranges and pinned contents.
            // Zero BSS/tails before copying; executable cache maintenance precedes mapping.
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
        Ok(cursor.done(segments.len()))
    }
}

/// Format-independent progress through validated image segments and the stack.
#[derive(Default)]
pub struct LoadCursor {
    segment: usize,
    offset: u64,
}
impl LoadCursor {
    fn next<'a>(
        &mut self,
        segments: &'a [UserSegment],
        stack: &'a UserSegment,
    ) -> Option<(&'a UserSegment, u64)> {
        if self.done(segments.len()) {
            return None;
        }
        let segment = segments.get(self.segment).unwrap_or(stack);
        let offset = self.offset;
        self.offset += PAGE;
        if self.offset >= segment.memory_size {
            self.segment += 1;
            self.offset = 0;
        }
        Some((segment, offset))
    }
    fn done(&self, segments: usize) -> bool {
        self.segment > segments
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cursor_crosses_partial_pages_gaps_and_stack_without_skips_or_duplicates() {
        let segments = [
            UserSegment {
                address: USER_CODE,
                file_offset: 0,
                file_size: 1,
                memory_size: PAGE + 1,
                writable: false,
                executable: true,
            },
            UserSegment {
                address: USER_CODE + 8 * PAGE,
                file_offset: PAGE,
                file_size: 0,
                memory_size: 3 * PAGE,
                writable: true,
                executable: false,
            },
        ];
        let stack = UserSegment {
            address: USER_STACK,
            file_offset: 0,
            file_size: 0,
            memory_size: USER_STACK_PAGES as u64 * PAGE,
            writable: true,
            executable: false,
        };
        let expected: alloc::vec::Vec<u64> = segments
            .iter()
            .chain(core::iter::once(&stack))
            .flat_map(|s| {
                (0..s.memory_size.div_ceil(PAGE)).map(move |page| s.address + page * PAGE)
            })
            .collect();
        for batch in [1, 2, 3, 8, 32] {
            let mut cursor = LoadCursor::default();
            let mut visited = alloc::vec::Vec::new();
            while !cursor.done(segments.len()) {
                for _ in 0..batch {
                    if let Some((segment, offset)) = cursor.next(&segments, &stack) {
                        visited.push(segment.address + offset);
                    }
                }
            }
            assert_eq!(visited, expected);
            assert!(cursor.next(&segments, &stack).is_none());
        }
    }
}
