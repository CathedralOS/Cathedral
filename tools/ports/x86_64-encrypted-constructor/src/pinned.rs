// SPDX-License-Identifier: MIT OR Apache-2.0
// Exact constructor checks with explicit numeric observation substitutions.
#[derive(Debug)] enum ObservedError {NotRecursive,NotActive}
fn observed_constructor(table_address:VirtAddr,observed:u64,recursive_entry:&PageTableEntry)->Result<PageTableIndex,ObservedError> {
        let page = Page::containing_address(table_address);
        let recursive_index = page.p4_index();

        if page.p3_index() != recursive_index
            || page.p2_index() != recursive_index
            || page.p1_index() != recursive_index
        {
            return Err(ObservedError::NotRecursive);
        }
        if Ok(observed_frame(observed)) != recursive_entry.frame() {
            return Err(ObservedError::NotActive);
        }

    Ok(recursive_index)
}
