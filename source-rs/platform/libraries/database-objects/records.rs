//! Root-prefixed records and fixed head/event encoding. One object per private
//! root in this experiment; revision zero means absent and is never persisted.
use super::*;
use cathedral_content_layout::Digest;
pub(crate) fn key(root: u8, kind: u8, suffix: &[u8]) -> alloc::vec::Vec<u8> {
    let mut key = alloc::vec![root, kind];
    key.extend_from_slice(suffix);
    key
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Head {
    pub revision: u64,
    pub manifest: Digest,
    pub len: usize,
}
impl Head {
    pub fn encode(self) -> [u8; 48] {
        let mut bytes = [0; 48];
        bytes[..8].copy_from_slice(&self.revision.to_le_bytes());
        bytes[8..40].copy_from_slice(&self.manifest);
        bytes[40..].copy_from_slice(&(self.len as u64).to_le_bytes());
        bytes
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 48 {
            return Err(Error::Corrupt);
        }
        let len = u64::from_le_bytes(bytes[40..].try_into().unwrap());
        let revision = u64::from_le_bytes(bytes[..8].try_into().unwrap());
        if len > MAX_BYTES as u64 || revision == 0 {
            return Err(Error::Corrupt);
        }
        Ok(Self {
            revision,
            manifest: bytes[8..40].try_into().unwrap(),
            len: len as usize,
        })
    }
}

pub(crate) enum Contents<'a> {
    Inline(&'a [u8]),
    Chunks(&'a [u8]),
}
/// Validate the bounded manifest before any digest indexing or range arithmetic.
pub(crate) fn manifest(head: Head, bytes: &[u8]) -> Result<Contents<'_>> {
    use cathedral_content_layout::{self as layout, digest};
    if bytes.len() < 17
        || bytes.len() > 17 + MAX_BYTES / CHUNK_BYTES * 32
        || digest(layout::MANIFEST_DOMAIN, bytes) != head.manifest
    {
        return Err(Error::Corrupt);
    }
    if u64::from_le_bytes(bytes[1..9].try_into().unwrap()) != head.len as u64
        || u32::from_le_bytes(bytes[9..13].try_into().unwrap()) as usize != CHUNK_BYTES
    {
        return Err(Error::Corrupt);
    }
    let count = u32::from_le_bytes(bytes[13..17].try_into().unwrap()) as usize;
    match bytes[0] {
        0 if head.len <= INLINE_BYTES && count == 0 && bytes.len() == 17 + head.len => {
            Ok(Contents::Inline(&bytes[17..]))
        }
        1 if head.len > INLINE_BYTES
            && count == head.len.div_ceil(CHUNK_BYTES)
            && bytes.len() == 17 + count * 32 =>
        {
            Ok(Contents::Chunks(&bytes[17..]))
        }
        _ => Err(Error::Corrupt),
    }
}
