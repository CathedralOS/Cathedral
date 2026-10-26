//! Omega plans stock layout; the host rechecks the live grant and applies atomically.
use cathedral_platform_contracts::matrix::Packet;
#[derive(Debug, PartialEq, Eq)]
pub struct Plan {
    pub rectangles: [[u64; 4]; 5],
    pub remember: u64,
}
#[inline(never)]
pub fn prepare(
    words: Packet,
    authorized: bool,
    busy: bool,
    extent: [u64; 2],
    standing: u64,
) -> Result<Plan, u64> {
    let mut p = [0; 128];
    p[0] = 18;
    p[2..7].copy_from_slice(&[
        authorized.into(),
        busy.into(),
        extent[0],
        extent[1],
        standing,
    ]);
    p[16..24].copy_from_slice(&words);
    match super::native::decide(&mut p) {
        0 => Ok(Plan {
            rectangles: core::array::from_fn(|slot| {
                core::array::from_fn(|coordinate| p[32 + slot * 4 + coordinate])
            }),
            remember: p[52],
        }),
        status => Err(status),
    }
}
