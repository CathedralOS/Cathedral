//! Private object storage. The accepted connection selects the root; objects
//! are numbered within it and numbers are never reused. No global names,
//! digests, root selectors or caller identities cross the wire.
//!
//! Every request and reply is one 64-byte packet of eight little-endian words.
//! Request: `[op, object, expected, offset, len, 0, 0, 0]`. A matrix forwards
//! the same words, scoping word zero by the child's binding (matrix.rs).
//! A failed or lost reply may hide a commit: read HEAD before repeating a write.
use cathedral_kernel_contracts::user as abi;
/// `object` → status, object, revision, length.
pub const HEAD: u64 = 1;
/// `object, expected, offset, len <= READ_BYTES` → bytes at 32..64.
pub const READ: u64 = 2;
/// `object or 0 to create, expected, 0, len` plus one frozen attachment.
pub const PUBLISH: u64 = 3;
/// `source object, expected` → a new object sharing the source's content.
pub const COPY: u64 = 4;
/// `object, expected`.
pub const DELETE: u64 = 5;
/// `after` → status, count, then up to LIST_IDS ascending object numbers.
pub const LIST: u64 = 6;
/// `object, expected revision, offset, 1..=MAX_BYTES` → sealed attachment.
pub const BULK: u64 = 7;
/// `object, revision, slot` retains that revision in a root-owned pin slot.
pub const PIN: u64 = 8;
/// `object, revision, slot`; a stale release is refused.
pub const RELEASE: u64 = 9;
/// `object, revision, slot, len` reads a pinned revision from offset zero.
pub const BULK_PIN: u64 = 10;
/// No arguments → status, removed, charged, more.
pub const COLLECT: u64 = 11;
/// Supervisor control connection only.
pub const CLOSE: u64 = 12;
/// Highest operation a delegated child may issue.
pub const LAST: u64 = COLLECT;
pub const MAX_BYTES: usize = 16384;
pub const READ_BYTES: usize = 32;
pub const LIST_IDS: usize = 6;
pub const PIN_SLOTS: u64 = 2;
pub const ROOTS: usize = 2;
pub const HEALTH: &[u8] = b"health";
/// Provider refusals use kernel status codes plus these two.
pub const TOO_LARGE: u64 = (-7i64) as u64;
pub const MISSING: u64 = (-2i64) as u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub op: u64,
    pub object: u64,
    pub expected: u64,
    pub offset: u64,
    pub len: u64,
}
impl Request {
    pub fn words(self) -> [u64; 8] {
        [
            self.op,
            self.object,
            self.expected,
            self.offset,
            self.len,
            0,
            0,
            0,
        ]
    }
    pub fn encode(self) -> [u8; 64] {
        pack(self.words())
    }
    /// Trailing words must be zero: extra fields are malformed, never a
    /// second namespace or root selector.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let words = unpack(bytes)?;
        if words[5..].iter().any(|w| *w != 0) {
            return None;
        }
        Some(Self {
            op: words[0],
            object: words[1],
            expected: words[2],
            offset: words[3],
            len: words[4],
        })
    }
}
/// Reply shape shared by HEAD, READ, PUBLISH, COPY, BULK and BULK_PIN.
/// `extra` carries the attached byte count for bulk reads. READ places its
/// bytes at 32..64 instead.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Reply {
    pub object: u64,
    pub revision: u64,
    pub length: u64,
    pub extra: u64,
}
impl Reply {
    pub fn encode(self) -> [u8; 64] {
        pack([
            0,
            self.object,
            self.revision,
            self.length,
            self.extra,
            0,
            0,
            0,
        ])
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, u64> {
        let words = unpack(bytes).ok_or(abi::IO_ERROR)?;
        if words[0] != 0 {
            return Err(words[0]);
        }
        Ok(Self {
            object: words[1],
            revision: words[2],
            length: words[3],
            extra: words[4],
        })
    }
}
pub fn error(status: u64) -> [u8; 64] {
    pack([status, 0, 0, 0, 0, 0, 0, 0])
}
pub fn word(bytes: &[u8], index: usize) -> u64 {
    u64::from_le_bytes(bytes[index * 8..index * 8 + 8].try_into().unwrap())
}
pub fn pack(words: [u64; 8]) -> [u8; 64] {
    let mut bytes = [0; 64];
    for (word, chunk) in words.iter().zip(bytes.chunks_exact_mut(8)) {
        chunk.copy_from_slice(&word.to_le_bytes());
    }
    bytes
}
pub fn unpack(bytes: &[u8]) -> Option<[u64; 8]> {
    if bytes.len() != 64 {
        return None;
    }
    let mut words = [0; 8];
    for (word, chunk) in words.iter_mut().zip(bytes.chunks_exact(8)) {
        *word = u64::from_le_bytes(chunk.try_into().ok()?);
    }
    Some(words)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn requests_round_trip_and_reject_extra_fields() {
        let request = Request {
            op: PUBLISH,
            object: 0,
            expected: 0,
            offset: 0,
            len: 300,
        };
        let bytes = request.encode();
        assert_eq!(Request::decode(&bytes), Some(request));
        let mut forged = bytes;
        forged[40] = 1;
        assert!(Request::decode(&forged).is_none());
        assert!(Request::decode(&bytes[..63]).is_none());
        let reply = Reply {
            object: 7,
            revision: 3,
            length: 300,
            extra: 0,
        };
        assert_eq!(Reply::decode(&reply.encode()), Ok(reply));
        assert_eq!(Reply::decode(&error(MISSING)), Err(MISSING));
    }
}
