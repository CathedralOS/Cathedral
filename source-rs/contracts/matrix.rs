//! Experimental local-world protocol. Children can address only their own binding.
//! No ancestor coordinates, global IDs, sibling enumeration or depth on the wire.
pub type Packet = [u64; 8];
pub const VIEW: u64 = 1;
pub const NEXT: u64 = 2;
pub const NONE: u64 = 0;
pub const POINTER: u64 = 1;
pub const KEY: u64 = 2;
pub const CANCEL: u64 = 3;
pub const KEY_MASK: u64 = 0x1f;
// Owner connection only. Child requests using these opcodes are uniformly denied.
pub const BIND: u64 = 100; // width,height,key_mask; physical/root adapter only
pub const PLACE: u64 = 101; // local child slot,x,y,width,height,key_mask
pub const REVOKE: u64 = 102; // local child slot
pub const POINTER_SOURCE: u64 = 103; // x,y,buttons; physical/root adapter only
pub const KEY_SOURCE: u64 = 104; // key,state; physical/root adapter only
pub const HEALTH: u64 = 105;
pub const MAX_EXTENT: u64 = 4096;
pub fn encode(words: Packet) -> [u8; 64] {
    let mut bytes = [0; 64];
    for (word, chunk) in words.iter().zip(bytes.chunks_exact_mut(8)) {
        chunk.copy_from_slice(&word.to_le_bytes());
    }
    bytes
}
pub fn decode(bytes: &[u8]) -> Option<Packet> {
    if bytes.len() != 64 {
        return None;
    }
    let mut words = [0; 8];
    for (word, chunk) in words.iter_mut().zip(bytes.chunks_exact(8)) {
        *word = u64::from_le_bytes(chunk.try_into().ok()?);
    }
    Some(words)
}
pub fn error(status: u64) -> Packet {
    [status, 0, 0, 0, 0, 0, 0, 0]
}
