//! Experimental private-object IPC; provider: services/storage/database/main.rs.
//! One object in the connection's granted root. No global names, digest lookup,
//! root selector or caller PID. Requests are synchronous; read ranges pin a revision.
//! A failed/lost reply may hide a commit: inspect HEAD before retrying publication.
pub const HEAD: u64 = 1;
pub const READ: u64 = 2;
pub const PUBLISH: u64 = 3;
pub const CLOSE: u64 = 4; // Supervisor control connection only.
pub const OK: u64 = 0;
pub const INVALID: u64 = 1;
pub const TOO_LARGE: u64 = 2;
pub const CONFLICT: u64 = 3;
pub const CAPACITY: u64 = 4;
pub const MAX_BYTES: usize = 16384;
pub const READ_BYTES: usize = 40;

/// HEAD/CLOSE have zero arguments and no attachment. READ has expected revision,
/// offset, length<=40. PUBLISH has expected revision, zero offset, length<=16384
/// and exactly one frozen memory attachment (even for an empty publication).
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
pub fn word(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}
