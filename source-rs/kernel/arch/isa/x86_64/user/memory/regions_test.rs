//! Inspect actual table encodings with the independent x86 crate translator.
extern crate std;
use super::{Flags, PAGE, PageTable, VirtAddr, regions};
use std::{boxed::Box, vec::Vec};
use x86_64::{
    PhysAddr,
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PhysFrame, Size4KiB, Translate,
        mapper::TranslateResult,
    },
};

#[derive(Default)]
struct Tables {
    // Boxes retain aligned stable addresses while the ownership vector grows.
    #[allow(clippy::vec_box)]
    owned: Vec<Box<PageTable>>,
}
// SAFETY: Every table is a fresh aligned allocation retained throughout the test.
unsafe impl FrameAllocator<Size4KiB> for Tables {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let mut table = Box::new(PageTable::new());
        let address = (&raw mut *table) as u64;
        self.owned.push(table);
        Some(PhysFrame::from_start_address(PhysAddr::new(address)).unwrap())
    }
}

#[test]
fn bulk_edits_cross_each_table_level_without_touching_neighbors() {
    // One leaf, then ranges crossing PT, PD and PDPT boundaries. Nonzero leaf
    // offsets catch assumptions that the first table starts at entry zero.
    for (base, pages) in [
        (7 * PAGE, 1),
        ((1 << 21) - 3 * PAGE, 128),
        ((1 << 30) - 3 * PAGE, 128),
        ((1 << 39) - 3 * PAGE, 128),
    ] {
        let mut root = Box::new(PageTable::new());
        let root_address = (&raw mut *root) as u64;
        let mut tables = Tables::default();
        let original =
            Flags::PRESENT | Flags::USER_ACCESSIBLE | Flags::WRITABLE | Flags::NO_EXECUTE;
        // SAFETY: Host allocations model exclusively owned inactive identity tables.
        // Synthetic backing is never dereferenced; only translation entries are read.
        unsafe {
            {
                let mut mapper = OffsetPageTable::new(&mut root, VirtAddr::zero());
                for index in 0..pages + 2 {
                    let address = base - PAGE + index as u64 * PAGE;
                    mapper
                        .map_to(
                            Page::<Size4KiB>::from_start_address(VirtAddr::new(address)).unwrap(),
                            PhysFrame::from_start_address(PhysAddr::new(
                                0x10_0000 + index as u64 * PAGE,
                            ))
                            .unwrap(),
                            original,
                            &mut tables,
                        )
                        .unwrap()
                        .ignore();
                }
            }
            let allocated = tables.owned.len();
            regions::edit(root_address, base, pages, true);
            {
                let mapper = OffsetPageTable::new(&mut root, VirtAddr::zero());
                for index in 0..pages + 2 {
                    let address = base - PAGE + index as u64 * PAGE;
                    let TranslateResult::Mapped {
                        frame,
                        offset,
                        flags,
                    } = mapper.translate(VirtAddr::new(address))
                    else {
                        panic!("sealing lost a mapping");
                    };
                    assert_eq!(
                        frame.start_address().as_u64(),
                        0x10_0000 + index as u64 * PAGE
                    );
                    assert_eq!(offset, 0);
                    let neighbor = index == 0 || index == pages + 1;
                    assert_eq!(
                        flags,
                        if neighbor {
                            original
                        } else {
                            original & !Flags::WRITABLE
                        }
                    );
                }
            }
            regions::edit(root_address, base, pages, false);
            {
                let mapper = OffsetPageTable::new(&mut root, VirtAddr::zero());
                for index in 0..pages + 2 {
                    let address = base - PAGE + index as u64 * PAGE;
                    let translation = mapper.translate(VirtAddr::new(address));
                    if index == 0 || index == pages + 1 {
                        let TranslateResult::Mapped { flags, frame, .. } = translation else {
                            panic!("unmapping lost a neighbor");
                        };
                        assert_eq!(flags, original);
                        assert_eq!(
                            frame.start_address().as_u64(),
                            0x10_0000 + index as u64 * PAGE
                        );
                    } else {
                        assert!(matches!(translation, TranslateResult::NotMapped));
                    }
                }
            }
            assert_eq!(tables.owned.len(), allocated);
        }
    }
}
