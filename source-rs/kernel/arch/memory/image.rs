//! Physical image permissions, independent of executable format and translation encoding.
use super::{ImageRange, MemoryError, PAGE, Permissions};

#[derive(Clone, Copy)]
pub struct ImageSection {
    pub range: ImageRange,
    pub permissions: Permissions,
}

pub struct KernelImage<'a> {
    pub range: ImageRange,
    sections: &'a [ImageSection],
}
impl<'a> KernelImage<'a> {
    /// Require page-separated sections and reject writable executable storage.
    pub fn new(range: ImageRange, sections: &'a [ImageSection]) -> Result<Self, MemoryError> {
        let valid = |r: ImageRange| {
            r.base != 0
                && r.bytes != 0
                && r.base.is_multiple_of(PAGE)
                && r.bytes.is_multiple_of(PAGE)
                && r.base.checked_add(r.bytes).is_some()
        };
        if !valid(range) || sections.is_empty() {
            return Err(MemoryError::InvalidImage);
        }
        let mut end = range.base;
        for section in sections {
            let r = section.range;
            if !valid(r)
                || r.base < end
                || r.base + r.bytes > range.base + range.bytes
                || (section.permissions.writable && section.permissions.executable)
            {
                return Err(MemoryError::InvalidImage);
            }
            end = r.base + r.bytes;
        }
        Ok(Self { range, sections })
    }

    pub fn overlaps(&self, base: u64, bytes: u64) -> bool {
        base < self.range.base + self.range.bytes && self.range.base < base.saturating_add(bytes)
    }

    /// Headers and inter-section padding are RO/NX. Outside the image is not code.
    pub fn permissions(&self, physical: u64) -> Option<Permissions> {
        if !self.overlaps(physical, 1) {
            return None;
        }
        Some(
            self.sections
                .iter()
                .find(|s| physical >= s.range.base && physical < s.range.base + s.range.bytes)
                .map_or(
                    Permissions {
                        writable: false,
                        executable: false,
                    },
                    |s| s.permissions,
                ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const IMAGE: ImageRange = ImageRange {
        base: 0x1000,
        bytes: 0x6000,
    };
    fn section(base: u64, writable: bool, executable: bool) -> ImageSection {
        ImageSection {
            range: ImageRange { base, bytes: PAGE },
            permissions: Permissions {
                writable,
                executable,
            },
        }
    }
    #[test]
    fn rejects_wx_overlaps_misalignment_and_overflow() {
        assert!(KernelImage::new(IMAGE, &[section(0x2000, true, true)]).is_err());
        assert!(
            KernelImage::new(
                IMAGE,
                &[section(0x2000, false, true), section(0x2000, true, false)]
            )
            .is_err()
        );
        assert!(KernelImage::new(IMAGE, &[section(0x2001, false, true)]).is_err());
        assert!(KernelImage::new(IMAGE, &[section(0x7000, false, true)]).is_err());
        assert!(
            KernelImage::new(
                ImageRange {
                    base: u64::MAX - 4095,
                    bytes: PAGE
                },
                &[]
            )
            .is_err()
        );
    }
    #[test]
    fn headers_padding_and_sections_have_distinct_permissions() {
        let sections = [section(0x2000, false, true), section(0x4000, true, false)];
        let image = KernelImage::new(IMAGE, &sections).unwrap();
        for address in [0x1000, 0x3000, 0x5000, 0x6fff] {
            let p = image.permissions(address).unwrap();
            assert!(!p.writable && !p.executable);
        }
        assert!(image.permissions(0x2000).unwrap().executable);
        assert!(image.permissions(0x4000).unwrap().writable);
        assert!(image.permissions(0x7000).is_none());
        assert!(!image.overlaps(0, 0x1000));
        assert!(image.overlaps(0, 1 << 21));
    }
}
