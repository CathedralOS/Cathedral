//! Experimental private catalogs. Object numbers are relative to the accepted connection.
//! Generations cover the entire root. Staging is bounded and bound to one connection incarnation.
use crate::user as abi;
pub const READ: u64 = 1;
pub const REPLACE: u64 = 2;
pub const LIST: u64 = 3;
pub const CREATE: u64 = 4;
pub const DELETE: u64 = 5;
pub const BEGIN: u64 = 6;
pub const STAGE_CREATE: u64 = 7;
pub const STAGE_REPLACE: u64 = 8;
pub const STAGE_DELETE: u64 = 9;
pub const COMMIT: u64 = 10;
pub const ABORT: u64 = 11;
pub const ROOTS: usize = 2;
pub const OBJECTS: usize = 4;
pub const CHANGES: usize = 2;
pub const NOT_FOUND: u64 = (-2i64) as u64;
pub const EXISTS: u64 = (-17i64) as u64;
pub const OBJECT: u64 = 1;
pub const HEALTH: &[u8] = b"health";
pub const CAPACITY: usize = 32;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Record {
    pub generation: u64,
    pub length: usize,
    pub bytes: [u8; CAPACITY],
}
impl Record {
    pub fn payload(&self) -> &[u8] {
        &self.bytes[..self.length]
    }
    pub fn reply(self) -> [u8; 64] {
        let mut bytes = error(0);
        bytes[8..16].copy_from_slice(&self.generation.to_le_bytes());
        bytes[16..24].copy_from_slice(&(self.length as u64).to_le_bytes());
        bytes[32..64].copy_from_slice(&self.bytes);
        bytes
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, u64> {
        if bytes.len() != 64 {
            return Err(abi::IO_ERROR);
        }
        let status = word(bytes, 0);
        if status != 0 {
            return Err(status);
        }
        let length = word(bytes, 2);
        if length > CAPACITY as u64 || word(bytes, 3) != 0 {
            return Err(abi::IO_ERROR);
        }
        Ok(Self {
            generation: word(bytes, 1),
            length: length as usize,
            bytes: bytes[32..].try_into().unwrap(),
        })
    }
}
pub fn word(bytes: &[u8], index: usize) -> u64 {
    u64::from_le_bytes(bytes[index * 8..index * 8 + 8].try_into().unwrap())
}
pub fn request(op: u64, object: u64, generation: u64, payload: &[u8]) -> Option<[u8; 64]> {
    if payload.len() > CAPACITY {
        return None;
    }
    let mut bytes = [0; 64];
    for (i, value) in [op, object, generation, payload.len() as u64]
        .into_iter()
        .enumerate()
    {
        bytes[i * 8..i * 8 + 8].copy_from_slice(&value.to_le_bytes());
    }
    bytes[32..32 + payload.len()].copy_from_slice(payload);
    Some(bytes)
}
pub fn error(code: u64) -> [u8; 64] {
    let mut bytes = [0; 64];
    bytes[..8].copy_from_slice(&code.to_le_bytes());
    bytes
}
