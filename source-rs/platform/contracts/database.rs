//! Experimental private-object IPC; provider: services/storage/database/main.rs.
//! One object in the connection's granted root. No global names, digest lookup,
//! root selector or caller PID. Requests are synchronous; read ranges pin a revision.
//! A failed/lost reply may hide a commit: inspect HEAD before retrying publication.
pub const HEAD: u64 = 1;
pub const READ: u64 = 2;
pub const PUBLISH: u64 = 3;
pub const CLOSE: u64 = 4; // Supervisor control connection only.
pub const BULK: u64 = 5; // expected revision, offset, length; sealed response.
pub const PIN: u64 = 6; // revision, slot (0..2), zero length.
pub const RELEASE: u64 = 7; // revision, slot, zero length; stale release refused.
pub const COLLECT: u64 = 8; // zero args; reply: removed, charged, more (u64 each).
pub const BULK_PIN: u64 = 9; // revision, pin slot, length; reads from offset zero.
pub const OK: u64 = 0;
pub const INVALID: u64 = 1;
pub const TOO_LARGE: u64 = 2;
pub const CONFLICT: u64 = 3;
pub const CAPACITY: u64 = 4;
pub const BUSY: u64 = 5; // transient buffer/reader funding refusal.
pub const MAX_BYTES: usize = 16384;
pub const READ_BYTES: usize = 40;

/// HEAD/CLOSE have zero arguments and no attachment. READ has expected revision,
/// offset, length<=40. PUBLISH has expected revision, zero offset, length<=16384
/// and exactly one frozen memory attachment (even for an empty publication).
/// BULK/BULK_PIN require a nonzero expected revision and 1..=MAX_BYTES length.
/// PIN/RELEASE name one of two durable root-owned slots; these are experimental
/// retention records, not process-local handles or a watcher replay guarantee.
#[derive(Clone, Copy)]
pub struct Request {
    pub op: u64,
    pub expected: u64,
    pub offset: u64,
    pub len: u64,
}
impl Request {
    pub fn encode(self) -> [u8; 32] {
        let mut bytes = [0; 32];
        for (slot, value) in
            bytes
                .chunks_exact_mut(8)
                .zip([self.op, self.expected, self.offset, self.len])
        {
            slot.copy_from_slice(&value.to_le_bytes());
        }
        bytes
    }
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 32 {
            return None;
        }
        Some(Self {
            op: word(bytes, 0),
            expected: word(bytes, 8),
            offset: word(bytes, 16),
            len: word(bytes, 24),
        })
    }
}
/// Response: status:u64, revision:u64, total_length:u64, then either manifest
/// digest (HEAD/PUBLISH) or up to 40 read bytes. Remaining bytes are zero.
/// BULK/BULK_PIN instead put returned length at byte 24 and attach one sealed
/// object rounded to whole pages, with zero padding after that length. Consumers
/// must drop it when done. BUSY attaches nothing; reader and provider-page funding
/// are finite. COLLECT instead returns removed/charged/more at bytes 8/16/24.
pub fn word(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}
