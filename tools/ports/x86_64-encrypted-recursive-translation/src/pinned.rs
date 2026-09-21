// SPDX-License-Identifier: MIT OR Apache-2.0
// Exact recursive translate body at cc35c876d3badb57df54a66e22f7768a52be95f2,
// replacing only topology resolution with disjoint initialized table borrows.
#![allow(unused_variables)]
use x86_64::VirtAddr;
use x86_64::structures::paging::{Page,Size4KiB,PageTable,PageTableFlags,PhysFrame,page::AddressNotAligned,mapper::{TranslateResult,MappedFrame}};
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
