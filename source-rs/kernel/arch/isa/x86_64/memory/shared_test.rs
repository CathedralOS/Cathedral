//! The image's identity leaves become global; firmware aliases of its frames do not.
extern crate std;
use super::{Frames, inherited, shared};
use crate::{
    ImageRange,
    memory::{ImageSection, KernelImage, Permissions},
};
use ::x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{OffsetPageTable, PageTable, PageTableFlags as Flags, PhysFrame},
};
use std::{boxed::Box, vec::Vec};

#[test]
fn identity_image_leaves_are_global_and_aliases_stay_per_root() {
    let sections = [ImageSection {
        range: ImageRange {
            base: 0x201000,
            bytes: 4096,
        },
        permissions: Permissions {
            writable: false,
            executable: true,
        },
    }];
    let image = KernelImage::new(
        ImageRange {
            base: 0x200000,
            bytes: 0x4000,
        },
        &sections,
    )
    .unwrap();
    let mut source = Box::new(PageTable::new());
    // Two 1-GiB firmware leaves of the same physical gigabyte: identity, then an alias.
    for index in [0, 1] {
        source[index].set_addr(
            PhysAddr::new(0),
            Flags::PRESENT | Flags::WRITABLE | Flags::HUGE_PAGE | Flags::GLOBAL,
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
    // SAFETY: Boxed source and destination tables stay live for the whole test.
    let level3 = unsafe { inherited::copy(physical, 3, &image, &mut frames) }.unwrap();
    let mut root = Box::new(PageTable::new());
    root[0].set_addr(level3.start_address(), Flags::PRESENT | Flags::WRITABLE);
    // SAFETY: Host tables addressed by their own pointers behave as identity mapped.
    let mut mapper = unsafe { OffsetPageTable::new(&mut root, VirtAddr::zero()) };
    unsafe { shared::share_image(&mut mapper, &image) }.unwrap();
    for offset in (0..0x4000).step_by(4096) {
        let (identity, alias) = (0x200000 + offset, (1 << 30) + 0x200000 + offset);
        let (frame, flags, level) = leaf(level3, identity);
        assert_eq!((frame, level), (identity, 1));
        assert!(flags.contains(Flags::GLOBAL));
        assert_eq!(flags.contains(Flags::NO_EXECUTE), offset != 0x1000);
        let (frame, flags, level) = leaf(level3, alias);
        assert_eq!((frame, level), (identity, 1));
        assert!(!flags.contains(Flags::GLOBAL));
    }
    // The split neighbour page and the untouched 2-MiB leaf beside it stay per root.
    let (_, flags, level) = leaf(level3, 0x204000);
    assert_eq!(level, 1);
    assert!(!flags.contains(Flags::GLOBAL));
    let (_, flags, level) = leaf(level3, 0x400000);
    assert_eq!(level, 2);
    assert!(!flags.contains(Flags::GLOBAL));
}
fn leaf(root: PhysFrame, address: u64) -> (u64, Flags, u8) {
    let mut table = root.start_address().as_u64();
    for level in (1..=3).rev() {
        // SAFETY: The test holds every table until traversal finishes.
        let entry = unsafe {
            &(&*(table as *const PageTable))[((address >> (12 + 9 * (level - 1))) & 511) as usize]
        };
        let flags = entry.flags();
        assert!(flags.contains(Flags::PRESENT));
        if level == 1 || flags.contains(Flags::HUGE_PAGE) {
            return (entry.addr().as_u64(), flags, level);
        }
        table = entry.addr().as_u64();
    }
    unreachable!()
}
