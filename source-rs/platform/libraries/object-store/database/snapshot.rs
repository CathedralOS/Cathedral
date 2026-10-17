//! Private experimental catalog payload. Not the legacy two-sector disk format.
use cathedral_database_objects::Error;
use cathedral_object_store::catalog::Root;
pub const SIZE: usize = 152;
pub fn encode(root: &Root) -> [u8; SIZE] {
    let mut bytes = [0; SIZE];
    bytes[..8].copy_from_slice(b"CTDBCAT1");
    bytes[8..16].copy_from_slice(&root.generation.to_le_bytes());
    for (chunk, entry) in bytes[16..].chunks_exact_mut(34).zip(&root.entries) {
        chunk[0] = u8::from(entry.present);
        chunk[1] = entry.length as u8;
        chunk[2..].copy_from_slice(&entry.bytes);
    }
    bytes
}
pub fn decode(bytes: &[u8; SIZE]) -> Result<Root, Error> {
    if &bytes[..8] != b"CTDBCAT1" {
        return Err(Error::Corrupt);
    }
    let mut root = Root {
        generation: u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
        ..Root::default()
    };
    if root.generation == 0 {
        return Err(Error::Corrupt);
    }
    for (entry, chunk) in root.entries.iter_mut().zip(bytes[16..].chunks_exact(34)) {
        if chunk[0] > 1 || chunk[1] > 32 {
            return Err(Error::Corrupt);
        }
        entry.present = chunk[0] == 1;
        entry.length = chunk[1] as usize;
        entry.bytes.copy_from_slice(&chunk[2..]);
        if (!entry.present && entry.length != 0)
            || entry.bytes[entry.length..].iter().any(|b| *b != 0)
        {
            return Err(Error::Corrupt);
        }
    }
    Ok(root)
}
