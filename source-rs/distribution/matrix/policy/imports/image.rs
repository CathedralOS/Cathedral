//! Feed Omega's bounded reads from one immutable image; interpret no ELF fields here.
use cathedral_platform_contracts::imports::Refusal;
#[inline(never)]
pub(super) fn locate(bytes: &[u8]) -> Result<Option<&[u8]>, Refusal> {
    let mut p = [0; 128];
    p[..3].copy_from_slice(&[11, 12, bytes.len() as u64]);
    loop {
        match super::super::native::decide(&mut p) {
            1 => {
                let read = slice(bytes, p[4], p[5])?;
                if read.len() > 64 {
                    return Err(Refusal::Malformed);
                }
                p[16..24].fill(0);
                for (i, byte) in read.iter().enumerate() {
                    p[16 + i / 8] |= u64::from(*byte) << ((i % 8) * 8);
                }
                p[1] = 13;
                p[2] = read.len() as u64;
            }
            2 => return Ok(None),
            3 => return slice(bytes, p[4], p[5]).map(Some),
            4 => return Err(Refusal::Malformed),
            _ => unreachable!("Omega image scan outcome"),
        }
    }
}
fn slice(bytes: &[u8], offset: u64, length: u64) -> Result<&[u8], Refusal> {
    let start = usize::try_from(offset).map_err(|_| Refusal::Malformed)?;
    let count = usize::try_from(length).map_err(|_| Refusal::Malformed)?;
    let end = start.checked_add(count).ok_or(Refusal::Malformed)?;
    bytes.get(start..end).ok_or(Refusal::Malformed)
}
