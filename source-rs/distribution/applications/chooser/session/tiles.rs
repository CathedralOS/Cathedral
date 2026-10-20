//! Small raster tiles: avatar, add-user, document and palette; no image assets.
use super::{
    NEW,
    layout::{HEIGHT, WIDTH},
};
use cathedral_user_runtime::memory::Private;
pub const BACKGROUND: u32 = 0x101827;
pub const CARD: u32 = 0x202d43;
pub fn label(pixels: &mut [u8], width: usize, left: usize, top: usize, text: &[u8]) {
    for (i, &ch) in text.iter().take_while(|&&c| c != 0).enumerate() {
        for (y, bits) in cathedral_bitmap_font::glyph(ch).into_iter().enumerate() {
            for x in 0..5 {
                if bits & (1 << (4 - x)) != 0 {
                    for dy in 0..2 {
                        for dx in 0..2 {
                            let offset =
                                ((top + y * 2 + dy) * width + left + i * 12 + x * 2 + dx) * 4;
                            if offset + 4 <= pixels.len() && left + i * 12 + x * 2 + dx < width {
                                pixels[offset..offset + 4]
                                    .copy_from_slice(&0xffffffu32.to_le_bytes());
                            }
                        }
                    }
                }
            }
        }
    }
}
pub fn paint(index: usize, picker: bool, selected: bool, title: &[u8]) -> Private {
    if !picker {
        return icon(selected, title);
    }
    let mut pixels = Private::allocate((WIDTH * HEIGHT * 4).div_ceil(4096)).unwrap();
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let edge = x < 2 || y < 2 || x >= WIDTH - 2 || y >= HEIGHT - 2;
            let mut color = if selected && edge { 0x66ccff } else { CARD };
            let dx = x as i32 - 64;
            let dy = y as i32 - 36;
            if index == NEW {
                if ((61..67).contains(&x) && (24..68).contains(&y))
                    || ((42..86).contains(&x) && (43..49).contains(&y))
                {
                    color = 0x91a8c8;
                }
            } else if dx * dx + dy * dy <= 14 * 14
                || ((52..76).contains(&y) && dx * dx + (y as i32 - 76).pow(2) < 30 * 30)
            {
                color = [0x66ccff, 0xc7a2ed, 0x84d4ad][(index - 1) % 3];
            }
            let at = (y * WIDTH + x) * 4;
            pixels.bytes_mut()[at..at + 4].copy_from_slice(&color.to_le_bytes());
        }
    }
    let len = title.iter().take_while(|&&c| c != 0).count().min(10);
    label(
        pixels.bytes_mut(),
        WIDTH,
        (WIDTH - len * 12) / 2,
        76,
        &title[..len],
    );
    pixels
}

fn icon(selected: bool, title: &[u8]) -> Private {
    const SIDE: usize = super::layout::ICON;
    let mut pixels = Private::allocate((SIDE * SIDE * 4).div_ceil(4096)).unwrap();
    for y in 0..SIDE {
        for x in 0..SIDE {
            let mut color: u32 = if selected && y >= SIDE - 3 {
                0x66ccff
            } else {
                CARD
            };
            if title.starts_with(b"NOTES") {
                if (14..34).contains(&x) && (8..36).contains(&y) {
                    color = 0xd6e5ef;
                }
                if (18..30).contains(&x) && [16, 22, 28].contains(&y) {
                    color = 0x53718a;
                }
            } else if title.starts_with(b"NESTED") {
                if (8..40).contains(&x) && (10..34).contains(&y) {
                    color = 0x66ccff;
                }
                if (11..37).contains(&x) && (13..31).contains(&y) {
                    color = CARD;
                }
                if (17..33).contains(&x) && (18..28).contains(&y) {
                    color = 0xc7a2ed;
                }
            } else if title.starts_with(b"SPLIT") {
                if (12..23).contains(&x) && (10..35).contains(&y) {
                    color = 0x84d4ad;
                }
                if (26..37).contains(&x) && (10..35).contains(&y) {
                    color = 0xc7a2ed;
                }
            } else {
                for (cx, cy, c) in [
                    (18, 17, 0x66ccff),
                    (30, 17, 0xc7a2ed),
                    (18, 29, 0x84d4ad),
                    (30, 29, 0xeeb46a),
                ] {
                    if (x as i32 - cx).pow(2) + (y as i32 - cy).pow(2) <= 36 {
                        color = c;
                    }
                }
            }
            let at = (y * SIDE + x) * 4;
            pixels.bytes_mut()[at..at + 4].copy_from_slice(&color.to_le_bytes());
        }
    }
    pixels
}
