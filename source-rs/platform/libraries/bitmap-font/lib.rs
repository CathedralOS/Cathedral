#![no_std]
#![forbid(unsafe_code)]
//! Original 5x7 diagnostic glyphs. Lowercase uses uppercase shapes; unsupported
//! printable characters use a question mark. No layout or distribution wording.
const DATA: &[u8; 95 * 15] = include_bytes!("font.hex");
pub const WIDTH: u64 = 5;
pub const HEIGHT: u64 = 7;
pub const ADVANCE: u64 = 6;
pub fn glyph(byte: u8) -> [u8; 7] {
    let byte = if (32..=126).contains(&byte) {
        byte
    } else {
        b'?'
    };
    let mut rows = [0; 7];
    for (row, value) in rows.iter_mut().enumerate() {
        let offset = (byte as usize - 32) * 15 + row * 2;
        *value = digit(DATA[offset]) * 16 + digit(DATA[offset + 1]);
    }
    rows
}
fn digit(byte: u8) -> u8 {
    if byte <= b'9' {
        byte - b'0'
    } else {
        byte - b'a' + 10
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn glyphs_stay_in_cell_and_fallback_is_visible() {
        for byte in 0..=255 {
            assert!(glyph(byte).iter().all(|row| *row < 32));
        }
        assert_eq!(glyph(b' '), [0; 7]);
        assert_eq!(glyph(0), glyph(b'?'));
        assert_eq!(glyph(b'A'), [14, 17, 17, 31, 17, 17, 17]);
    }
}
