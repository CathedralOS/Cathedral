//! One retained card; the matrix supplies the background and clips local input.
use crate::recovery::Pair;
use cathedral_platform_contracts::matrix as wire;
use cathedral_user_runtime::{memory::Private, time};
const WIDTH: usize = 288;
const HEIGHT: usize = 112;
fn origin(view: wire::Packet) -> (u64, u64) {
    (
        view[2].saturating_sub(WIDTH as u64) / 2,
        view[3].saturating_sub(HEIGHT as u64) / 2,
    )
}
pub(crate) fn button(view: wire::Packet, x: u64, y: u64) -> bool {
    let (left, top) = origin(view);
    (left + 64..left + 224).contains(&x) && (top + 64..top + 104).contains(&y)
}
pub(crate) fn paint(parent: Pair, view: wire::Packet, surface: u64, failed: bool) {
    let mut pixels = Private::allocate((WIDTH * HEIGHT * 4).div_ceil(4096)).unwrap();
    for (index, pixel) in pixels.bytes_mut()[..WIDTH * HEIGHT * 4]
        .chunks_exact_mut(4)
        .enumerate()
    {
        let x = index % WIDTH;
        let y = index / WIDTH;
        let color: u32 = if failed && (64..224).contains(&x) && (64..104).contains(&y) {
            0x315ca8
        } else {
            0x101827
        };
        pixel.copy_from_slice(&color.to_le_bytes());
    }
    for (y, text) in [
        (
            16,
            if failed {
                b"SIGN-IN UNAVAILABLE".as_slice()
            } else {
                b"STARTING...".as_slice()
            },
        ),
        (
            77,
            if failed {
                b"TRY AGAIN".as_slice()
            } else {
                b"".as_slice()
            },
        ),
    ] {
        let left = (WIDTH - text.len() * 12) / 2;
        for (i, &ch) in text.iter().enumerate() {
            for (row, bits) in cathedral_bitmap_font::glyph(ch).into_iter().enumerate() {
                for col in 0..5 {
                    if bits & (1 << (4 - col)) == 0 {
                        continue;
                    }
                    for dy in 0..2 {
                        for dx in 0..2 {
                            let at =
                                ((y + row * 2 + dy) * WIDTH + left + i * 12 + col * 2 + dx) * 4;
                            pixels.bytes_mut()[at..at + 4]
                                .copy_from_slice(&0xffffffu32.to_le_bytes());
                        }
                    }
                }
            }
        }
    }
    let mut image = pixels.freeze().unwrap();
    image.unmap().unwrap();
    let (x, y) = origin(view);
    parent
        .0
        .send_memory(
            &wire::encode([
                wire::SURFACE_IMAGE,
                view[1],
                surface,
                x,
                y,
                WIDTH as u64,
                HEIGHT as u64,
                WIDTH as u64,
            ]),
            &image,
        )
        .unwrap();
    let mut bytes = [0; 64];
    let n = parent
        .1
        .receive_until(&mut bytes, time::after(300).unwrap())
        .unwrap();
    assert_eq!(wire::decode(&bytes[..n]).unwrap(), [0; 8]);
}
