// Validate saved desktop configuration and calculate placement in Omega.
// Cross-language entrance: source/distribution/matrix/configuration/main.omg.
// These synchronous values carry no capability, endpoint or task identity.
use cathedral_desktop_matrix_contracts::matrix::config::Config;

pub fn decode(bytes: &[u8]) -> Option<Config> {
    if bytes.len() != 104 {
        return None;
    }
    let mut p = [0; 128];
    p[0] = 1;
    p[1] = bytes.len() as u64;
    for (slot, byte) in p[16..120].iter_mut().zip(bytes) {
        *slot = *byte as u64;
    }
    if native::decide(&mut p) != 1 {
        return None;
    }
    let word = |at: usize| u64::from_le_bytes(core::array::from_fn(|i| p[16 + at + i] as u8));
    Some(Config {
        launcher: core::array::from_fn(|i| p[24 + i] as u8),
        launcher_pages: word(40),
        app_pages: word(48),
        app_readers: word(56),
        margin: word(64),
        launcher_height: word(72),
        gap: word(80),
        app_surfaces: word(88),
        keys: word(96),
    })
}
pub fn rectangles(
    margin: u64,
    bar: u64,
    gap: u64,
    width: u64,
    height: u64,
) -> Option<[[u64; 4]; 2]> {
    let mut p = [0; 128];
    p[..6].copy_from_slice(&[2, margin, bar, gap, width, height]);
    if native::decide(&mut p) != 1 {
        return None;
    }
    Some(core::array::from_fn(|row| {
        core::array::from_fn(|col| p[16 + row * 4 + col])
    }))
}
