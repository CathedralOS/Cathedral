//! Clone topology and cache attributes, imposing supervisor W^X by physical
//! backing. Every alias of an image page gets the same permissions. Split only
//! huge leaves intersecting the image; all other inherited leaves become NX.
use super::{Frames, MemoryError, TABLE_BUDGET};
use crate::memory::KernelImage;
use ::x86_64::{
    PhysAddr,
    structures::paging::{PageTable, PageTableFlags as Flags, PhysFrame},
};

pub(super) unsafe fn copy<F: FnMut() -> Option<u64>>(
    source: PhysFrame,
    level: u8,
    image: &KernelImage<'_>,
    frames: &mut Frames<'_, F>,
) -> Result<PhysFrame, MemoryError> {
    let dest = table(frames)?;
    // SAFETY: Valid firmware table, disjoint fresh destination, identity mapped.
    unsafe {
        let source = &*(source.start_address().as_u64() as *const PageTable);
        let target = &mut *(dest.start_address().as_u64() as *mut PageTable);
        for (old, new) in source.iter().zip(target.iter_mut()) {
            let flags = old.flags();
            if !flags.contains(Flags::PRESENT) {
                continue;
            }
            if level > 1 && !flags.contains(Flags::HUGE_PAGE) {
                let child = copy(
                    PhysFrame::from_start_address(old.addr()).unwrap(),
                    level - 1,
                    image,
                    frames,
                )?;
                new.set_addr(child.start_address(), parents(flags));
            } else {
                // Huge-leaf bit 12 is PAT, not part of its physical address.
                let size = 1u64 << (12 + 9 * (level - 1));
                let physical = old.addr().as_u64() & !(size - 1);
                let pat = level > 1 && old.addr().as_u64() & (1 << 12) != 0;
                install(new, physical, level, flags, pat, image, frames)?;
            }
        }
    }
    Ok(dest)
}
unsafe fn install<F: FnMut() -> Option<u64>>(
    entry: &mut ::x86_64::structures::paging::page_table::PageTableEntry,
    physical: u64,
    level: u8,
    flags: Flags,
    pat: bool,
    image: &KernelImage<'_>,
    frames: &mut Frames<'_, F>,
) -> Result<(), MemoryError> {
    let size = 1u64 << (12 + 9 * (level - 1));
    if level > 1 && image.overlaps(physical, size) {
        let child = table(frames)?;
        // SAFETY: Fresh table, exclusive ownership; entries retain original memory type.
        unsafe {
            let table = &mut *(child.start_address().as_u64() as *mut PageTable);
            for (index, leaf) in table.iter_mut().enumerate() {
                let mut child_flags = flags;
                if level == 2 {
                    child_flags.set(Flags::HUGE_PAGE, pat);
                }
                install(
                    leaf,
                    physical + index as u64 * (size / 512),
                    level - 1,
                    child_flags,
                    pat,
                    image,
                    frames,
                )?;
            }
        }
        entry.set_addr(child.start_address(), parents(flags) & !Flags::HUGE_PAGE);
    } else {
        let mut flags = flags & !(Flags::USER_ACCESSIBLE | Flags::GLOBAL);
        flags.insert(Flags::NO_EXECUTE);
        if let Some(permissions) = image.permissions(physical) {
            flags.set(Flags::WRITABLE, permissions.writable);
            flags.set(Flags::NO_EXECUTE, !permissions.executable);
        }
        let address = physical | if level > 1 && pat { 1 << 12 } else { 0 };
        entry.set_addr(PhysAddr::new(address), flags);
    }
    Ok(())
}
fn parents(flags: Flags) -> Flags {
    (flags & !(Flags::NO_EXECUTE | Flags::USER_ACCESSIBLE | Flags::GLOBAL)) | Flags::WRITABLE
}
fn table<F: FnMut() -> Option<u64>>(frames: &mut Frames<'_, F>) -> Result<PhysFrame, MemoryError> {
    if frames.tables >= TABLE_BUDGET {
        return Err(MemoryError::TableBudget);
    }
    let frame = frames.frame()?;
    frames.tables += 1;
    // SAFETY: Frame source promises unique writable identity RAM.
    unsafe {
        (frame.start_address().as_u64() as *mut PageTable).write(PageTable::new());
    }
    Ok(frame)
}
