//! Warning and two choices fit the launcher's existing three tile surfaces.
use super::*;
use cathedral_user_runtime::memory::Private;
pub const WIDTH: usize = 180;
pub fn origin(extent: wire::Packet, row: usize) -> (u64, u64) {
    (
        extent[2].saturating_sub(3 * WIDTH as u64 + 32) / 2 + row as u64 * 196,
        16,
    )
}
pub fn height(row: usize) -> usize {
    if row == 0 { 48 } else { 32 }
}
pub fn hit(extent: wire::Packet, x: u64, y: u64) -> Option<bool> {
    (1..3).find_map(|row| {
        let (left, top) = origin(extent, row);
        ((left..left + WIDTH as u64).contains(&x) && (top..top + height(row) as u64).contains(&y))
            .then_some(row == 2)
    })
}
pub fn paint(controls: &Controls, row: usize) -> Private {
    let mut pixels = Private::allocate(if row == 0 { 9 } else { 6 }).unwrap();
    let selected = row != 0 && (row == 2) == controls.cancel;
    let color: u32 = if selected { 0x53718a } else { tiles::CARD };
    for pixel in pixels.bytes_mut().chunks_exact_mut(4) {
        pixel.copy_from_slice(&color.to_le_bytes());
    }
    if row == 0 {
        tiles::label(pixels.bytes_mut(), WIDTH, 18, 4, b"UNSAVED WORK");
        tiles::label(pixels.bytes_mut(), WIDTH, 24, 26, b"MAY BE LOST");
    } else {
        let label: &[u8] = if row == 1 { b"END APP" } else { b"CANCEL" };
        tiles::label(
            pixels.bytes_mut(),
            WIDTH,
            (WIDTH - label.len() * 12) / 2,
            8,
            label,
        );
    }
    pixels
}
