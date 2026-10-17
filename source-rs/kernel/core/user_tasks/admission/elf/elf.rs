// Bounded ELF64 ET_EXEC preflight, entirely before physical allocation.
// ELF shapes follow https://gabi.xinuos.com/elf/08-pheader.html. The accepted
// subset is lab policy: selected CPU, static, page-disjoint readable PT_LOAD segments,
// no W+X, no interpreter/dynamic/TLS headers, a file-backed executable entry,
// and the header plus every program header within the first file page, so an
// image held one frame per page needs no contiguous copy to be checked.

use crate::user_tasks::address_space::{Contents, UserSegment, valid_segments};

const MAX_FILE: usize = 4 * 1024 * 1024;
const EMPTY: UserSegment = UserSegment {
    address: 0,
    file_offset: 0,
    file_size: 0,
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
    pub contents: Contents<'a>,
    segments: [UserSegment; 8],
    count: usize,
}
impl Image<'_> {
    pub fn segments(&self) -> &[UserSegment] {
        &self.segments[..self.count]
    }
}

/// Check an image held as contiguous bytes.
pub fn parse(bytes: &[u8]) -> Result<Image<'_>, Error> {
    parse_with(&bytes[..bytes.len().min(4096)], Contents::Bytes(bytes))
}
/// Check an image whose first file page the caller has already read; the
/// contents are only measured here and copied from at load time.
pub fn parse_with<'a>(first_page: &[u8], contents: Contents<'a>) -> Result<Image<'a>, Error> {
    let len = contents.len();
    if !(64..=MAX_FILE).contains(&len) {
        return Err(Error::Bounds);
    }
    let bytes = first_page;
    if !(64..=4096).contains(&bytes.len()) {
        return Err(Error::Header);
    }
    if &bytes[..7] != b"\x7fELF\x02\x01\x01"
        || bytes[7..16].iter().any(|byte| *byte != 0)
        || u16_at(bytes, 16) != 2
        || u16_at(bytes, 18) != cathedral_arch::ELF_MACHINE
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
    let headers = range(bytes, offset, (count * 56) as u64).map_err(|_| Error::Header)?;
    let mut image = Image {
        entry: u64_at(bytes, 24),
        contents,
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
        if offset
            .checked_add(file_size)
            .is_none_or(|end| end > len as u64)
        {
            return Err(Error::Bounds);
        }
        image.segments[image.count] = UserSegment {
            address,
            file_offset: offset,
            file_size,
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
            && image.entry - segment.address < segment.file_size
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
