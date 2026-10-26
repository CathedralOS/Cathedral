//! Marshal the common Omega declaration decoder; metadata never grants resources.
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
#[inline(never)]
pub fn decode(bytes: &[u8]) -> Result<Declaration, Refusal> {
    let mut p = [0; 128];
    p[..3].copy_from_slice(&[11, 11, bytes.len() as u64]);
    for (i, byte) in bytes.iter().take(imports::PROFILE_BYTES).enumerate() {
        p[16 + i / 8] |= u64::from(*byte) << ((i % 8) * 8);
    }
    match super::super::native::decide(&mut p) {
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
