//! Bounded bitmap of fixed rows; only a changed selection or status replaces
//! its sealed image. Entries show their object numbers.
use super::*;
use cathedral_user_runtime::memory::Private;
pub const WIDTH: usize = 160;
pub const ROW: u64 = 16;
const HEIGHT: usize = ROWS * ROW as usize;
/// Where the bitmap sits in the chooser's extent.
pub const LEFT: u64 = 10;
pub const TOP: u64 = 20;
/// Pages one bitmap occupies; the chooser holds two during replacement.
pub const PAGES: usize = (WIDTH * HEIGHT * 4).div_ceil(4096);
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
        realm: &realm::Realm,
        selected: usize,
        status: &str,
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
        let mut pixels = Private::allocate(PAGES).unwrap();
        for (i, pixel) in pixels.bytes_mut()[..WIDTH * HEIGHT * 4]
            .chunks_exact_mut(4)
            .enumerate()
        {
            let row = i / WIDTH / ROW as usize;
            let color: u32 = if row == selected && row <= CLOSE {
                0x315ca8
            } else {
                0x182132
            };
            pixel.copy_from_slice(&color.to_le_bytes());
        }
        let mut labels = [[0u8; 13]; ROWS];
        for (row, label) in labels.iter_mut().enumerate() {
            let text: &[u8] = match row {
                row if row < realm.count => {
                    let mut line = *b"OBJECT       ";
                    let digits = decimal(realm.ids[row]);
                    line[7..].copy_from_slice(&digits);
                    *label = line;
                    continue;
                }
                NEW => b"NEW",
                CLOSE => b"CLOSE",
                row if row == ROWS - 1 => status.as_bytes(),
                _ => b"",
            };
            label[..text.len().min(13)].copy_from_slice(&text[..text.len().min(13)]);
        }
        for (row, label) in labels.iter().enumerate() {
            for (column, &ch) in label.iter().enumerate() {
                if ch == 0 {
                    continue;
                }
                for (y, bits) in cathedral_bitmap_font::glyph(ch).into_iter().enumerate() {
                    for x in 0..5 {
                        if bits & (1 << (4 - x)) != 0 {
                            for dy in 0..2 {
                                for dx in 0..2 {
                                    let offset = ((row * ROW as usize + y * 2 + dy) * WIDTH
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
                    LEFT,
                    TOP,
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
        assert_eq!(wire::decode(&bytes[..n]).unwrap()[0], 0);
    }
}
/// Six right-aligned decimal digits, blank-padded; larger numbers show their
/// low six digits, which is honest for a lab that never reaches them.
fn decimal(mut value: u64) -> [u8; 6] {
    let mut digits = *b"      ";
    let mut at = 6;
    loop {
        at -= 1;
        digits[at] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 || at == 0 {
            break;
        }
    }
    digits
}
