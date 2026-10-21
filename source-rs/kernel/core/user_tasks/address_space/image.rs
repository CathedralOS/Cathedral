//! Image descriptions and pure geometry/permission validation before allocating frames.
use super::{MAX_IMAGE_PAGES, PAGE, USER_CODE, USER_IMAGE_END};

/// One loadable range: file bytes at `file_offset`, zero-filled to `memory_size`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UserSegment {
    pub address: u64,
    pub file_offset: u64,
    pub file_size: u64,
    pub memory_size: u64,
    pub writable: bool,
    pub executable: bool,
}

/// Where an image's file bytes live: contiguous kernel-readable bytes, or one
/// identity-mapped frame per file page in file order. Segments are page-aligned
/// in the file, so file page k is frame k and loading never needs a contiguous copy.
#[derive(Clone, Copy, Debug)]
pub enum Contents<'a> {
    Bytes(&'a [u8]),
    Frames(&'a [u64]),
}
impl Contents<'_> {
    pub fn len(&self) -> usize {
        match self {
            Self::Bytes(bytes) => bytes.len(),
            Self::Frames(frames) => frames.len() * PAGE as usize,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Initialize a whole destination page: copy its file prefix, then zero
    /// only the remaining tail. Full file pages are never zeroed and overwritten.
    /// # Safety
    /// `to` is exclusive writable RAM for PAGE bytes, disjoint from the source;
    /// len <= PAGE. For a nonempty prefix, every source frame is live identity RAM
    /// and the range lies within the contents and one aligned file page. With
    /// len == 0, file_offset need not refer to contents (BSS and stack pages).
    pub unsafe fn populate_page(&self, file_offset: u64, to: *mut u8, len: usize) {
        debug_assert!(file_offset.is_multiple_of(PAGE) && len <= PAGE as usize);
        // SAFETY: Caller retains a full fresh page and the validated immutable
        // prefix. These two disjoint writes initialize every byte before mapping.
        unsafe {
            if len != 0 {
                let from = match self {
                    Self::Bytes(bytes) => bytes[file_offset as usize..][..len].as_ptr(),
                    Self::Frames(frames) => frames[(file_offset / PAGE) as usize] as *const u8,
                };
                core::ptr::copy_nonoverlapping(from, to, len);
            }
            core::ptr::write_bytes(to.add(len), 0, PAGE as usize - len);
        }
    }
}

/// Geometry/permission check, independent of live paging and physical allocation.
pub fn valid_segments(segments: &[UserSegment]) -> bool {
    if segments.is_empty() || segments.len() > 8 {
        return false;
    }
    let mut pages = 0;
    for (index, segment) in segments.iter().enumerate() {
        if segment.memory_size == 0
            || segment.memory_size > USER_IMAGE_END - USER_CODE
            || segment.file_size > segment.memory_size
            || !segment.file_offset.is_multiple_of(PAGE)
            || segment.address < USER_CODE
            || !segment.address.is_multiple_of(PAGE)
            || (segment.writable && segment.executable)
        {
            return false;
        }
        let Some(end) = segment
            .address
            .checked_add(segment.memory_size.div_ceil(PAGE) * PAGE)
        else {
            return false;
        };
        if end > USER_IMAGE_END {
            return false;
        }
        pages += segment.memory_size.div_ceil(PAGE) as usize;
        if pages > MAX_IMAGE_PAGES {
            return false;
        }
        for previous in &segments[..index] {
            let previous_end = previous.address + previous.memory_size.div_ceil(PAGE) * PAGE;
            if segment.address < previous_end && previous.address < end {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;
    #[test]
    fn frames_copy_one_file_page_each_and_bytes_copy_from_their_offset() {
        let file: Vec<u8> = (0..8193u32).map(|i| (i % 251) as u8).collect();
        let pages: Vec<Vec<u8>> = file
            .chunks(PAGE as usize)
            .map(|chunk| {
                let mut page = chunk.to_vec();
                page.resize(PAGE as usize, 0);
                page
            })
            .collect();
        let frames: Vec<u64> = pages.iter().map(|page| page.as_ptr() as u64).collect();
        let framed = Contents::Frames(&frames);
        let contiguous = Contents::Bytes(&file);
        assert_eq!(framed.len(), 3 * PAGE as usize);
        assert_eq!(contiguous.len(), 8193);
        let mut out = [0u8; 4096];
        // SAFETY: Host pages stand in for identity-mapped frames; copies stay in bounds.
        unsafe {
            framed.populate_page(PAGE, out.as_mut_ptr(), PAGE as usize);
            assert_eq!(out[..], file[4096..8192]);
            framed.populate_page(2 * PAGE, out.as_mut_ptr(), 1);
            assert_eq!(out[0], file[8192]);
            assert!(out[1..].iter().all(|&byte| byte == 0));
            out.fill(0xa5);
            contiguous.populate_page(2 * PAGE, out.as_mut_ptr(), 1);
            assert_eq!(out[0], file[8192]);
            assert!(out[1..].iter().all(|&byte| byte == 0));
        }
    }

    #[test]
    fn every_prefix_length_initializes_poisoned_pages_without_touching_neighbors() {
        let bytes: Vec<u8> = (0..8192).map(|index| (index % 251 + 1) as u8).collect();
        let frames = [bytes.as_ptr() as u64, bytes[4096..].as_ptr() as u64];
        for contents in [Contents::Bytes(&bytes), Contents::Frames(&frames)] {
            for len in 0..=4096 {
                let mut guarded = [0xa5; 4098];
                // SAFETY: Interior page is exclusive; the retained second source
                // page supplies every prefix. Sentinel bytes surround the target.
                unsafe { contents.populate_page(PAGE, guarded.as_mut_ptr().add(1), len) };
                assert_eq!(guarded[0], 0xa5);
                assert_eq!(guarded[4097], 0xa5);
                assert_eq!(&guarded[1..1 + len], &bytes[4096..4096 + len]);
                assert!(guarded[1 + len..4097].iter().all(|&byte| byte == 0));
            }
        }
        for empty in [Contents::Bytes(&[]), Contents::Frames(&[])] {
            let mut page = [0xa5; 4096];
            // SAFETY: Full destination page; a BSS page has no source bytes.
            unsafe { empty.populate_page(256 * PAGE, page.as_mut_ptr(), 0) };
            assert_eq!(page, [0; 4096]);
        }
    }
}
