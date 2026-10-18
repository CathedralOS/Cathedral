//! When an oracle disagrees with an image, say which records differ. Keys are
//! printed as hex with their kind byte, values by length, so a failing proof
//! names the record rather than only the fact of a mismatch.
use super::*;
use std::collections::BTreeMap;

/// Cold-compare every table/record and the assigned region's identity/guards.
/// Engine open/recovery may change bookkeeping bytes without a logical write.
pub fn unchanged(reference: &Path, actual: &Path, offset: u64, bytes: u64) -> Result<()> {
    let before = std::fs::read(reference)?;
    let after = std::fs::read(actual)?;
    let start = usize::try_from(offset)?;
    let end = usize::try_from(offset.checked_add(bytes).ok_or("region overflow")?)?;
    if before.len() != after.len()
        || end > before.len()
        || before[..start] != after[..start]
        || before[end..] != after[end..]
    {
        return Err("volume size or guard bytes changed".into());
    }
    let read = |path: &Path| -> Result<_> {
        let region = Region::new(Arc::new(Image::open(path)?), offset, bytes)?;
        let header = region.headers()?;
        let rows = compact::records(&open(region)?)?;
        Ok((header.id, header.region_bytes, header.ready, rows))
    };
    if read(reference)? != read(actual)? {
        return Err("volume identity, state or records changed".into());
    }
    println!(
        "VERIFIED: all tables/records, volume identity and guard bytes unchanged ({} physical bytes differ)",
        before.iter().zip(&after).filter(|(a, b)| a != b).count()
    );
    Ok(())
}

pub(super) fn report(actual: &BTreeMap<Vec<u8>, Vec<u8>>, expected: &BTreeMap<Vec<u8>, Vec<u8>>) {
    let describe = |key: &[u8]| {
        let kind = key.get(1).map_or('?', |k| *k as char);
        format!(
            "root {} kind {kind} key {}",
            key.first().unwrap_or(&255),
            hex(key)
        )
    };
    for (key, value) in actual {
        match expected.get(key) {
            None => println!("UNEXPECTED {} ({} bytes)", describe(key), value.len()),
            Some(other) if other != value => println!(
                "DIFFERENT  {} ({} bytes, expected {} bytes){}",
                describe(key),
                value.len(),
                other.len(),
                if value.len() <= 64 && other.len() <= 64 {
                    format!(" actual {} expected {}", hex(value), hex(other))
                } else {
                    String::new()
                }
            ),
            Some(_) => {}
        }
    }
    for (key, value) in expected {
        if !actual.contains_key(key) {
            println!("MISSING    {} ({} bytes)", describe(key), value.len());
        }
    }
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
