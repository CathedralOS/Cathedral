//! Image descriptions and pure geometry/permission validation before allocating frames.
use super::{MAX_IMAGE_PAGES, PAGE, USER_CODE, USER_IMAGE_END};

#[derive(Clone, Copy, Debug)]
pub struct UserSegment<'a> {
    pub address: u64,
    pub bytes: &'a [u8],
    pub memory_size: u64,
    pub writable: bool,
    pub executable: bool,
}

/// Geometry/permission check, independent of live paging and physical allocation.
pub fn valid_segments(segments: &[UserSegment<'_>]) -> bool {
    if segments.is_empty() || segments.len() > 8 {
        return false;
    }
    let mut pages = 0;
    for (index, segment) in segments.iter().enumerate() {
        if segment.memory_size == 0
            || segment.memory_size > USER_IMAGE_END - USER_CODE
            || segment.bytes.len() as u64 > segment.memory_size
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
