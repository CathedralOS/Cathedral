//! Three small panels reuse the app-tile surfaces and stay within its page grant.
use super::*;
use cathedral_user_runtime::memory::Private;
pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 64;
pub fn origin(extent: wire::Packet, row: usize) -> (u64, u64) {
    (
        extent[2].saturating_sub(3 * WIDTH as u64 + 32) / 2 + row as u64 * 176,
        8,
    )
}
pub fn hit(extent: wire::Packet, x: u64, y: u64) -> Option<usize> {
    (0..3).find_map(|row| {
        let (left, top) = origin(extent, row);
        ((left..left + 160).contains(&x) && (top + 40..top + 64).contains(&y))
            .then_some(row * 2 + usize::from(x >= left + 80))
    })
}
pub fn paint(editor: &Editor, row: usize) -> Private {
    let mut pixels = Private::allocate(10).unwrap();
    for (i, pixel) in pixels.bytes_mut().chunks_exact_mut(4).enumerate() {
        let (x, y) = (i % WIDTH, i / WIDTH);
        let action = row * 2 + usize::from(x >= 80);
        let symbol = row < 2
            && ((50..52).contains(&y) && ((34..46).contains(&x) || (114..126).contains(&x))
                || (118..120).contains(&x) && (46..58).contains(&y));
        let color: u32 = if symbol {
            0xffffff
        } else if y >= 40 && action == editor.selected {
            0x53718a
        } else {
            tiles::CARD
        };
        pixel.copy_from_slice(&color.to_le_bytes());
    }
    let bytes = pixels.bytes_mut();
    tiles::label(
        bytes,
        WIDTH,
        4,
        2,
        [b"BAR HEIGHT".as_slice(), b"SPACING", b"SAVE LAYOUT"][row],
    );
    if row < 2 {
        let value = if row == 0 {
            editor.draft.launcher_height
        } else {
            editor.draft.margin
        };
        let digits = [
            b'0' + (value / 1000 % 10) as u8,
            b'0' + (value / 100 % 10) as u8,
            b'0' + (value / 10 % 10) as u8,
            b'0' + (value % 10) as u8,
        ];
        let first = digits.iter().position(|b| *b != b'0').unwrap_or(3);
        let digits = &digits[first..];
        tiles::label(bytes, WIDTH, (WIDTH - digits.len() * 12) / 2, 22, digits);
    } else {
        tiles::label(bytes, WIDTH, 10, 44, b"SAVE");
        tiles::label(bytes, WIDTH, 84, 44, b"CANCEL");
    }
    pixels
}
