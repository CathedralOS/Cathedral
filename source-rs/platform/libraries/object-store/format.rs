//! Two complete catalog snapshots; one sector each plus a separately flushed commit.
use super::catalog::{Catalog, Entry, Root};
pub use super::legacy::crc;
use cathedral_contracts::storage as wire;
fn seal(bytes: &mut [u8; 512]) {
    let checksum = crc(&bytes[..508]);
    bytes[508..].copy_from_slice(&checksum.to_le_bytes());
}
fn valid(bytes: &[u8; 512], magic: &[u8; 8]) -> bool {
    &bytes[..8] == magic && crc(&bytes[..508]).to_le_bytes() == bytes[508..]
}
pub fn encode(catalog: &Catalog, body: &mut [u8; 512], commit: &mut [u8; 512]) {
    *body = [0; 512];
    *commit = [0; 512];
    body[..8].copy_from_slice(b"CTCAT002");
    body[8..16].copy_from_slice(&catalog.sequence.to_le_bytes());
    for (root, value) in catalog.roots.iter().enumerate() {
        let base = 16 + root * 168;
        body[base..base + 8].copy_from_slice(&value.generation.to_le_bytes());
        for (index, entry) in value.entries.iter().enumerate() {
            let offset = base + 8 + index * 40;
            body[offset] = u8::from(entry.present);
            body[offset + 1] = entry.length as u8;
            body[offset + 8..offset + 40].copy_from_slice(&entry.bytes);
        }
    }
    seal(body);
    commit[..8].copy_from_slice(b"CTCOM002");
    commit[8..16].copy_from_slice(&catalog.sequence.to_le_bytes());
    commit[16..20].copy_from_slice(&body[508..]);
    seal(commit);
}
pub fn decode(body: &[u8; 512], commit: &[u8; 512]) -> Option<Catalog> {
    if &body[..8] == b"CTOBJ001" {
        let record = super::legacy::decode(body, commit)?;
        let mut catalog = Catalog {
            sequence: record.generation,
            ..Catalog::default()
        };
        catalog.roots[0].generation = record.generation;
        catalog.roots[0].entries[0] = Entry {
            present: true,
            length: record.length,
            bytes: record.bytes,
        };
        return Some(catalog);
    }
    if !valid(body, b"CTCAT002")
        || !valid(commit, b"CTCOM002")
        || body[8..16] != commit[8..16]
        || body[508..] != commit[16..20]
    {
        return None;
    }
    let mut catalog = Catalog {
        sequence: u64::from_le_bytes(body[8..16].try_into().ok()?),
        ..Catalog::default()
    };
    for root in 0..wire::ROOTS {
        let base = 16 + root * 168;
        let mut value = Root {
            generation: u64::from_le_bytes(body[base..base + 8].try_into().ok()?),
            ..Root::default()
        };
        if value.generation > catalog.sequence {
            return None;
        }
        for (index, entry) in value.entries.iter_mut().enumerate() {
            let offset = base + 8 + index * 40;
            if body[offset] > 1 || body[offset + 1] > 32 {
                return None;
            }
            *entry = Entry {
                present: body[offset] == 1,
                length: body[offset + 1] as usize,
                bytes: body[offset + 8..offset + 40].try_into().ok()?,
            };
            if (!entry.present && entry.length != 0)
                || entry.bytes[entry.length..].iter().any(|&b| b != 0)
                || body[offset + 2..offset + 8].iter().any(|&b| b != 0)
            {
                return None;
            }
        }
        catalog.roots[root] = value;
    }
    if body[352..508]
        .iter()
        .chain(commit[20..508].iter())
        .any(|&b| b != 0)
    {
        return None;
    }
    Some(catalog)
}
