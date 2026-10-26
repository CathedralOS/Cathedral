//! One retained card; the matrix supplies the background and clips local input.
use cathedral_desktop_matrix_contracts::matrix as wire;
use cathedral_user_runtime::{memory::Private, services::Services};
fn height(launcher: bool) -> usize {
    if launcher { 64 } else { 112 }
}
const WIDTH: usize = 288;
fn origin(view: wire::Packet, launcher: bool) -> (u64, u64) {
    (
        view[2].saturating_sub(WIDTH as u64) / 2,
        view[3].saturating_sub(height(launcher) as u64) / 2,
    )
}
pub(crate) fn button(view: wire::Packet, x: u64, y: u64, launcher: bool) -> bool {
    let (left, top) = origin(view, launcher);
    let (start, end) = (64, 224);
    let button_top = if launcher { 32 } else { 64 };
    (left + start..left + end).contains(&x)
        && (top + button_top..top + button_top + 32).contains(&y)
}
pub(crate) fn paint(
    parent: &mut Services,
    view: wire::Packet,
    surface: u64,
    failed: bool,
    launcher: bool,
) {
    let width = WIDTH;
    let (start, end) = (64, 224);
    let button_top = if launcher { 32 } else { 64 };
    let mut pixels = Private::allocate((width * height(launcher) * 4).div_ceil(4096)).unwrap();
    for (index, pixel) in pixels.bytes_mut()[..width * height(launcher) * 4]
        .chunks_exact_mut(4)
        .enumerate()
    {
        let x = index % width;
        let y = index / width;
        let color: u32 =
            if failed && (start..end).contains(&x) && (button_top..button_top + 32).contains(&y) {
                0x315ca8
            } else {
                0x101827
            };
        pixel.copy_from_slice(&color.to_le_bytes());
    }
    for (y, text) in [
        (
            if launcher { 2 } else { 16 },
            if launcher {
                b"LAUNCHER UNAVAILABLE".as_slice()
            } else if failed {
                b"SIGN-IN UNAVAILABLE".as_slice()
            } else {
                b"STARTING...".as_slice()
            },
        ),
        (
            if launcher { 42 } else { 77 },
            if failed {
                b"TRY AGAIN".as_slice()
            } else {
                b"".as_slice()
            },
        ),
    ] {
        let left = (width - text.len() * 12) / 2;
        for (i, &ch) in text.iter().enumerate() {
            for (row, bits) in cathedral_bitmap_font::glyph(ch).into_iter().enumerate() {
                for col in 0..5 {
                    if bits & (1 << (4 - col)) == 0 {
                        continue;
                    }
                    for dy in 0..2 {
                        for dx in 0..2 {
                            let at =
                                ((y + row * 2 + dy) * width + left + i * 12 + col * 2 + dx) * 4;
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
    let (x, y) = origin(view, launcher);
    let reply = parent
        .request_interaction(
            [
                wire::SURFACE_IMAGE,
                view[1],
                surface,
                x,
                y,
                width as u64,
                height(launcher) as u64,
                width as u64,
            ],
            Some(&image),
        )
        .unwrap();
    assert_eq!(reply, [0; 8]);
}
