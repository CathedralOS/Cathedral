//! Root-prefixed record keys and their fixed encodings. Kinds: `h` object head,
//! `e` per-object revision event, `t` tree edge from parent to child, `m`
//! manifest, `c` chunk, `p` pin, `q` quota ledger, `n` next object number.
//! Object numbers are never reused; parent zero is the root itself.
use super::*;
use cathedral_content_layout::Digest;
use redb::{ReadableTable, Table};
pub(crate) fn key(root: u8, kind: u8, suffix: &[u8]) -> alloc::vec::Vec<u8> {
    let mut key = alloc::vec![root, kind];
    key.extend_from_slice(suffix);
    key
}
pub(crate) fn head_key(root: u8, id: u64) -> alloc::vec::Vec<u8> {
    key(root, b'h', &id.to_be_bytes())
}
pub(crate) fn event_key(root: u8, id: u64, revision: u64) -> alloc::vec::Vec<u8> {
    let mut suffix = [0; 16];
    suffix[..8].copy_from_slice(&id.to_be_bytes());
    suffix[8..].copy_from_slice(&revision.to_be_bytes());
    key(root, b'e', &suffix)
}
pub(crate) fn edge_key(root: u8, parent: u64, child: u64) -> alloc::vec::Vec<u8> {
    let mut suffix = [0; 16];
    suffix[..8].copy_from_slice(&parent.to_be_bytes());
    suffix[8..].copy_from_slice(&child.to_be_bytes());
    key(root, b't', &suffix)
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Head {
    pub revision: u64,
    pub manifest: Digest,
    pub len: usize,
    pub parent: u64,
}
impl Head {
    pub const SIZE: usize = 56;
    pub fn encode(self) -> [u8; Self::SIZE] {
        let mut bytes = [0; Self::SIZE];
        bytes[..8].copy_from_slice(&self.revision.to_le_bytes());
        bytes[8..40].copy_from_slice(&self.manifest);
        bytes[40..48].copy_from_slice(&(self.len as u64).to_le_bytes());
        bytes[48..].copy_from_slice(&self.parent.to_le_bytes());
        bytes
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != Self::SIZE {
            return Err(Error::Corrupt);
        }
        let len = u64::from_le_bytes(bytes[40..48].try_into().unwrap());
        let revision = u64::from_le_bytes(bytes[..8].try_into().unwrap());
        if len > MAX_BYTES as u64 || revision == 0 {
            return Err(Error::Corrupt);
        }
        Ok(Self {
            revision,
            manifest: bytes[8..40].try_into().unwrap(),
            len: len as usize,
            parent: u64::from_le_bytes(bytes[48..].try_into().unwrap()),
        })
    }
}
/// A numbered object and its current head, as returned by every operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Object {
    pub id: u64,
    pub head: Head,
}
/// A pin names one object revision; its value is the object number followed by
/// the pinned head, so a later delete or edit cannot change what was retained.
pub(crate) fn encode_pin(object: Object) -> [u8; 8 + Head::SIZE] {
    let mut bytes = [0; 8 + Head::SIZE];
    bytes[..8].copy_from_slice(&object.id.to_be_bytes());
    bytes[8..].copy_from_slice(&object.head.encode());
    bytes
}
pub(crate) fn decode_pin(bytes: &[u8]) -> Result<Object> {
    if bytes.len() != 8 + Head::SIZE {
        return Err(Error::Corrupt);
    }
    Ok(Object {
        id: u64::from_be_bytes(bytes[..8].try_into().unwrap()),
        head: Head::decode(&bytes[8..])?,
    })
}
pub(crate) fn quota(
    table: &impl ReadableTable<&'static [u8], &'static [u8]>,
    root: u8,
) -> Result<u64> {
    match engine(table.get(key(root, b'q', &[]).as_slice()))? {
        None => Ok(0),
        Some(value) if value.value().len() == 8 => {
            Ok(u64::from_le_bytes(value.value().try_into().unwrap()))
        }
        _ => Err(Error::Corrupt),
    }
}
pub(crate) fn set_quota(table: &mut Table<&[u8], &[u8]>, root: u8, value: u64) -> Result<()> {
    engine(table.insert(
        key(root, b'q', &[]).as_slice(),
        value.to_le_bytes().as_slice(),
    ))?;
    Ok(())
}
pub(crate) fn charge(charged: &mut u64, bytes: usize) -> Result<()> {
    *charged = charged.checked_add(bytes as u64).ok_or(Error::Capacity)?;
    if *charged > ROOT_QUOTA {
        return Err(Error::Capacity);
    }
    Ok(())
}
/// Insert a record only if absent; an existing record must be byte-identical.
/// Identical content is therefore stored once per root, whichever object owns it.
pub(crate) fn retain(
    table: &mut Table<&[u8], &[u8]>,
    key: &[u8],
    bytes: &[u8],
    charged: &mut u64,
) -> Result<()> {
    if let Some(existing) = engine(table.get(key))? {
        return if existing.value() == bytes {
            Ok(())
        } else {
            Err(Error::Corrupt)
        };
    }
    charge(charged, key.len() + bytes.len())?;
    engine(table.insert(key, bytes))?;
    Ok(())
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
