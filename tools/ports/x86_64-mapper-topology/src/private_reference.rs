// SPDX-License-Identifier: MIT OR Apache-2.0
// Generated from x86_64 cc35c876d3badb57df54a66e22f7768a52be95f2.
// Private coordinate bodies are exact copies, not public upstream API calls.
use x86_64::structures::paging::page::NotGiantPageSize;


fn p3_page<S: PageSize>(page: Page<S>, recursive_index: PageTableIndex) -> Page {
    Page::from_page_table_indices(
        recursive_index,
        recursive_index,
        recursive_index,
        page.p4_index(),
    )
}

fn p2_page<S: NotGiantPageSize>(page: Page<S>, recursive_index: PageTableIndex) -> Page {
    Page::from_page_table_indices(
        recursive_index,
        recursive_index,
        page.p4_index(),
        page.p3_index(),
    )
}

fn p1_page(page: Page<Size4KiB>, recursive_index: PageTableIndex) -> Page {
    Page::from_page_table_indices(
        recursive_index,
        page.p4_index(),
        page.p3_index(),
        page.p2_index(),
    )
}

// Observation mirror only: pointer acquisition -> supplied VirtAddr, CR3 read
// -> supplied PhysFrame, table indexing -> supplied entry, final Self -> index.
#[derive(Debug)] enum ObservedError { NotRecursive, NotActive }
fn observed_constructor(table_address:VirtAddr,observed_cr3:PhysFrame,recursive_entry:&PageTableEntry)->Result<PageTableIndex,ObservedError> {
        let page = Page::containing_address(table_address);
        let recursive_index = page.p4_index();

        if page.p3_index() != recursive_index
            || page.p2_index() != recursive_index
            || page.p1_index() != recursive_index
        {
            return Err(ObservedError::NotRecursive);
        }
        if Ok(observed_cr3) != recursive_entry.frame() {
            return Err(ObservedError::NotActive);
        }

        Ok(recursive_index)
}

// Numeric expression extracted from private PhysOffset::frame_to_pointer.
// The following as_mut_ptr operation is deliberately absent.
fn offset_numeric(offset:VirtAddr,frame:PhysFrame)->VirtAddr {
    let virt = offset + frame.start_address().as_u64();
    virt
}
