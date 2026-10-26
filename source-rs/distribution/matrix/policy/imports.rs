//! Common Omega interface matching plus the stock matrix's supported offer.
use cathedral_platform_contracts::imports::{self, Import, Refusal};
/// Validated declaration returned by Omega; resource requests are not grants.
#[derive(Debug, PartialEq, Eq)]
pub struct Declaration {
    pub brokered: bool,
    pub profile: Option<[u64; 3]>,
    pub resources: Option<(u64, u64)>,
    entries: [Import; 4],
    count: usize,
}
impl Declaration {
    pub fn entries(&self) -> &[Import] {
        &self.entries[..self.count]
    }
}
/// Host locates the sealed ELF record; Omega owns its full contents validation.
pub fn from_elf(bytes: &[u8]) -> Result<Option<Declaration>, Refusal> {
    imports::record_from_elf(bytes)?.map(decode).transpose()
}
#[inline(never)]
pub fn decode(bytes: &[u8]) -> Result<Declaration, Refusal> {
    let mut p = [0; 128];
    p[..3].copy_from_slice(&[11, 11, bytes.len() as u64]);
    for (i, byte) in bytes.iter().take(imports::PROFILE_BYTES).enumerate() {
        p[16 + i / 8] |= u64::from(*byte) << ((i % 8) * 8);
    }
    match super::native::decide(&mut p) {
        0 => Ok(Declaration {
            brokered: p[65] != 0,
            resources: (p[66] != 0).then_some((p[67], p[68])),
            profile: (p[69] != 0).then_some([p[70], p[71], p[72]]),
            entries: core::array::from_fn(|i| Import {
                contract: p[80 + i * 4],
                minimum: p[81 + i * 4],
                maximum: p[82 + i * 4],
                required: p[83 + i * 4] != 0,
            }),
            count: p[64] as usize,
        }),
        1 => Err(Refusal::Malformed),
        _ => unreachable!("Omega declaration refusal"),
    }
}
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
