// SPDX-License-Identifier: MIT OR Apache-2.0
// Generated pinned branch/mutation bodies with topology accesses explicitly replaced.
// No RecursivePageTable instance, raw pointer, live root or instruction exists here.
#![allow(unused_variables,unused_unsafe,non_snake_case)]
use x86_64::VirtAddr;
use x86_64::structures::paging::{Page,PageSize,Size4KiB,Size2MiB,Size1GiB,PhysFrame,PageTableFlags,page::AddressNotAligned,page_table::FrameError,mapper::{MapToError,UnmapError,FlagUpdateError,TranslateError,MapperFlush,MapperFlushAll,TranslateResult,MappedFrame}};
use crate::{ObservedTable as PageTable,ObservedEntry as PageTableEntry,AllocationObservations as FrameAllocator};
fn create_next_table<'b,A,S:PageSize>(entry:&mut PageTableEntry,next_table_page:&'b mut PageTable,insert_flags:PageTableFlags,allocator:&mut A)->Result<&'b mut PageTable,MapToError<S>> where A:FrameAllocator<Size4KiB>+?Sized {
            use x86_64::structures::paging::PageTableFlags as Flags;

            let created;

            if entry.is_unused() {
                if let Some(frame) = allocator.allocate_frame() {
                    entry.set_frame(frame, Flags::PRESENT | Flags::WRITABLE | insert_flags);
                    created = true;
                } else {
                    return Err(MapToError::FrameAllocationFailed);
                }
            } else {
                if !insert_flags.is_empty() && !entry.flags().contains(insert_flags) {
                    entry.set_flags(entry.flags() | insert_flags);
                }
                created = false;
            }
            if entry.flags().contains(Flags::HUGE_PAGE) {
                return Err(MapToError::ParentEntryHugePage);
            }

            let page_table: &mut PageTable = next_table_page;
            if created {
                page_table.zero();
            }
            Ok(page_table)
        }
pub fn map_to_with_table_flags_Size1GiB<A>(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size1GiB>,frame:PhysFrame<Size1GiB>,flags:PageTableFlags,parent_table_flags:PageTableFlags,allocator:&mut A)->Result<MapperFlush<Size1GiB>,MapToError<Size1GiB>> where A:FrameAllocator<Size4KiB>+?Sized {
        use x86_64::structures::paging::PageTableFlags as Flags;


        let p3_page = table3;
        let p3 = unsafe {
            create_next_table(
                &mut p4[page.p4_index()],
                p3_page,
                parent_table_flags,
                allocator,
            )?
        };

        if !p3[page.p3_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped(frame));
        }
        p3[page.p3_index()].set_addr(frame.start_address(), flags | Flags::HUGE_PAGE);

        Ok(MapperFlush::new(page))
    }
pub fn unmap_Size1GiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size1GiB>)->Result<(PhysFrame<Size1GiB>,MapperFlush<Size1GiB>),UnmapError> {

        let p4_entry = &p4[page.p4_index()];

        p4_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;

        let p3 = table3;
        let p3_entry = &mut p3[page.p3_index()];
        let flags = p3_entry.flags();

        if !flags.contains(PageTableFlags::PRESENT) {
            return Err(UnmapError::PageNotMapped);
        }
        if !flags.contains(PageTableFlags::HUGE_PAGE) {
            return Err(UnmapError::ParentEntryHugePage);
        }

        let frame = PhysFrame::from_start_address(p3_entry.addr())
            .map_err(|AddressNotAligned| UnmapError::InvalidFrameAddress(p3_entry.addr()))?;

        p3_entry.set_unused();
        Ok((frame, MapperFlush::new(page)))
    }
pub fn update_flags_Size1GiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size1GiB>,flags:PageTableFlags)->Result<MapperFlush<Size1GiB>,FlagUpdateError> {
        use x86_64::structures::paging::PageTableFlags as Flags;


        if p4[page.p4_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        let p3 = table3;

        if p3[page.p3_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }
        p3[page.p3_index()].set_flags(flags | Flags::HUGE_PAGE);

        Ok(MapperFlush::new(page))
    }
pub fn set_flags_p4_entry_Size1GiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size1GiB>,flags:PageTableFlags)->Result<MapperFlushAll,FlagUpdateError> {

        let p4_entry = &mut p4[page.p4_index()];

        if p4_entry.is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p4_entry.set_flags(flags);

        Ok(MapperFlushAll::new())
    }
pub fn set_flags_p3_entry_Size1GiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size1GiB>,flags:PageTableFlags)->Result<MapperFlushAll,FlagUpdateError> {
        Err(FlagUpdateError::ParentEntryHugePage)
    }
pub fn set_flags_p2_entry_Size1GiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size1GiB>,flags:PageTableFlags)->Result<MapperFlushAll,FlagUpdateError> {
        Err(FlagUpdateError::ParentEntryHugePage)
    }
pub fn translate_page_Size1GiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size1GiB>)->Result<PhysFrame<Size1GiB>,TranslateError> {


        if p4[page.p4_index()].is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        let p3 = table3;
        let p3_entry = &p3[page.p3_index()];

        if p3_entry.is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        PhysFrame::from_start_address(p3_entry.addr())
            .map_err(|AddressNotAligned| TranslateError::InvalidFrameAddress(p3_entry.addr()))
    }
pub fn map_to_with_table_flags_Size2MiB<A>(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size2MiB>,frame:PhysFrame<Size2MiB>,flags:PageTableFlags,parent_table_flags:PageTableFlags,allocator:&mut A)->Result<MapperFlush<Size2MiB>,MapToError<Size2MiB>> where A:FrameAllocator<Size4KiB>+?Sized {
        use x86_64::structures::paging::PageTableFlags as Flags;


        let p3_page = table3;
        let p3 = unsafe {
            create_next_table(
                &mut p4[page.p4_index()],
                p3_page,
                parent_table_flags,
                allocator,
            )?
        };

        let p2_page = table2;
        let p2 = unsafe {
            create_next_table(
                &mut p3[page.p3_index()],
                p2_page,
                parent_table_flags,
                allocator,
            )?
        };

        if !p2[page.p2_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped(frame));
        }
        p2[page.p2_index()].set_addr(frame.start_address(), flags | Flags::HUGE_PAGE);

        Ok(MapperFlush::new(page))
    }
pub fn unmap_Size2MiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size2MiB>)->Result<(PhysFrame<Size2MiB>,MapperFlush<Size2MiB>),UnmapError> {

        let p4_entry = &p4[page.p4_index()];
        p4_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;

        let p3 = table3;
        let p3_entry = &p3[page.p3_index()];
        p3_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;

        let p2 = table2;
        let p2_entry = &mut p2[page.p2_index()];
        let flags = p2_entry.flags();

        if !flags.contains(PageTableFlags::PRESENT) {
            return Err(UnmapError::PageNotMapped);
        }
        if !flags.contains(PageTableFlags::HUGE_PAGE) {
            return Err(UnmapError::ParentEntryHugePage);
        }

        let frame = PhysFrame::from_start_address(p2_entry.addr())
            .map_err(|AddressNotAligned| UnmapError::InvalidFrameAddress(p2_entry.addr()))?;

        p2_entry.set_unused();
        Ok((frame, MapperFlush::new(page)))
    }
pub fn update_flags_Size2MiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size2MiB>,flags:PageTableFlags)->Result<MapperFlush<Size2MiB>,FlagUpdateError> {
        use x86_64::structures::paging::PageTableFlags as Flags;


        if p4[page.p4_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        let p3 = table3;

        if p3[page.p3_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        let p2 = table2;

        if p2[page.p2_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p2[page.p2_index()].set_flags(flags | Flags::HUGE_PAGE);

        Ok(MapperFlush::new(page))
    }
pub fn set_flags_p4_entry_Size2MiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size2MiB>,flags:PageTableFlags)->Result<MapperFlushAll,FlagUpdateError> {

        let p4_entry = &mut p4[page.p4_index()];

        if p4_entry.is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p4_entry.set_flags(flags);

        Ok(MapperFlushAll::new())
    }
pub fn set_flags_p3_entry_Size2MiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size2MiB>,flags:PageTableFlags)->Result<MapperFlushAll,FlagUpdateError> {


        if p4[page.p4_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        let p3 = table3;
        let p3_entry = &mut p3[page.p3_index()];

        if p3_entry.is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p3_entry.set_flags(flags);

        Ok(MapperFlushAll::new())
    }
pub fn set_flags_p2_entry_Size2MiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size2MiB>,flags:PageTableFlags)->Result<MapperFlushAll,FlagUpdateError> {
        Err(FlagUpdateError::ParentEntryHugePage)
    }
pub fn translate_page_Size2MiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size2MiB>)->Result<PhysFrame<Size2MiB>,TranslateError> {


        if p4[page.p4_index()].is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        let p3 = table3;
        let p3_entry = &p3[page.p3_index()];

        if p3_entry.is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        let p2 = table2;
        let p2_entry = &p2[page.p2_index()];

        if p2_entry.is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        PhysFrame::from_start_address(p2_entry.addr())
            .map_err(|AddressNotAligned| TranslateError::InvalidFrameAddress(p2_entry.addr()))
    }
pub fn map_to_with_table_flags_Size4KiB<A>(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size4KiB>,frame:PhysFrame<Size4KiB>,flags:PageTableFlags,parent_table_flags:PageTableFlags,allocator:&mut A)->Result<MapperFlush<Size4KiB>,MapToError<Size4KiB>> where A:FrameAllocator<Size4KiB>+?Sized {


        let p3_page = table3;
        let p3 = unsafe {
            create_next_table(
                &mut p4[page.p4_index()],
                p3_page,
                parent_table_flags,
                allocator,
            )?
        };

        let p2_page = table2;
        let p2 = unsafe {
            create_next_table(
                &mut p3[page.p3_index()],
                p2_page,
                parent_table_flags,
                allocator,
            )?
        };

        let p1_page = table1;
        let p1 = unsafe {
            create_next_table(
                &mut p2[page.p2_index()],
                p1_page,
                parent_table_flags,
                allocator,
            )?
        };

        if !p1[page.p1_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped(frame));
        }
        p1[page.p1_index()].set_frame(frame, flags);

        Ok(MapperFlush::new(page))
    }
pub fn unmap_Size4KiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size4KiB>)->Result<(PhysFrame<Size4KiB>,MapperFlush<Size4KiB>),UnmapError> {

        let p4_entry = &p4[page.p4_index()];
        p4_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;

        let p3 = table3;
        let p3_entry = &p3[page.p3_index()];
        p3_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;

        let p2 = table2;
        let p2_entry = &p2[page.p2_index()];
        p2_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;

        let p1 = table1;
        let p1_entry = &mut p1[page.p1_index()];

        let frame = p1_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;

        p1_entry.set_unused();
        Ok((frame, MapperFlush::new(page)))
    }
pub fn update_flags_Size4KiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size4KiB>,flags:PageTableFlags)->Result<MapperFlush<Size4KiB>,FlagUpdateError> {


        if p4[page.p4_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        let p3 = table3;

        if p3[page.p3_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        let p2 = table2;

        if p2[page.p2_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        let p1 = table1;

        if p1[page.p1_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p1[page.p1_index()].set_flags(flags);

        Ok(MapperFlush::new(page))
    }
pub fn set_flags_p4_entry_Size4KiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size4KiB>,flags:PageTableFlags)->Result<MapperFlushAll,FlagUpdateError> {

        let p4_entry = &mut p4[page.p4_index()];

        if p4_entry.is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p4_entry.set_flags(flags);

        Ok(MapperFlushAll::new())
    }
pub fn set_flags_p3_entry_Size4KiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size4KiB>,flags:PageTableFlags)->Result<MapperFlushAll,FlagUpdateError> {


        if p4[page.p4_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        let p3 = table3;
        let p3_entry = &mut p3[page.p3_index()];

        if p3_entry.is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p3_entry.set_flags(flags);

        Ok(MapperFlushAll::new())
    }
pub fn set_flags_p2_entry_Size4KiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size4KiB>,flags:PageTableFlags)->Result<MapperFlushAll,FlagUpdateError> {


        if p4[page.p4_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        let p3 = table3;

        if p3[page.p3_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        let p2 = table2;
        let p2_entry = &mut p2[page.p2_index()];

        if p2_entry.is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p2_entry.set_flags(flags);

        Ok(MapperFlushAll::new())
    }
pub fn translate_page_Size4KiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size4KiB>)->Result<PhysFrame<Size4KiB>,TranslateError> {


        if p4[page.p4_index()].is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        let p3 = table3;
        let p3_entry = &p3[page.p3_index()];

        if p3_entry.is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        let p2 = table2;
        let p2_entry = &p2[page.p2_index()];

        if p2_entry.is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        let p1 = table1;
        let p1_entry = &p1[page.p1_index()];

        if p1_entry.is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        PhysFrame::from_start_address(p1_entry.addr())
            .map_err(|AddressNotAligned| TranslateError::InvalidFrameAddress(p1_entry.addr()))
    }
pub fn translate(p4:&PageTable,table3:&PageTable,table2:&PageTable,table1:&PageTable,addr:VirtAddr)->TranslateResult {
        let page: Page<Size4KiB> = Page::containing_address(addr);


        let p4_entry = &p4[addr.p4_index()];
        if p4_entry.is_unused() {
            return TranslateResult::NotMapped;
        }
        if p4_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
            panic!("level 4 entry has huge page bit set")
        }

        let p3 = table3;
        let p3_entry = &p3[addr.p3_index()];
        if p3_entry.is_unused() {
            return TranslateResult::NotMapped;
        }
        if p3_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
            let entry = &p3[addr.p3_index()];
            let frame = PhysFrame::containing_address(entry.addr());
            #[allow(clippy::unusual_byte_groupings)]
            let offset = addr.as_u64() & 0o_777_777_7777;
            let flags = entry.flags();
            return TranslateResult::Mapped {
                frame: MappedFrame::Size1GiB(frame),
                offset,
                flags,
            };
        }

        let p2 = table2;
        let p2_entry = &p2[addr.p2_index()];
        if p2_entry.is_unused() {
            return TranslateResult::NotMapped;
        }
        if p2_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
            let entry = &p2[addr.p2_index()];
            let frame = PhysFrame::containing_address(entry.addr());
            #[allow(clippy::unusual_byte_groupings)]
            let offset = addr.as_u64() & 0o_777_7777;
            let flags = entry.flags();
            return TranslateResult::Mapped {
                frame: MappedFrame::Size2MiB(frame),
                offset,
                flags,
            };
        }

        let p1 = table1;
        let p1_entry = &p1[addr.p1_index()];
        if p1_entry.is_unused() {
            return TranslateResult::NotMapped;
        }
        if p1_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
            panic!("level 1 entry has huge page bit set")
        }

        let frame = match PhysFrame::from_start_address(p1_entry.addr()) {
            Ok(frame) => frame,
            Err(AddressNotAligned) => return TranslateResult::InvalidFrameAddress(p1_entry.addr()),
        };
        let offset = u64::from(addr.page_offset());
        let flags = p1_entry.flags();
        TranslateResult::Mapped {
            frame: MappedFrame::Size4KiB(frame),
            offset,
            flags,
        }
    }
