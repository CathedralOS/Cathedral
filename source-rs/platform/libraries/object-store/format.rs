use cathedral_contracts::storage::Record;
pub fn crc(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb88320 & 0u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}
fn seal(bytes: &mut [u8; 512]) {
    let checksum = crc(&bytes[..508]);
    bytes[508..].copy_from_slice(&checksum.to_le_bytes());
}
fn valid(bytes: &[u8; 512], magic: &[u8; 8]) -> bool {
    &bytes[..8] == magic && crc(&bytes[..508]).to_le_bytes() == bytes[508..]
}
pub fn encode(record: Record) -> ([u8; 512], [u8; 512]) {
    let mut body = [0; 512];
    let mut commit = [0; 512];
    body[..8].copy_from_slice(b"CTOBJ001");
    body[8..16].copy_from_slice(&record.generation.to_le_bytes());
    body[16..24].copy_from_slice(&(record.length as u64).to_le_bytes());
    body[24..32].copy_from_slice(&1u64.to_le_bytes());
    body[32..64].copy_from_slice(&record.bytes);
    seal(&mut body);
    commit[..8].copy_from_slice(b"CTCOM001");
    commit[8..16].copy_from_slice(&record.generation.to_le_bytes());
    commit[16..20].copy_from_slice(&body[508..]);
    seal(&mut commit);
    (body, commit)
}
pub fn decode(body: &[u8; 512], commit: &[u8; 512]) -> Option<Record> {
    if !valid(body, b"CTOBJ001")
        || !valid(commit, b"CTCOM001")
        || body[8..16] != commit[8..16]
        || body[508..] != commit[16..20]
    {
        return None;
    }
    let generation = u64::from_le_bytes(body[8..16].try_into().ok()?);
    let length = u64::from_le_bytes(body[16..24].try_into().ok()?);
    if (generation == 0 && length != 0) || length > 32 || body[24..32] != 1u64.to_le_bytes() {
        return None;
    }
    Some(Record {
        generation,
        length: length as usize,
        bytes: body[32..64].try_into().ok()?,
    })
}
