//! Experimental envelope encoding. Two complete checksummed copies; no native
//! struct serialization. Engine bytes and their own recovery metadata are opaque.
use std::io;
pub const BLOCK: u64 = 4096;
pub const DATA_START: u64 = 2 * BLOCK;
const MAGIC: &[u8; 16] = b"CATH-VOLUME-LAB\0";
const HASH_START: usize = BLOCK as usize - 32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub id: [u8; 16],
    pub generation: u64,
    pub region_bytes: u64,
    pub length: u64,
    pub ready: bool,
}
pub fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
pub fn geometry(bytes: u64) -> io::Result<()> {
    if bytes < DATA_START + 2 * 1024 * 1024 || !bytes.is_multiple_of(BLOCK) {
        return Err(invalid(
            "volume must be block-aligned and provide at least 2 MiB engine capacity",
        ));
    }
    Ok(())
}
fn put(out: &mut [u8], offset: usize, value: u64) {
    out[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}
fn get(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

impl Header {
    pub fn capacity(&self) -> u64 {
        self.region_bytes - DATA_START
    }
    pub fn encode(&self) -> [u8; BLOCK as usize] {
        let mut bytes = [0; BLOCK as usize];
        bytes[..16].copy_from_slice(MAGIC);
        put(&mut bytes, 16, 1); // Envelope version, independent of redb format.
        put(&mut bytes, 24, if self.ready { 2 } else { 1 });
        bytes[32..48].copy_from_slice(&self.id);
        put(&mut bytes, 48, self.generation);
        put(&mut bytes, 56, self.region_bytes);
        put(&mut bytes, 64, DATA_START);
        put(&mut bytes, 72, self.capacity());
        put(&mut bytes, 80, self.length);
        put(&mut bytes, 88, 1); // Experimental engine discriminator: redb.
        seal(&mut bytes);
        bytes
    }
    // None means blank/torn/damaged copy. An intact unsupported format is an
    // error even if another copy looks old and supported: no silent downgrade.
    pub fn decode(bytes: &[u8; BLOCK as usize]) -> io::Result<Option<Self>> {
        let mut expected = *bytes;
        seal(&mut expected);
        if expected[HASH_START..] != bytes[HASH_START..] {
            return Ok(None);
        }
        if &bytes[..16] != MAGIC {
            return Err(invalid("unknown volume magic"));
        }
        if get(bytes, 16) != 1 || get(bytes, 88) != 1 {
            return Err(invalid("unsupported envelope version or engine"));
        }
        if bytes[96..HASH_START].iter().any(|b| *b != 0) {
            return Err(invalid("unknown envelope fields"));
        }
        let region_bytes = get(bytes, 56);
        geometry(region_bytes)?;
        if get(bytes, 64) != DATA_START || get(bytes, 72) != region_bytes - DATA_START {
            return Err(invalid("invalid data-region geometry"));
        }
        let state = get(bytes, 24);
        if state != 1 && state != 2 {
            return Err(invalid("unknown formatting state"));
        }
        let header = Self {
            id: bytes[32..48].try_into().unwrap(),
            generation: get(bytes, 48),
            region_bytes,
            length: get(bytes, 80),
            ready: state == 2,
        };
        if header.id == [0; 16]
            || header.length > header.capacity()
            || header.ready && header.length == 0
        {
            return Err(invalid("invalid volume identity or logical length"));
        }
        Ok(Some(header))
    }
}
pub(super) fn seal(bytes: &mut [u8; BLOCK as usize]) {
    let mut hash = blake3::Hasher::new();
    hash.update(b"cathedral-volume-lab-envelope-v1\0");
    hash.update(&bytes[..HASH_START]);
    bytes[HASH_START..].copy_from_slice(hash.finalize().as_bytes());
}

pub fn select(copies: &[Option<Header>; 2]) -> io::Result<Header> {
    match (&copies[0], &copies[1]) {
        (None, None) => Err(invalid("no valid volume header")),
        (Some(h), None) | (None, Some(h)) => Ok(h.clone()),
        (Some(a), Some(b)) => {
            if a.id != b.id
                || a.region_bytes != b.region_bytes
                || (a.generation == b.generation && a != b)
            {
                return Err(invalid("conflicting volume headers"));
            }
            Ok(if a.generation >= b.generation { a } else { b }.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn codec_checks_versions_bounds_checksum_and_identity() {
        let h = Header {
            id: [1; 16],
            generation: 1,
            region_bytes: 4 * 1024 * 1024,
            length: 0,
            ready: false,
        };
        assert_eq!(Header::decode(&h.encode()).unwrap(), Some(h.clone()));
        let mut damaged = h.encode();
        damaged[100] ^= 1;
        assert_eq!(Header::decode(&damaged).unwrap(), None);
        let mut future = h.encode();
        put(&mut future, 16, 2);
        seal(&mut future);
        assert!(Header::decode(&future).is_err());
        let mut bad = h.clone();
        bad.length = u64::MAX;
        assert!(Header::decode(&bad.encode()).is_err());
        bad = h.clone();
        bad.id = [2; 16];
        assert!(select(&[Some(h), Some(bad)]).is_err());
    }
}
