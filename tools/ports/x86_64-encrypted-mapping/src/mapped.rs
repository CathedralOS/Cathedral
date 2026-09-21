// SPDX-License-Identifier: MIT OR Apache-2.0
// Generated pinned branch bodies; borrowed snapshots replace all raw topology access.
#![allow(unused_variables,unused_unsafe,unused_imports,non_snake_case)]
use x86_64::structures::paging::{Page,PageSize,Size4KiB,Size2MiB,Size1GiB,PhysFrame,PageTableFlags,page::AddressNotAligned,page_table::FrameError,mapper::{MapToError,UnmapError,FlagUpdateError,TranslateError,MapperFlush,MapperFlushAll}};
use crate::{ObservedTable as PageTable,ObservedEntry as PageTableEntry,AllocationObservations as FrameAllocator};
#[derive(Debug)]
enum PageTableWalkError {
    NotMapped,
    MappedToHugePage,
}

#[derive(Debug)]
enum PageTableCreateError {
    MappedToHugePage,
    FrameAllocationFailed,
}

impl From<PageTableCreateError> for MapToError<Size4KiB> {
    #[inline]
    fn from(err: PageTableCreateError) -> Self {
        match err {
            PageTableCreateError::MappedToHugePage => MapToError::ParentEntryHugePage,
            PageTableCreateError::FrameAllocationFailed => MapToError::FrameAllocationFailed,
        }
    }
}

impl From<PageTableCreateError> for MapToError<Size2MiB> {
    #[inline]
    fn from(err: PageTableCreateError) -> Self {
        match err {
            PageTableCreateError::MappedToHugePage => MapToError::ParentEntryHugePage,
            PageTableCreateError::FrameAllocationFailed => MapToError::FrameAllocationFailed,
        }
    }
}

impl From<PageTableCreateError> for MapToError<Size1GiB> {
    #[inline]
    fn from(err: PageTableCreateError) -> Self {
        match err {
            PageTableCreateError::MappedToHugePage => MapToError::ParentEntryHugePage,
            PageTableCreateError::FrameAllocationFailed => MapToError::FrameAllocationFailed,
        }
    }
}

impl From<FrameError> for PageTableWalkError {
    #[inline]
    fn from(err: FrameError) -> Self {
        match err {
            FrameError::HugeFrame => PageTableWalkError::MappedToHugePage,
            FrameError::FrameNotPresent => PageTableWalkError::NotMapped,
        }
    }
}

impl From<PageTableWalkError> for UnmapError {
    #[inline]
    fn from(err: PageTableWalkError) -> Self {
        match err {
            PageTableWalkError::MappedToHugePage => UnmapError::ParentEntryHugePage,
            PageTableWalkError::NotMapped => UnmapError::PageNotMapped,
        }
    }
}

impl From<PageTableWalkError> for FlagUpdateError {
    #[inline]
    fn from(err: PageTableWalkError) -> Self {
        match err {
            PageTableWalkError::MappedToHugePage => FlagUpdateError::ParentEntryHugePage,
            PageTableWalkError::NotMapped => FlagUpdateError::PageNotMapped,
        }
    }
}

impl From<PageTableWalkError> for TranslateError {
    #[inline]
    fn from(err: PageTableWalkError) -> Self {
        match err {
            PageTableWalkError::MappedToHugePage => TranslateError::ParentEntryHugePage,
            PageTableWalkError::NotMapped => TranslateError::PageNotMapped,
        }
    }
}

fn next_table<'b>(table:&'b PageTable,entry:&PageTableEntry)->Result<&'b PageTable,PageTableWalkError>{
        entry.frame()?;
        let page_table = table;

        Ok(page_table)
    }
fn next_table_mut<'b>(table:&'b mut PageTable,entry:&mut PageTableEntry)->Result<&'b mut PageTable,PageTableWalkError>{
        entry.frame()?;
        let page_table = table;

        Ok(page_table)
    }
fn create_next_table<'b,A>(table:&'b mut PageTable,entry:&mut PageTableEntry,insert_flags:PageTableFlags,allocator:&mut A)->Result<&'b mut PageTable,PageTableCreateError> where A:FrameAllocator<Size4KiB>+?Sized {
        let created;

        if entry.is_unused() {
            if let Some(frame) = allocator.allocate_frame() {
                entry.set_frame(frame, insert_flags);
                created = true;
            } else {
                return Err(PageTableCreateError::FrameAllocationFailed);
            }
        } else {
            if !insert_flags.is_empty() && !entry.flags().contains(insert_flags) {
                entry.set_flags(entry.flags() | insert_flags);
            }
            created = false;
        }

        let page_table = match next_table_mut(table,entry) {
            Err(PageTableWalkError::MappedToHugePage) => {
                return Err(PageTableCreateError::MappedToHugePage);
            }
            Err(PageTableWalkError::NotMapped) => panic!("entry should be mapped at this point"),
            Ok(page_table) => page_table,
        };

        if created {
            page_table.zero();
        }
        Ok(page_table)
    }
pub fn map_to_with_table_flags_Size1GiB<A>(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size1GiB>,frame:PhysFrame<Size1GiB>,flags:PageTableFlags,parent_table_flags:PageTableFlags,allocator:&mut A)->Result<MapperFlush<Size1GiB>,MapToError<Size1GiB>> where A:FrameAllocator<Size4KiB>+?Sized{

        let p3 = create_next_table(table3,
            &mut p4[page.p4_index()],
            parent_table_flags,
            allocator,
        )?;

        if !p3[page.p3_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped(frame));
        }
        p3[page.p3_index()].set_addr(frame.start_address(), flags | PageTableFlags::HUGE_PAGE);

        Ok(MapperFlush::new(page))
    }
pub fn unmap_Size1GiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size1GiB>)->Result<(PhysFrame<Size1GiB>,MapperFlush<Size1GiB>),UnmapError>{

        let p3 = next_table_mut(table3, &mut p4[page.p4_index()])?;

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
pub fn update_flags_Size1GiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size1GiB>,flags:PageTableFlags)->Result<MapperFlush<Size1GiB>,FlagUpdateError>{

        let p3 = next_table_mut(table3, &mut p4[page.p4_index()])?;

        if p3[page.p3_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }
        p3[page.p3_index()].set_flags(flags | PageTableFlags::HUGE_PAGE);

        Ok(MapperFlush::new(page))
    }
pub fn set_flags_p4_entry_Size1GiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size1GiB>,flags:PageTableFlags)->Result<MapperFlushAll,FlagUpdateError>{

        let p4_entry = &mut p4[page.p4_index()];

        if p4_entry.is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p4_entry.set_flags(flags);

        Ok(MapperFlushAll::new())
    }
pub fn set_flags_p3_entry_Size1GiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size1GiB>,flags:PageTableFlags)->Result<MapperFlushAll,FlagUpdateError>{
        Err(FlagUpdateError::ParentEntryHugePage)
    }
pub fn set_flags_p2_entry_Size1GiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size1GiB>,flags:PageTableFlags)->Result<MapperFlushAll,FlagUpdateError>{
        Err(FlagUpdateError::ParentEntryHugePage)
    }
pub fn translate_page_Size1GiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size1GiB>)->Result<PhysFrame<Size1GiB>,TranslateError>{

        let p3 = next_table(table3, &p4[page.p4_index()])?;

        let p3_entry = &p3[page.p3_index()];

        if p3_entry.is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        PhysFrame::from_start_address(p3_entry.addr())
            .map_err(|AddressNotAligned| TranslateError::InvalidFrameAddress(p3_entry.addr()))
    }
pub fn map_to_with_table_flags_Size2MiB<A>(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size2MiB>,frame:PhysFrame<Size2MiB>,flags:PageTableFlags,parent_table_flags:PageTableFlags,allocator:&mut A)->Result<MapperFlush<Size2MiB>,MapToError<Size2MiB>> where A:FrameAllocator<Size4KiB>+?Sized{

        let p3 = create_next_table(table3,
            &mut p4[page.p4_index()],
            parent_table_flags,
            allocator,
        )?;
        let p2 = create_next_table(table2,
            &mut p3[page.p3_index()],
            parent_table_flags,
            allocator,
        )?;

        if !p2[page.p2_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped(frame));
        }
        p2[page.p2_index()].set_addr(frame.start_address(), flags | PageTableFlags::HUGE_PAGE);

        Ok(MapperFlush::new(page))
    }
pub fn unmap_Size2MiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size2MiB>)->Result<(PhysFrame<Size2MiB>,MapperFlush<Size2MiB>),UnmapError>{

        let p3 = next_table_mut(table3, &mut p4[page.p4_index()])?;
        let p2 = next_table_mut(table2, &mut p3[page.p3_index()])?;

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
pub fn update_flags_Size2MiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size2MiB>,flags:PageTableFlags)->Result<MapperFlush<Size2MiB>,FlagUpdateError>{

        let p3 = next_table_mut(table3, &mut p4[page.p4_index()])?;
        let p2 = next_table_mut(table2, &mut p3[page.p3_index()])?;

        if p2[page.p2_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p2[page.p2_index()].set_flags(flags | PageTableFlags::HUGE_PAGE);

        Ok(MapperFlush::new(page))
    }
pub fn set_flags_p4_entry_Size2MiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size2MiB>,flags:PageTableFlags)->Result<MapperFlushAll,FlagUpdateError>{

        let p4_entry = &mut p4[page.p4_index()];

        if p4_entry.is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p4_entry.set_flags(flags);

        Ok(MapperFlushAll::new())
    }
pub fn set_flags_p3_entry_Size2MiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size2MiB>,flags:PageTableFlags)->Result<MapperFlushAll,FlagUpdateError>{

        let p3 = next_table_mut(table3, &mut p4[page.p4_index()])?;
        let p3_entry = &mut p3[page.p3_index()];

        if p3_entry.is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p3_entry.set_flags(flags);

        Ok(MapperFlushAll::new())
    }
pub fn set_flags_p2_entry_Size2MiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size2MiB>,flags:PageTableFlags)->Result<MapperFlushAll,FlagUpdateError>{
        Err(FlagUpdateError::ParentEntryHugePage)
    }
pub fn translate_page_Size2MiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size2MiB>)->Result<PhysFrame<Size2MiB>,TranslateError>{

        let p3 = next_table(table3, &p4[page.p4_index()])?;
        let p2 = next_table(table2, &p3[page.p3_index()])?;

        let p2_entry = &p2[page.p2_index()];

        if p2_entry.is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        PhysFrame::from_start_address(p2_entry.addr())
            .map_err(|AddressNotAligned| TranslateError::InvalidFrameAddress(p2_entry.addr()))
    }
pub fn map_to_with_table_flags_Size4KiB<A>(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size4KiB>,frame:PhysFrame<Size4KiB>,flags:PageTableFlags,parent_table_flags:PageTableFlags,allocator:&mut A)->Result<MapperFlush<Size4KiB>,MapToError<Size4KiB>> where A:FrameAllocator<Size4KiB>+?Sized{

        let p3 = create_next_table(table3,
            &mut p4[page.p4_index()],
            parent_table_flags,
            allocator,
        )?;
        let p2 = create_next_table(table2,
            &mut p3[page.p3_index()],
            parent_table_flags,
            allocator,
        )?;
        let p1 = create_next_table(table1,
            &mut p2[page.p2_index()],
            parent_table_flags,
            allocator,
        )?;

        if !p1[page.p1_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped(frame));
        }
        p1[page.p1_index()].set_frame(frame, flags);

        Ok(MapperFlush::new(page))
    }
pub fn unmap_Size4KiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size4KiB>)->Result<(PhysFrame<Size4KiB>,MapperFlush<Size4KiB>),UnmapError>{

        let p3 = next_table_mut(table3, &mut p4[page.p4_index()])?;
        let p2 = next_table_mut(table2, &mut p3[page.p3_index()])?;
        let p1 = next_table_mut(table1, &mut p2[page.p2_index()])?;

        let p1_entry = &mut p1[page.p1_index()];

        let frame = p1_entry.frame().map_err(|err| match err {
            FrameError::FrameNotPresent => UnmapError::PageNotMapped,
            FrameError::HugeFrame => UnmapError::ParentEntryHugePage,
        })?;

        p1_entry.set_unused();
        Ok((frame, MapperFlush::new(page)))
    }
pub fn update_flags_Size4KiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size4KiB>,flags:PageTableFlags)->Result<MapperFlush<Size4KiB>,FlagUpdateError>{

        let p3 = next_table_mut(table3, &mut p4[page.p4_index()])?;
        let p2 = next_table_mut(table2, &mut p3[page.p3_index()])?;
        let p1 = next_table_mut(table1, &mut p2[page.p2_index()])?;

        if p1[page.p1_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p1[page.p1_index()].set_flags(flags);

        Ok(MapperFlush::new(page))
    }
pub fn set_flags_p4_entry_Size4KiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size4KiB>,flags:PageTableFlags)->Result<MapperFlushAll,FlagUpdateError>{

        let p4_entry = &mut p4[page.p4_index()];

        if p4_entry.is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p4_entry.set_flags(flags);

        Ok(MapperFlushAll::new())
    }
pub fn set_flags_p3_entry_Size4KiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size4KiB>,flags:PageTableFlags)->Result<MapperFlushAll,FlagUpdateError>{

        let p3 = next_table_mut(table3, &mut p4[page.p4_index()])?;
        let p3_entry = &mut p3[page.p3_index()];

        if p3_entry.is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p3_entry.set_flags(flags);

        Ok(MapperFlushAll::new())
    }
pub fn set_flags_p2_entry_Size4KiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size4KiB>,flags:PageTableFlags)->Result<MapperFlushAll,FlagUpdateError>{

        let p3 = next_table_mut(table3, &mut p4[page.p4_index()])?;
        let p2 = next_table_mut(table2, &mut p3[page.p3_index()])?;
        let p2_entry = &mut p2[page.p2_index()];

        if p2_entry.is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p2_entry.set_flags(flags);

        Ok(MapperFlushAll::new())
    }
pub fn translate_page_Size4KiB(p4:&mut PageTable,table3:&mut PageTable,table2:&mut PageTable,table1:&mut PageTable,page:Page<Size4KiB>)->Result<PhysFrame<Size4KiB>,TranslateError>{

        let p3 = next_table(table3, &p4[page.p4_index()])?;
        let p2 = next_table(table2, &p3[page.p3_index()])?;
        let p1 = next_table(table1, &p2[page.p2_index()])?;

        let p1_entry = &p1[page.p1_index()];

        if p1_entry.is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        PhysFrame::from_start_address(p1_entry.addr())
            .map_err(|AddressNotAligned| TranslateError::InvalidFrameAddress(p1_entry.addr()))
    }
