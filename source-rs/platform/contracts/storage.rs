//! Private object storage. The accepted connection selects the root; objects
//! are numbered within it, arranged as a tree, and numbers are never reused.
//! No global names, digests, root selectors or caller identities cross the wire.
//!
//! COPY and COPY_TREE preserve independent edits in both directions while
//! sharing immutable backing. See wiki/spec/storage/realms_and_objects.md;
//! persisted references use paths in an authorized view. Names and queryable
//! media types belong to hierarchy metadata. LOOKUP resolves relative paths;
//! FILTER uses an exact-type index under a scoped parent. IDs remain transient
//! request handles, not persisted cross-view references.
//!
//! Every request and reply is one 64-byte packet of eight little-endian words.
//! Request: `[op, object, expected, argument, len, scope, scope, deadline]`. The
//! argument word is the byte offset for reads, the parent for creation and
//! copies, the pin slot for pins and the listing cursor. The two scope words
//! are stamped by matrices on the way to the provider (matrix.rs): every
//! nonzero scope names an object, and the request may only touch objects at
//! or below each of them. A client that stamps its own scope only narrows
//! itself. A matrix can fold a full packet only after proving containment of
//! its innermost scope under all other restrictions. Immutable parent links
//! and never-reused IDs make that proof stable. A matrix may retain successful
//! proofs only within the live binding/provider incarnation that checked them;
//! revocation or upstream replacement invalidates them. The last word is an absolute
//! end-to-end deadline; intermediaries never extend it. A failed or lost reply
//! may hide a commit: reconcile HEAD, or LOOKUP for a named creation.
use cathedral_kernel_contracts::user as abi;
/// `object` → status, object, revision, length.
pub const HEAD: u64 = 1;
/// `object, expected, offset, len <= READ_BYTES` → bytes at 32..64.
pub const READ: u64 = 2;
/// `object or 0 to create, expected, parent when creating, len` plus one
/// frozen attachment; with `len` zero the attachment may be omitted and the
/// content is empty, which is how a container object is made.
pub const PUBLISH: u64 = 3;
/// `source object, expected, parent` → a new object sharing the source's content.
/// An optional name attachment names the copy atomically, as with COPY_TREE.
pub const COPY: u64 = 4;
/// `object, expected`; an object that still has children is refused BUSY.
pub const DELETE: u64 = 5;
/// `parent, 0, after` → status, count, then up to LIST_IDS ascending children.
pub const LIST: u64 = 6;
/// `object, expected revision or 0 for the current one, offset, len` → sealed
/// attachment of `len` bytes, at most MAX_BYTES. A `len` of zero reads the
/// whole object from `offset`, so a launcher needs no HEAD first; the reply's
/// revision and length say what was read.
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
/// `source object, expected, parent` → the copy of the source, whose whole
/// subtree is copied beneath it with the same shape, sharing all content.
/// An optional one-page attachment supplies a new root name of `len` bytes;
/// without it the root is unnamed. Descendant names/types are preserved.
pub const COPY_TREE: u64 = 13;
/// `object, 0, field, 0`: field 0 is name, 1 is media type. Reply words 0/1
/// hold status/revision; bytes 32..64 hold the zero-padded field.
pub const METADATA: u64 = 14;
/// `object, expected, 0, 64` plus Metadata attachment; atomic rename/retype.
pub const SET_METADATA: u64 = 15;
/// `base, 0, 0, path length` plus relative-path attachment; no parent traversal.
pub const LOOKUP: u64 = 16;
/// `parent, 0, after, type length` plus type attachment; indexed direct children.
pub const FILTER: u64 = 17;
/// `0, 0, parent, payload length`: one sealed attachment containing 64 bytes
/// of Metadata followed by initial content. Name must be nonempty. Content,
/// entry, indexes and quota commit atomically; collisions return WOULD_BLOCK.
/// A lost reply requires LOOKUP/HEAD reconciliation, never blind replacement.
/// Total payload is at most MAX_BYTES (initial content at most MAX_BYTES - 64).
pub const CREATE_NAMED: u64 = 18;
/// `parent, 0, after, 0` → status, count, last listed number, then one sealed
/// page of `count` `Entry` rows: the direct children after `after` in
/// ascending order with their revisions and metadata, at most
/// ENTRIES_PER_PAGE. A count of zero carries no page. One request names a
/// realm; LOOKUP, FILTER and METADATA remain for single answers.
pub const ENTRIES: u64 = 19;
/// Highest operation a delegated child may issue.
pub const LAST: u64 = ENTRIES;
#[path = "storage_metadata.rs"]
pub mod metadata;
/// One bounded shared object, including a reusable UI application's native code.
/// Larger object geometry does not increase per-task funding or root quota.
pub const MAX_BYTES: usize =
    cathedral_kernel_contracts::memory::MAX_PAGES * cathedral_kernel_contracts::memory::PAGE_BYTES;
pub const READ_BYTES: usize = 32;
pub const LIST_IDS: usize = 6;
/// One row of an ENTRIES page: number and revision, little-endian, then the
/// child's Metadata.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Entry {
    pub object: u64,
    pub revision: u64,
    pub metadata: metadata::Metadata,
}
pub const ENTRY_BYTES: usize = 16 + metadata::BYTES;
pub const ENTRIES_PER_PAGE: usize = 4096 / ENTRY_BYTES;
impl Entry {
    pub fn encode(self) -> [u8; ENTRY_BYTES] {
        let mut bytes = [0; ENTRY_BYTES];
        bytes[..8].copy_from_slice(&self.object.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.revision.to_le_bytes());
        bytes[16..].copy_from_slice(&self.metadata.encode());
        bytes
    }
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != ENTRY_BYTES {
            return None;
        }
        Some(Self {
            object: u64::from_le_bytes(bytes[..8].try_into().ok()?),
            revision: u64::from_le_bytes(bytes[8..16].try_into().ok()?),
            metadata: metadata::Metadata::decode(&bytes[16..])?,
        })
    }
}
pub const PIN_SLOTS: u64 = 2;
pub const SCOPES: usize = 2;
/// End-to-end forwarding bound. Matrices preserve an earlier supplied deadline;
/// only the root adapter applies the shorter provider-operation bound.
pub const EXCHANGE_TICKS: u64 = 2 * crate::matrix::STORAGE_DEADLINE_TICKS;
pub fn deadline(now: u64, requested: u64, budget: u64) -> Option<u64> {
    if budget == 0 || budget >= 1 << 63 {
        return None;
    }
    let remaining = if requested == 0 {
        budget
    } else {
        requested.wrapping_sub(now)
    };
    if remaining == 0 || remaining >= 1 << 63 {
        return None;
    }
    let result = now.wrapping_add(remaining.min(budget));
    // Zero denotes an unspecified deadline on the wire, so expire one tick
    // earlier when the bound would land exactly on that sentinel.
    let result = if result == 0 { u64::MAX } else { result };
    (result != now).then_some(result)
}
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
    pub argument: u64,
    pub len: u64,
    pub scopes: [u64; SCOPES],
    /// Absolute monotonic tick, never extended by forwarding. Zero asks the
    /// first adapter to supply its bounded default (legacy/direct clients).
    pub deadline: u64,
}
impl Request {
    /// A request with no scope of its own; matrices stamp the scope words.
    pub const fn new(op: u64, object: u64, expected: u64, argument: u64, len: u64) -> Self {
        Self {
            op,
            object,
            expected,
            argument,
            len,
            scopes: [0; SCOPES],
            deadline: 0,
        }
    }
    pub fn words(self) -> [u64; 8] {
        [
            self.op,
            self.object,
            self.expected,
            self.argument,
            self.len,
            self.scopes[0],
            self.scopes[1],
            self.deadline,
        ]
    }
    pub fn encode(self) -> [u8; 64] {
        pack(self.words())
    }
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let words = unpack(bytes)?;
        Some(Self {
            op: words[0],
            object: words[1],
            expected: words[2],
            argument: words[3],
            len: words[4],
            scopes: [words[5], words[6]],
            deadline: words[7],
        })
    }
    /// The objects every scope must contain: the subject, and the parent a
    /// creation or copy would place its result under.
    pub fn subjects(self) -> ([u64; 2], usize) {
        match self.op {
            CREATE_NAMED => ([self.argument, 0], 1),
            PUBLISH if self.object == 0 => ([self.argument, 0], 1),
            COPY | COPY_TREE => ([self.object, self.argument], 2),
            COLLECT => ([0; 2], 0),
            _ => ([self.object, 0], 1),
        }
    }
}
/// Reply shape shared by HEAD, READ, PUBLISH, COPY, COPY_TREE, BULK and
/// BULK_PIN. `extra` carries the attached byte count for bulk reads. READ
/// places its bytes at 32..64 instead.
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
/// Append a restriction when the packet has room. A full packet needs a
/// containment proof before restrictions can be folded; never drop one here.
pub fn stamp(words: &mut [u64; 8], scope: u64) -> Result<(), u64> {
    if scope == 0 || words[5..7].contains(&scope) {
        return Ok(());
    }
    let free = words[5..7]
        .iter_mut()
        .find(|s| **s == 0)
        .ok_or(abi::DENIED)?;
    *free = scope;
    Ok(())
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
    fn requests_round_trip_with_scopes_and_name_their_subjects() {
        let request = Request::new(PUBLISH, 0, 0, 7, 300);
        let bytes = request.encode();
        assert_eq!(Request::decode(&bytes), Some(request));
        assert!(Request::decode(&bytes[..63]).is_none());
        assert_eq!(request.subjects(), ([7, 0], 1));
        let mut words = request.words();
        stamp(&mut words, 0).unwrap();
        stamp(&mut words, 4).unwrap();
        stamp(&mut words, 9).unwrap();
        assert_eq!(stamp(&mut words, 2), Err(abi::DENIED));
        assert_eq!(Request::decode(&pack(words)).unwrap().scopes, [4, 9]);
        assert_eq!(deadline(10, 100, 200), Some(100));
        assert_eq!(deadline(10, 1000, 200), Some(210));
        assert_eq!(deadline(10, 10, 200), None);
        assert_eq!(deadline(u64::MAX - 2, 1, 10), Some(1));
        assert_eq!(deadline(u64::MAX - 2, 0, 3), Some(u64::MAX));
        assert_eq!(Request::new(COPY_TREE, 3, 1, 8, 0).subjects(), ([3, 8], 2));
        assert_eq!(Request::new(LIST, 3, 0, 0, 0).subjects(), ([3, 0], 1));
        assert_eq!(Request::new(COLLECT, 0, 0, 0, 0).subjects().1, 0);
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
