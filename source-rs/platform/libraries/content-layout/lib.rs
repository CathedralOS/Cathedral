#![no_std]
#![forbid(unsafe_code)]
//! Experimental layout-lab encoding shared by host experiments and guest storage.
//! Hashes authenticate records; they are never application authority or object IDs.
extern crate alloc;
use alloc::vec::Vec;
pub type Digest = [u8; 32];
pub const CHUNK_DOMAIN: &[u8] = b"cathedral.layout-lab.chunk.v1\0";
pub const MANIFEST_DOMAIN: &[u8] = b"cathedral.layout-lab.manifest.v1\0";

pub fn digest(domain: &[u8], bytes: &[u8]) -> Digest {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(bytes);
    *hasher.finalize().as_bytes()
}

pub enum Contents<'a> {
    Inline(&'a [u8]),
    Chunks(&'a [Digest]),
    Tree { fanout: u32, root: &'a Digest },
}

/// Caller bounds allocation before encoding. Exact host experiment format, not
/// an installed filesystem format commitment. Tree nodes remain a host experiment.
/// Packed bytes: mode:u8, length:u64 LE, chunk size:u32 LE, count:u32 LE,
/// followed by inline bytes (count=0), count digests, or fanout:u32 LE + root digest.
pub fn manifest(len: usize, chunk_bytes: u32, contents: Contents<'_>) -> Vec<u8> {
    assert!(chunk_bytes != 0);
    let mode = match contents {
        Contents::Inline(_) => 0,
        Contents::Chunks(_) => 1,
        Contents::Tree { .. } => 2,
    };
    let count = if mode == 0 {
        0
    } else {
        u32::try_from(len.div_ceil(chunk_bytes as usize)).unwrap()
    };
    let mut bytes = Vec::new();
    bytes.push(mode);
    bytes.extend_from_slice(&(len as u64).to_le_bytes());
    bytes.extend_from_slice(&chunk_bytes.to_le_bytes());
    bytes.extend_from_slice(&count.to_le_bytes());
    match contents {
        Contents::Inline(data) => {
            assert_eq!(data.len(), len);
            bytes.extend_from_slice(data);
        }
        Contents::Chunks(parts) => {
            assert_eq!(parts.len(), count as usize);
            for part in parts {
                bytes.extend_from_slice(part);
            }
        }
        Contents::Tree { fanout, root } => {
            bytes.extend_from_slice(&fanout.to_le_bytes());
            bytes.extend_from_slice(root);
        }
    }
    bytes
}
