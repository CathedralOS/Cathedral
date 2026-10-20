//! Executable-format boundary. Decode today's ELF/probes into load ranges;
//! construction owns pages, funding and progress without knowing the file format.
use super::{Error, Executable, Program, elf};
use crate::user_tasks::address_space::{Contents, USER_CODE, USER_DATA, UserSegment};
use cathedral_arch::memory::PAGE;

pub(super) struct Plan {
    pub entry: u64,
    pub segments: alloc::vec::Vec<UserSegment>,
}

pub(super) fn contents<'a>(program: &Program<'a>) -> Contents<'a> {
    match program.executable {
        Executable::Probe(bytes) | Executable::Elf(bytes) => Contents::Bytes(bytes),
        Executable::Object(pages) => Contents::Frames(pages),
    }
}

/// # Safety
/// Object pages are retained immutable, identity-mapped frames for this call.
pub(super) unsafe fn decode<'a>(program: &Program<'a>) -> Result<(Plan, Contents<'a>), Error> {
    let contents = contents(program);
    let (entry, segments) = match program.executable {
        Executable::Probe(code) => {
            if code.is_empty() || code.len() > PAGE as usize {
                return Err(Error::Memory(cathedral_arch::MemoryError::InvalidFrame));
            }
            let segments = [
                UserSegment {
                    address: USER_CODE,
                    file_offset: 0,
                    file_size: code.len() as u64,
                    memory_size: PAGE,
                    writable: false,
                    executable: true,
                },
                UserSegment {
                    address: USER_DATA,
                    file_offset: 0,
                    file_size: 0,
                    memory_size: PAGE,
                    writable: true,
                    executable: false,
                },
            ];
            (USER_CODE, copy(&segments)?)
        }
        Executable::Elf(bytes) => {
            let parsed = elf::parse(bytes).map_err(Error::Executable)?;
            (parsed.entry, copy(parsed.segments())?)
        }
        Executable::Object(pages) => {
            if pages.is_empty() {
                return Err(Error::Executable(elf::Error::Bounds));
            }
            // SAFETY: Caller retains immutable executable custody across decode.
            let first =
                unsafe { core::slice::from_raw_parts(pages[0] as *const u8, PAGE as usize) };
            let parsed = elf::parse_with(first, contents).map_err(Error::Executable)?;
            (parsed.entry, copy(parsed.segments())?)
        }
    };
    Ok((Plan { entry, segments }, contents))
}

fn copy(segments: &[UserSegment]) -> Result<alloc::vec::Vec<UserSegment>, Error> {
    let mut owned = alloc::vec::Vec::new();
    owned
        .try_reserve_exact(segments.len())
        .map_err(|_| Error::OutOfHeap)?;
    owned.extend_from_slice(segments);
    Ok(owned)
}
