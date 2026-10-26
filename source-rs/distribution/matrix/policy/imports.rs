//! Common Omega interface matching plus the stock matrix's supported offer.
use cathedral_platform_contracts::imports::{Declaration, Import, Refusal};
pub fn resolve(declaration: Option<&Declaration>, offered: u64) -> Result<u64, Refusal> {
    decide(
        9,
        declaration.is_some(),
        declaration.is_some_and(|d| d.brokered),
        declaration.map_or(&[], Declaration::entries),
        offered,
    )
}
#[cfg(test)]
pub fn bind(entries: &[Import], offered: u64) -> Result<u64, Refusal> {
    decide(10, true, true, entries, offered)
}
#[inline(never)]
fn decide(
    operation: u64,
    present: bool,
    brokered: bool,
    entries: &[Import],
    offered: u64,
) -> Result<u64, Refusal> {
    let mut p = [0; 128];
    p[..6].copy_from_slice(&[
        11,
        operation,
        present.into(),
        brokered.into(),
        entries.len() as u64,
        offered,
    ]);
    for (i, entry) in entries.iter().take(4).enumerate() {
        p[16 + i * 4..20 + i * 4].copy_from_slice(&[
            entry.contract,
            entry.minimum,
            entry.maximum,
            entry.required.into(),
        ]);
    }
    match super::native::decide(&mut p) {
        0 => Ok(p[32]),
        1 => Err(Refusal::Malformed),
        2 => Err(Refusal::Missing(p[33])),
        3 => Err(Refusal::Version(p[33])),
        _ => unreachable!("Omega import refusal"),
    }
}
