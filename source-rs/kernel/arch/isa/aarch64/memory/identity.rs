//! Board apertures are NX; only kernel image code becomes executable. Image
//! blocks descend to pages so code, immutable data and mutable state never mix.
use super::{MemoryError, PAGE, tables};
use crate::memory::KernelImage;
pub(super) unsafe fn prepare(
    root: u64,
    image: &KernelImage<'_>,
    next: &mut impl FnMut() -> Result<u64, MemoryError>,
) -> Result<(), MemoryError> {
    let windows = &crate::board::virt::IDENTITY_MAP;
    if !windows.iter().any(|w| {
        !w.device && image.range.base >= w.start && image.range.base + image.range.bytes <= w.end
    }) {
        return Err(MemoryError::InvalidImage);
    }
    for window in windows {
        let flags = if window.device {
            (1 << 10) | (1 << 2) | tables::NX
        } else {
            tables::NORMAL | tables::NX
        };
        for block in (window.start..window.end).step_by(1 << 21) {
            // SAFETY: Disjoint board windows, inactive owned tree and fresh table source.
            unsafe {
                if image.overlaps(block, 1 << 21) {
                    for address in (block..block + (1 << 21)).step_by(PAGE as usize) {
                        let mut flags = flags;
                        if let Some(p) = image.permissions(address) {
                            if !p.writable {
                                flags |= tables::RO;
                            }
                            if p.executable {
                                flags &= !(1 << 53);
                            }
                        }
                        tables::map(root, address, address, flags, next)?;
                    }
                } else {
                    tables::identity_blocks(root, block, block + (1 << 21), flags, next)?;
                }
            }
        }
    }
    Ok(())
}
