//! Inspect actual table encodings with the independent x86 crate translator.
extern crate std;
use super::{Flags, PAGE, PageTable, VirtAddr, regions};
use crate::MemoryError;
use std::{boxed::Box, vec::Vec};
use x86_64::{
    PhysAddr,
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PhysFrame, Size2MiB, Size4KiB, Translate,
        mapper::TranslateResult,
    },
};

#[derive(Default)]
struct Tables {
    // Boxes retain aligned stable addresses while the ownership vector grows.
    #[allow(clippy::vec_box)]
    owned: Vec<Box<PageTable>>,
}

fn snapshot(root: &PageTable, tables: &Tables) -> Vec<(u64, Flags)> {
    root.iter()
        .chain(tables.owned.iter().flat_map(|table| table.iter()))
        .map(|entry| (entry.addr().as_u64(), entry.flags()))
        .collect()
}

const RW: Flags = Flags::PRESENT
    .union(Flags::USER_ACCESSIBLE)
    .union(Flags::WRITABLE)
    .union(Flags::NO_EXECUTE);

#[test]
fn reserved_install_rejects_later_huge_leaf_without_treating_it_as_a_table() {
    let base = (1 << 21) - 3 * PAGE;
    let mut root = Box::new(PageTable::new());
    let mut tables = Tables::default();
    // SAFETY: Private host tables; synthetic huge-page backing must never be
    // dereferenced as a child table by the runtime installer.
    unsafe {
        put(&mut root, &mut tables, base - PAGE, RW);
        OffsetPageTable::new(&mut root, VirtAddr::zero())
            .map_to(
                Page::<Size2MiB>::from_start_address(VirtAddr::new(1 << 21)).unwrap(),
                PhysFrame::from_start_address(PhysAddr::new(1 << 22)).unwrap(),
                RW,
                &mut tables,
            )
            .unwrap()
            .ignore();
        let before = snapshot(&root, &tables);
        assert_eq!(
            regions::install((&raw mut *root) as u64, base, &[0x20_0000; 128], true),
            Err(MemoryError::OccupiedVirtualRange)
        );
        assert_eq!(snapshot(&root, &tables), before);
    }
}

unsafe fn put(root: &mut PageTable, tables: &mut Tables, address: u64, flags: Flags) {
    // SAFETY: Tests own all table allocations; synthetic backing is never dereferenced.
    unsafe {
        OffsetPageTable::new(root, VirtAddr::zero())
            .map_to(
                Page::<Size4KiB>::from_start_address(VirtAddr::new(address)).unwrap(),
                PhysFrame::from_start_address(PhysAddr::new(0x10_0000)).unwrap(),
                flags,
                tables,
            )
            .unwrap()
            .ignore();
    }
}

#[test]
fn reserved_install_preserves_backing_permissions_and_neighbors_across_levels() {
    for base in [
        7 * PAGE,
        (1 << 21) - 3 * PAGE,
        (1 << 30) - 3 * PAGE,
        (1 << 39) - 3 * PAGE,
    ] {
        let mut root = Box::new(PageTable::new());
        let mut tables = Tables::default();
        let physical: Vec<_> = (0..128).map(|index| 0x20_0000 + index * PAGE).collect();
        // SAFETY: Owned inactive host tables, never-accessed synthetic backing.
        unsafe {
            put(&mut root, &mut tables, base - PAGE, RW);
            put(&mut root, &mut tables, base + 128 * PAGE, RW);
            let before = snapshot(&root, &tables);
            for writable in [false, true] {
                regions::install((&raw mut *root) as u64, base, &physical, writable).unwrap();
                {
                    let mapper = OffsetPageTable::new(&mut root, VirtAddr::zero());
                    for (index, &expected) in physical.iter().enumerate() {
                        let TranslateResult::Mapped {
                            frame,
                            offset,
                            flags,
                        } = mapper.translate(VirtAddr::new(base + index as u64 * PAGE))
                        else {
                            panic!("installation lost a page");
                        };
                        assert_eq!(frame.start_address().as_u64(), expected);
                        assert_eq!(offset, 0);
                        assert_eq!(flags, if writable { RW } else { RW & !Flags::WRITABLE });
                    }
                }
                regions::edit((&raw mut *root) as u64, base, physical.len(), false);
                assert_eq!(snapshot(&root, &tables), before);
            }
        }
    }
}

#[test]
fn reserved_install_refuses_every_collision_without_touching_any_table() {
    let base = (1 << 21) - 3 * PAGE;
    let mut root = Box::new(PageTable::new());
    let mut tables = Tables::default();
    let physical: Vec<_> = (0..128).map(|index| 0x20_0000 + index * PAGE).collect();
    // SAFETY: Private host fixture, all tables retained across refusal and retry.
    unsafe {
        put(&mut root, &mut tables, base - PAGE, RW);
        put(&mut root, &mut tables, base + 128 * PAGE, RW);
        for collision in 0..128 {
            let address = base + collision * PAGE;
            put(&mut root, &mut tables, address, RW & !Flags::WRITABLE);
            let before = snapshot(&root, &tables);
            assert_eq!(
                regions::install((&raw mut *root) as u64, base, &physical, true),
                Err(MemoryError::OccupiedVirtualRange)
            );
            assert_eq!(snapshot(&root, &tables), before);
            regions::edit((&raw mut *root) as u64, address, 1, false);
        }
        let before = snapshot(&root, &tables);
        for bad in [0, PAGE + 1, 1 << 63] {
            let mut invalid = physical.clone();
            invalid[127] = bad;
            assert_eq!(
                regions::install((&raw mut *root) as u64, base, &invalid, true),
                Err(MemoryError::InvalidFrame)
            );
            assert_eq!(snapshot(&root, &tables), before);
        }
        assert_eq!(
            regions::install((&raw mut *root) as u64, base, &[], true),
            Err(MemoryError::InvalidFrame)
        );
        assert_eq!(
            regions::install((&raw mut *root) as u64, base + 1, &physical, true),
            Err(MemoryError::InvalidFrame)
        );
        assert_eq!(snapshot(&root, &tables), before);
        regions::install((&raw mut *root) as u64, base, &physical, true).unwrap();
        regions::edit((&raw mut *root) as u64, base, physical.len(), false);
        assert_eq!(snapshot(&root, &tables), before);
    }
}

#[test]
fn late_missing_or_readonly_parent_refuses_without_partial_install_or_widening() {
    let base = (1 << 21) - 3 * PAGE;
    for readonly in [false, true] {
        let mut root = Box::new(PageTable::new());
        let mut tables = Tables::default();
        // SAFETY: Fresh private fixture. Only the first leaf table is required
        // initially; the later one is either absent or deliberately read-only.
        unsafe {
            put(&mut root, &mut tables, base - PAGE, RW);
            if readonly {
                put(
                    &mut root,
                    &mut tables,
                    base + 128 * PAGE,
                    RW & !Flags::WRITABLE,
                );
            }
            let before = snapshot(&root, &tables);
            let physical = [0x20_0000; 128];
            assert_eq!(
                regions::install((&raw mut *root) as u64, base, &physical, true),
                Err(if readonly {
                    MemoryError::OccupiedVirtualRange
                } else {
                    MemoryError::OutOfFrames
                })
            );
            assert_eq!(snapshot(&root, &tables), before);
            if readonly {
                regions::install((&raw mut *root) as u64, base, &physical, false).unwrap();
                regions::edit((&raw mut *root) as u64, base, physical.len(), false);
                assert_eq!(snapshot(&root, &tables), before);
            }
        }
    }
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
