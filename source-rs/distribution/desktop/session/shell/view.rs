//! Bounded menu bitmap; only a changed selection/status replaces its sealed image.
use super::*;
use cathedral_user_runtime::memory::Private;
const WIDTH: usize = 180;
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
        selected: u64,
        active: u64,
        blocked: u64,
    ) {
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
                    0x202d43
                ]
            )[0],
            0
        );
        let mut pixels = Private::allocate(12).unwrap();
        for (i, pixel) in pixels.bytes_mut().chunks_exact_mut(4).enumerate() {
            let row = i / WIDTH / 16;
            let color: u32 = if row < 3 && row as u64 + 1 == selected {
                0x315ca8
            } else {
                0x182132
            };
            pixel.copy_from_slice(&color.to_le_bytes());
        }
        let status: &[u8] = if blocked == UNSAVED {
            b"SAVE NOTE FIRST"
        } else if blocked == BUSY {
            b"NOTES BUSY"
        } else if active == NOTES {
            b"NOTES OPEN"
        } else if active == COLORS {
            b"COLORS OPEN"
        } else {
            b"CHOOSE PROGRAM"
        };
        for (row, label) in [b"NOTES".as_slice(), b"COLORS", b"CLOSE APP", status]
            .into_iter()
            .enumerate()
        {
            for (column, &ch) in label.iter().enumerate() {
                for (y, bits) in cathedral_bitmap_font::glyph(ch).into_iter().enumerate() {
                    for x in 0..5 {
                        if bits & (1 << (4 - x)) != 0 {
                            for dy in 0..2 {
                                for dx in 0..2 {
                                    let offset = ((row * 16 + y * 2 + dy) * WIDTH
                                        + column * 12
                                        + x * 2
                                        + dx)
                                        * 4;
                                    pixels.bytes_mut()[offset..offset + 4]
                                        .copy_from_slice(&0xffffffu32.to_le_bytes());
                                }
                            }
                        }
                    }
                }
            }
        }
        let mut image = pixels.freeze().unwrap();
        image.unmap().unwrap();
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
                    64,
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
        assert_eq!(wire::decode(&bytes[..n]).unwrap()[0], 0);
    }
}
