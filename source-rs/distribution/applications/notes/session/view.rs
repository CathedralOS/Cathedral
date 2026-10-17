//! Rasterize a bounded note into one sealed bitmap. The ordinary matrix surface
//! chain owns clipping/placement and shares its read capability without copying.
use super::*;
use cathedral_user_runtime::memory::Private;
const WIDTH: usize = 180;
const HEIGHT: usize = 64;
pub struct Scene {
    pub epoch: u64,
    base: u64,
    text: u64,
}
impl Scene {
    pub fn new(parent: Pair, epoch: u64) -> Self {
        let mut names = [0; 2];
        for (rank, name) in names.iter_mut().enumerate() {
            let reply = call(
                parent,
                [wire::SURFACE_CREATE, epoch, rank as u64, 0, 0, 0, 0, 0],
            );
            assert_eq!(reply[0], 0);
            *name = reply[1];
        }
        Self {
            epoch,
            base: names[0],
            text: names[1],
        }
    }
    pub fn paint(
        &self,
        parent: Pair,
        extent: wire::Packet,
        doc: &Document,
        cursor: usize,
        dirty: bool,
        error: u64,
    ) {
        let color = if error != 0 {
            0x993f46
        } else if dirty {
            0x9b7028
        } else {
            0x27664a
        };
        assert_eq!(
            call(
                parent,
                [
                    wire::PAINT,
                    self.epoch,
                    self.base,
                    0,
                    0,
                    extent[2],
                    extent[3],
                    color
                ]
            )[0],
            0
        );
        let mut pixels = Private::allocate(12).unwrap();
        for pixel in pixels.bytes_mut().chunks_exact_mut(4) {
            pixel.copy_from_slice(&0x182132u32.to_le_bytes());
        }
        let mut positions = [(0usize, 0usize); 61];
        let (mut x, mut y) = (0, 0);
        for (i, position) in positions.iter_mut().enumerate().take(doc.len + 1) {
            *position = (x, y);
            if i < doc.len {
                if doc.bytes[i] == b'\n' {
                    x = 0;
                    y += 1;
                } else {
                    x += 1;
                    if x == 15 {
                        x = 0;
                        y += 1;
                    }
                }
            }
        }
        let top = positions[cursor].1.saturating_sub(3);
        let bytes = pixels.bytes_mut();
        for (i, &(x, y)) in positions[..doc.len].iter().enumerate() {
            if y < top || y >= top + 4 || doc.bytes[i] == b'\n' {
                continue;
            }
            for (row, bits) in cathedral_bitmap_font::glyph(doc.bytes[i])
                .into_iter()
                .enumerate()
            {
                for col in 0..5 {
                    if bits & (1 << (4 - col)) != 0 {
                        for dy in 0..2 {
                            for dx in 0..2 {
                                pixel(
                                    bytes,
                                    x * 12 + col * 2 + dx,
                                    (y - top) * 16 + row * 2 + dy,
                                    0xffffff,
                                );
                            }
                        }
                    }
                }
            }
        }
        let (x, y) = positions[cursor];
        for dx in 0..10 {
            pixel(bytes, x * 12 + dx, (y - top) * 16 + 14, 0x66ccff);
        }
        let mut object = pixels.freeze().unwrap();
        object.unmap().unwrap();
        parent
            .0
            .send_memory(
                &wire::encode([
                    wire::SURFACE_IMAGE,
                    self.epoch,
                    self.text,
                    10,
                    20,
                    WIDTH as u64,
                    HEIGHT as u64,
                    WIDTH as u64,
                ]),
                &object,
            )
            .unwrap();
        let mut reply = [0; 64];
        let n = parent
            .1
            .receive_until(&mut reply, time::after(300).unwrap())
            .unwrap();
        assert_eq!(wire::decode(&reply[..n]).unwrap()[0], 0);
    }
}
fn pixel(bytes: &mut [u8], x: usize, y: usize, color: u32) {
    assert!(x < WIDTH && y < HEIGHT);
    let offset = (y * WIDTH + x) * 4;
    bytes[offset..offset + 4].copy_from_slice(&color.to_le_bytes());
}
