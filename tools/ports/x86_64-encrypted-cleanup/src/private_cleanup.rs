// SPDX-License-Identifier: MIT OR Apache-2.0
// Private clean_up body from x86_64 cc35c876d3badb57df54a66e22f7768a52be95f2.
// Only pointer resolution is substituted by a stable owned registry (with
// parameter plumbing), and the private step helper is reached via public Step.
// This is an extracted mirror, not a call to RecursivePageTable::clean_up.
        fn clean_up(
            recursive_index: PageTableIndex,
            page_table: &mut PageTable,
            level: PageTableLevel,
            range: PageRangeInclusive,
            frame_deallocator: &mut impl FrameDeallocator<Size4KiB>,
            registry: &Registry,
        ) -> bool {
            if range.is_empty() {
                return false;
            }

            let table_addr = range
                .start
                .start_address()
                .align_down(level.table_address_space_alignment());

            let start = range.start.page_table_index(level);
            let end = range.end.page_table_index(level);

            if let Some(next_level) = level.next_lower_level() {
                let offset_per_entry = level.entry_address_space_alignment();
                for (i, entry) in page_table
                    .iter_mut()
                    .enumerate()
                    .take(usize::from(end) + 1)
                    .skip(usize::from(start))
                    .filter(|(i, _)| {
                        !(level == PageTableLevel::Four && *i == recursive_index.into())
                    })
                {
                    if let Ok(frame) = entry.frame() {
                        let start = <VirtAddr as Step>::forward_checked(
                            table_addr,
                            (offset_per_entry as usize) * i,
                        )
                        .unwrap();
                        let end = start + (offset_per_entry - 1);
                        let start = Page::<Size4KiB>::containing_address(start);
                        let start = start.max(range.start);
                        let end = Page::<Size4KiB>::containing_address(end);
                        let end = end.min(range.end);
                        let page_table = registry.resolve(frame);
                        let page_table = unsafe { &mut *page_table };
                        if clean_up(
                            recursive_index,
                            page_table,
                            next_level,
                            Page::range_inclusive(start, end),
                            frame_deallocator,
                            registry,
                        ) {
                            entry.set_unused();
                            unsafe {
                                frame_deallocator.deallocate_frame(frame);
                            }
                        }
                    }
                }
            }

            page_table.iter().all(PageTableEntry::is_unused)
        }
