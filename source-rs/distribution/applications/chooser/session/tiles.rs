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
    let mut pixels = Private::allocate((WIDTH * HEIGHT * 4).div_ceil(4096)).unwrap();
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let edge = x < 2 || y < 2 || x >= WIDTH - 2 || y >= HEIGHT - 2;
            let mut color = if selected && edge { 0x66ccff } else { CARD };
            let dx = x as i32 - 64;
            let dy = y as i32 - 36;
            if picker {
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
            } else if title.starts_with(b"NOTES") {
                if (43..85).contains(&x) && (20..74).contains(&y) {
                    color = 0xd9e4ef;
                }
                if (51..77).contains(&x) && [34, 35, 44, 45, 54, 55, 64, 65].contains(&y) {
                    color = 0x53718a;
                }
            } else if title.starts_with(b"SPLIT") {
                if (36..61).contains(&x) && (24..72).contains(&y) {
                    color = 0x84d4ad;
                }
                if (67..92).contains(&x) && (24..72).contains(&y) {
                    color = 0xc7a2ed;
                }
            } else {
                for (cx, cy, c) in [
                    (52, 34, 0x66ccff),
                    (76, 34, 0xc7a2ed),
                    (52, 58, 0x84d4ad),
                    (76, 58, 0xeeb46a),
                ] {
                    if (x as i32 - cx).pow(2) + (y as i32 - cy).pow(2) <= 12 * 12 {
                        color = c;
                    }
                }
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
