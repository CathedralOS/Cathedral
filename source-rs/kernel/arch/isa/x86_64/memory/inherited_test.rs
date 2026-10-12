//! Synthetic firmware aliases exercise huge-page splitting without privileged instructions.
extern crate std;
use super::{Frames, inherited};
use crate::{
    ImageRange,
    memory::{ImageSection, KernelImage, Permissions},
};
use ::x86_64::{
    PhysAddr,
    structures::paging::{PageTable, PageTableFlags as Flags, PhysFrame},
};
use std::{boxed::Box, vec::Vec};

#[test]
fn every_alias_obeys_sections_and_huge_pat_survives_splitting() {
    let sections = [
        ImageSection {
            range: ImageRange {
                base: 0x201000,
                bytes: 4096,
            },
            permissions: Permissions {
                writable: false,
                executable: true,
            },
        },
        ImageSection {
            range: ImageRange {
                base: 0x202000,
                bytes: 4096,
            },
            permissions: Permissions {
                writable: true,
                executable: false,
            },
        },
    ];
    let image = KernelImage::new(
        ImageRange {
            base: 0x200000,
            bytes: 0x4000,
        },
        &sections,
    )
    .unwrap();
    let mut source = Box::new(PageTable::new());
    // Two virtual 1-GiB mappings of the same physical range, originally user/RWX/global.
    for index in [0, 1] {
        source[index].set_addr(
            PhysAddr::new(1 << 12),
            Flags::PRESENT
                | Flags::WRITABLE
                | Flags::HUGE_PAGE
                | Flags::GLOBAL
                | Flags::USER_ACCESSIBLE,
        );
    }
    let mut allocations = Vec::new();
    let mut next = || {
        let mut table = Box::new(PageTable::new());
        let address = (&mut *table as *mut PageTable) as u64;
        allocations.push(table);
        Some(address)
    };
    let mut frames = Frames {
        next: &mut next,
        tables: 0,
    };
    let physical =
        PhysFrame::from_start_address(PhysAddr::new((&*source as *const PageTable) as u64))
            .unwrap();
    // SAFETY: Aligned boxed source and uniquely owned destination tables remain live for the test.
    let root = unsafe { inherited::copy(physical, 3, &image, &mut frames) }.unwrap();
    assert_eq!(frames.tables, 5); // Root plus two levels per alias; unaffected blocks stay huge.
    for alias in [0, 1 << 30] {
        for (offset, writable, executable) in [
            (0x200000, false, false),
            (0x201000, false, true),
            (0x202000, true, false),
            (0x203000, false, false),
        ] {
            let (address, flags, level) = leaf(root, alias + offset);
            assert_eq!(address, offset);
            assert_eq!(level, 1);
            assert_eq!(flags.contains(Flags::WRITABLE), writable);
            assert_eq!(!flags.contains(Flags::NO_EXECUTE), executable);
            assert!(!flags.intersects(Flags::USER_ACCESSIBLE | Flags::GLOBAL));
            assert!(flags.contains(Flags::HUGE_PAGE)); // At level 1 bit 7 is PAT.
        }
        let (address, flags, level) = leaf(root, alias + 0x400000);
        assert_eq!(address, 0x401000); // Unsplit level-2 leaf retains PAT at bit 12.
        assert_eq!(level, 2);
        assert!(flags.contains(Flags::WRITABLE | Flags::NO_EXECUTE | Flags::HUGE_PAGE));
    }
    assert!(
        source[0]
            .flags()
            .contains(Flags::GLOBAL | Flags::USER_ACCESSIBLE)
    );
}
fn leaf(root: PhysFrame, address: u64) -> (u64, Flags, u8) {
    let mut table = root.start_address().as_u64();
    for level in (1..=3).rev() {
        // SAFETY: Test holds every source/destination table until traversal finishes.
        let entry = unsafe {
            &(&*(table as *const PageTable))[((address >> (12 + 9 * (level - 1))) & 511) as usize]
        };
        let flags = entry.flags();
        assert!(flags.contains(Flags::PRESENT));
        if level == 1 || flags.contains(Flags::HUGE_PAGE) {
            return (entry.addr().as_u64(), flags, level);
        }
        assert!(!flags.intersects(Flags::USER_ACCESSIBLE | Flags::NO_EXECUTE));
        table = entry.addr().as_u64();
    }
    unreachable!()
}
