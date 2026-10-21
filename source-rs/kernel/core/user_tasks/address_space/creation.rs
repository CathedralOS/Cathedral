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

    /// Advance the ISA's supervisor mappings without exposing a runnable root.
    /// # Safety
    /// Same permanently live boot resources and exclusive private-root custody
    /// as begin, including between calls. Retire the whole space on error.
    pub unsafe fn prepare_kernel(
        &mut self,
        frames: &mut impl FrameSource,
        budget: usize,
    ) -> Result<bool, MemoryError> {
        let mut source = Tracked {
            frames,
            addresses: &mut self.owned,
            len: &mut self.len,
        };
        // SAFETY: Backend retains owned progress only; caller owns every table frame.
        unsafe {
            self.mapping
                .as_mut()
                .unwrap()
                .prepare_kernel(&mut source, budget)
        }
    }

    /// Load within both page and file-byte allowances into a private inactive root.
    /// # Safety
    /// Same frame/root custody as from_segments. Segments and contents describe
    /// the same immutable image on every call. Retire the entire root on error.
    pub unsafe fn load_pages(
        &mut self,
        segments: &[UserSegment],
        contents: Contents<'_>,
        cursor: &mut LoadCursor,
        budget: LoadBudget,
        frames: &mut impl FrameSource,
        #[cfg(feature = "memory-lab")] mut observe: Option<&mut dyn FnMut(LoadStage, u64)>,
    ) -> Result<bool, MemoryError> {
        assert!(budget.pages != 0 && budget.file_bytes >= PAGE as usize);
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
        let mut file_bytes = budget.file_bytes;
        for _ in 0..budget.pages {
            let Some((segment, offset)) = cursor.next(segments, &stack, &mut file_bytes) else {
                return Ok(cursor.done(segments.len()));
            };
            #[cfg(feature = "memory-lab")]
            let start = observe.as_ref().map(|_| cathedral_arch::cpu_time_us());
            let allocated = source.allocate().ok_or(MemoryError::OutOfFrames);
            #[cfg(feature = "memory-lab")]
            observe_stage(&mut observe, LoadStage::Backing, start);
            let physical = allocated?;
            let address = segment.address + offset;
            let len = segment.file_size.saturating_sub(offset).min(PAGE) as usize;
            #[cfg(feature = "memory-lab")]
            let start = observe.as_ref().map(|_| cathedral_arch::cpu_time_us());
            // SAFETY: Fresh exclusive backing; validated file ranges and pinned contents.
            // Initialize file bytes and zero tails/BSS once; cache maintenance precedes mapping.
            unsafe {
                let file_offset = if len == 0 {
                    0
                } else {
                    segment.file_offset + offset
                };
                contents.populate_page(file_offset, physical as *mut u8, len);
            }
            #[cfg(feature = "memory-lab")]
            observe_stage(&mut observe, LoadStage::Populate { file_bytes: len }, start);
            #[cfg(feature = "memory-lab")]
            let start = observe.as_ref().map(|_| cathedral_arch::cpu_time_us());
            // SAFETY: Fully initialized retained page; cache publication and
            // translation construction finish before the private ledger entry.
            let mapped = unsafe {
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
                )
            };
            #[cfg(feature = "memory-lab")]
            observe_stage(&mut observe, LoadStage::Publish, start);
            mapped?;
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

/// Lab-only per-page stages; publication includes cache maintenance and mapping.
#[cfg(feature = "memory-lab")]
pub enum LoadStage {
    Backing,
    Populate { file_bytes: usize },
    Publish,
}

#[cfg(feature = "memory-lab")]
fn observe_stage(
    observer: &mut Option<&mut dyn FnMut(LoadStage, u64)>,
    stage: LoadStage,
    start: Option<u64>,
) {
    if let (Some(observer), Some(start)) = (observer.as_mut(), start) {
        observer(stage, cathedral_arch::cpu_time_us().saturating_sub(start));
    }
}

/// Independent work allowances for a private image-loading turn.
pub struct LoadBudget {
    pub pages: usize,
    /// At least one page, so every valid next page can make progress. Only file
    /// bytes spend this allowance; zero-only pages still spend the page allowance.
    pub file_bytes: usize,
}

/// A refused next-page allowance leaves both segment and offset unchanged.
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
        file_bytes: &mut usize,
    ) -> Option<(&'a UserSegment, u64)> {
        if self.done(segments.len()) {
            return None;
        }
        let segment = segments.get(self.segment).unwrap_or(stack);
        let offset = self.offset;
        let copied = segment.file_size.saturating_sub(offset).min(PAGE) as usize;
        if copied > *file_bytes {
            return None;
        }
        *file_bytes -= copied;
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
                let mut file_bytes = usize::MAX;
                for _ in 0..batch {
                    if let Some((segment, offset)) = cursor.next(&segments, &stack, &mut file_bytes)
                    {
                        visited.push(segment.address + offset);
                    }
                }
            }
            assert_eq!(visited, expected);
            let mut file_bytes = usize::MAX;
            assert!(cursor.next(&segments, &stack, &mut file_bytes).is_none());
        }
    }

    #[test]
    fn file_byte_allowance_preserves_progress_across_partial_pages_and_bss() {
        let segments = [
            UserSegment {
                address: USER_CODE,
                file_offset: 0,
                file_size: 3 * PAGE + 17,
                memory_size: 6 * PAGE,
                writable: false,
                executable: true,
            },
            UserSegment {
                address: USER_CODE + 8 * PAGE,
                file_offset: 4 * PAGE,
                file_size: 2 * PAGE,
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
        let mut cursor = LoadCursor::default();
        let mut batches = alloc::vec::Vec::new();
        let mut visited = alloc::vec::Vec::new();
        while !cursor.done(segments.len()) {
            let mut file_bytes = 2 * PAGE as usize;
            let mut pages = 0;
            for _ in 0..8 {
                let Some((segment, offset)) = cursor.next(&segments, &stack, &mut file_bytes)
                else {
                    break;
                };
                visited.push(segment.address + offset);
                pages += 1;
            }
            assert!(pages != 0, "allowance must never strand a valid next page");
            batches.push((pages, 2 * PAGE as usize - file_bytes));
        }
        assert_eq!(batches, [(2, 8192), (4, 4113), (8, 8192), (3, 0)]);
        let expected: alloc::vec::Vec<_> = segments
            .iter()
            .chain(core::iter::once(&stack))
            .flat_map(|s| {
                (0..s.memory_size.div_ceil(PAGE)).map(move |page| s.address + page * PAGE)
            })
            .collect();
        assert_eq!(visited, expected);
    }
}
