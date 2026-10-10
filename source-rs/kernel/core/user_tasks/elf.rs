//! Bounded ELF64 ET_EXEC preflight, entirely before physical allocation.
//! ELF shapes follow https://gabi.xinuos.com/elf/08-pheader.html. The accepted
//! subset is lab policy: x86-64, static, page-disjoint readable PT_LOAD segments,
//! no W+X, no interpreter/dynamic/TLS headers, and a file-backed executable entry.

#![forbid(unsafe_code)]

use cathedral_arch::{UserSegment, valid_segments};

const MAX_FILE: usize = 4 * 1024 * 1024;
const EMPTY: UserSegment<'static> = UserSegment {
    address: 0,
    bytes: &[],
    memory_size: 0,
    writable: false,
    executable: false,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Header,
    Bounds,
    Unsupported,
    Segments,
    Entry,
}

#[derive(Debug)]
pub struct Image<'a> {
    pub entry: u64,
    segments: [UserSegment<'a>; 8],
    count: usize,
}
impl<'a> Image<'a> {
    pub fn segments(&self) -> &[UserSegment<'a>] {
        &self.segments[..self.count]
    }
}

pub fn parse(bytes: &[u8]) -> Result<Image<'_>, Error> {
    if bytes.len() < 64 || bytes.len() > MAX_FILE {
        return Err(Error::Bounds);
    }
    if &bytes[..7] != b"\x7fELF\x02\x01\x01"
        || bytes[7..16].iter().any(|byte| *byte != 0)
        || u16_at(bytes, 16) != 2
        || u16_at(bytes, 18) != 62
        || u32_at(bytes, 20) != 1
        || u32_at(bytes, 48) != 0
        || u16_at(bytes, 52) != 64
        || u16_at(bytes, 54) != 56
    {
        return Err(Error::Header);
    }
    let count = usize::from(u16_at(bytes, 56));
    let offset = u64_at(bytes, 32);
    if count == 0 || count > 16 || offset < 64 {
        return Err(Error::Header);
    }
    let headers = range(bytes, offset, (count * 56) as u64)?;
    let mut image = Image {
        entry: u64_at(bytes, 24),
        segments: [EMPTY; 8],
        count: 0,
    };
    let mut stack = false;
    for header in headers.chunks_exact(56) {
        let kind = u32_at(header, 0);
        let flags = u32_at(header, 4);
        let offset = u64_at(header, 8);
        let address = u64_at(header, 16);
        let file_size = u64_at(header, 32);
        let memory_size = u64_at(header, 40);
        let align = u64_at(header, 48);
        match kind {
            0 => continue, // PT_NULL has no loading semantics.
            0x6474_e551 if !stack && flags == 6 && file_size == 0 && memory_size == 0 => {
                stack = true;
                continue;
            }
            1 => {}
            _ => return Err(Error::Unsupported),
        }
        if image.count == image.segments.len()
            || flags & !7 != 0
            || flags & 4 == 0
            || flags & 3 == 3
            || align != 4096
            || !offset.is_multiple_of(4096)
            || !address.is_multiple_of(4096)
            || file_size > memory_size
        {
            return Err(Error::Segments);
        }
        image.segments[image.count] = UserSegment {
            address,
            bytes: range(bytes, offset, file_size)?,
            memory_size,
            writable: flags & 2 != 0,
            executable: flags & 1 != 0,
        };
        image.count += 1;
    }
    if !valid_segments(image.segments()) {
        return Err(Error::Segments);
    }
    if !image.segments().iter().any(|segment| {
        segment.executable
            && image.entry >= segment.address
            && image.entry - segment.address < segment.bytes.len() as u64
    }) {
        return Err(Error::Entry);
    }
    Ok(image)
}

fn range(bytes: &[u8], offset: u64, size: u64) -> Result<&[u8], Error> {
    let end = offset.checked_add(size).ok_or(Error::Bounds)?;
    let start = usize::try_from(offset).map_err(|_| Error::Bounds)?;
    let end = usize::try_from(end).map_err(|_| Error::Bounds)?;
    bytes.get(start..end).ok_or(Error::Bounds)
}
// Only called on a previously bounded 64-byte header or 56-byte program header.
fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap())
}
fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}
fn u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

#[cfg(test)]
mod tests;
