//! Rasterize a bounded note into one sealed bitmap and show the note count as
//! retained rectangles: a track for the capacity, a fill for the notes that
//! exist and a mark on the current one. The ordinary matrix surface chain owns
//! clipping/placement and shares the bitmap's read capability without copying.
use super::*;
use cathedral_user_runtime::memory::Private;
const WIDTH: usize = 180;
const HEIGHT: usize = 64;
const COLUMNS: usize = 15;
const ROWS: usize = 4;
const TAB_PITCH: u64 = 16;
const TAB_TOP: u64 = 88;
const TAB_HEIGHT: u64 = 8;
pub struct Scene {
    pub epoch: u64,
    base: u64,
    text: u64,
    track: u64,
    filled: u64,
    current: u64,
    close: u64,
}
impl Scene {
    pub fn new(parent: Pair, epoch: u64, temporary: bool) -> Result<Self, u64> {
        let mut names = [0; 6];
        for (rank, name) in names
            .iter_mut()
            .enumerate()
            .take(if temporary { 4 } else { 6 })
        {
            let reply = call(
                parent,
                [wire::SURFACE_CREATE, epoch, rank as u64, 0, 0, 0, 0, 0],
            );
            accepted(reply[0])?;
            *name = reply[1];
            #[cfg(feature = "render-probes")]
            if rank == 0 {
                super::render_probe::gate(1);
            }
        }
        Ok(Self {
            epoch,
            base: names[0],
            text: names[1],
            track: names[2],
            filled: names[3],
            current: names[4],
            close: names[if temporary { 3 } else { 5 }],
        })
    }
    /// Paint one retained rectangle at a local rectangle: x, y, width, height.
    fn rectangle(
        &self,
        parent: Pair,
        surface: u64,
        [x, y, width, height]: [u64; 4],
        color: u64,
    ) -> Result<(), u64> {
        let reply = call(
            parent,
            [wire::PAINT, self.epoch, surface, x, y, width, height, color],
        );
        accepted(reply[0])
    }
    #[allow(clippy::too_many_arguments)]
    pub fn paint(
        &self,
        parent: Pair,
        extent: wire::Packet,
        doc: &Document,
        cursor: usize,
        dirty: bool,
        error: u64,
        notes: &storage::Notes,
        closing: bool,
    ) -> Result<(), u64> {
        let color = if error != 0 {
            0x993f46
        } else if dirty {
            0x9b7028
        } else if notes.temporary() {
            0x55447a
        } else {
            0x27664a
        };
        self.rectangle(parent, self.base, [0, 0, extent[2], extent[3]], color)?;
        self.image(parent, self.text, [10, 20, WIDTH, HEIGHT], |bytes| {
            if closing {
                label(
                    bytes,
                    WIDTH,
                    0,
                    0,
                    if error == 0 {
                        b"UNSAVED NOTE"
                    } else {
                        b"SAVE ERROR"
                    },
                );
                label(
                    bytes,
                    WIDTH,
                    0,
                    16,
                    if notes.temporary() {
                        b"NO STORAGE"
                    } else {
                        b"SAVE"
                    },
                );
                label(bytes, WIDTH, 0, 32, b"DISCARD");
                label(bytes, WIDTH, 0, 48, b"CANCEL");
            } else {
                text(bytes, doc, cursor);
            }
        })?;
        self.image(
            parent,
            self.close,
            [extent[2].saturating_sub(24) as usize, 2, 20, 16],
            |bytes| {
                for pixel in bytes.chunks_exact_mut(4) {
                    pixel.copy_from_slice(&0x993f46u32.to_le_bytes());
                }
                label(bytes, 20, 5, 1, b"X");
            },
        )?;
        if notes.temporary() {
            self.image(
                parent,
                self.track,
                [10, TAB_TOP as usize, WIDTH, 16],
                |bytes| {
                    for (i, &ch) in b"TEMP NO SAVE".iter().enumerate() {
                        for (y, bits) in cathedral_bitmap_font::glyph(ch).into_iter().enumerate() {
                            for x in 0..5 {
                                if bits & (1 << (4 - x)) != 0 {
                                    for dy in 0..2 {
                                        for dx in 0..2 {
                                            pixel(
                                                bytes,
                                                WIDTH,
                                                i * 12 + x * 2 + dx,
                                                y * 2 + dy,
                                                0xffffff,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
            )?;
            return Ok(());
        }
        let width = |count: usize| count as u64 * TAB_PITCH - 4;
        let bar = |x: u64, width: u64| [x, TAB_TOP, width, TAB_HEIGHT];
        self.rectangle(
            parent,
            self.track,
            bar(10, width(storage::MAX_NOTES)),
            0x182132,
        )?;
        if notes.count > 0 {
            self.rectangle(parent, self.filled, bar(10, width(notes.count)), 0x3a4a63)?;
            let x = 10 + notes.current as u64 * TAB_PITCH;
            self.rectangle(parent, self.current, bar(x, width(1)), 0x66ccff)?;
        }
        Ok(())
    }
    /// Publish one sealed image at a local rectangle: x, y, width, height.
    fn image(
        &self,
        parent: Pair,
        surface: u64,
        [x, y, width, height]: [usize; 4],
        draw: impl FnOnce(&mut [u8]),
    ) -> Result<(), u64> {
        let mut pixels = Private::allocate((width * height * 4).div_ceil(4096)).unwrap();
        for pixel in pixels.bytes_mut()[..width * height * 4].chunks_exact_mut(4) {
            pixel.copy_from_slice(&0x182132u32.to_le_bytes());
        }
        draw(&mut pixels.bytes_mut()[..width * height * 4]);
        let mut object = pixels.freeze().unwrap();
        object.unmap().unwrap();
        parent
            .0
            .send_memory(
                &wire::encode([
                    wire::SURFACE_IMAGE,
                    self.epoch,
                    surface,
                    x as u64,
                    y as u64,
                    width as u64,
                    height as u64,
                    width as u64,
                ]),
                &object,
            )
            .unwrap();
        let mut reply = [0; 64];
        let n = parent
            .1
            .receive_until(&mut reply, time::after(300).unwrap())
            .unwrap();
        let result = accepted(wire::decode(&reply[..n]).unwrap()[0]);
        #[cfg(feature = "render-probes")]
        if result.is_ok() {
            super::render_probe::gate(2);
        }
        result
    }
}
/// Glyph rows follow the cursor: the window shows the four rows around it.
fn text(bytes: &mut [u8], doc: &Document, cursor: usize) {
    let mut positions = [(0usize, 0usize); cathedral_notes::document::CAPACITY + 1];
    let (mut x, mut y) = (0, 0);
    for (i, position) in positions.iter_mut().enumerate().take(doc.len + 1) {
        *position = (x, y);
        if i < doc.len {
            if doc.bytes[i] == b'\n' {
                x = 0;
                y += 1;
            } else {
                x += 1;
                if x == COLUMNS {
                    x = 0;
                    y += 1;
                }
            }
        }
    }
    let top = positions[cursor].1.saturating_sub(ROWS - 1);
    for (i, &(x, y)) in positions[..doc.len].iter().enumerate() {
        if y < top || y >= top + ROWS || doc.bytes[i] == b'\n' {
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
                                WIDTH,
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
        pixel(bytes, WIDTH, x * 12 + dx, (y - top) * 16 + 14, 0x66ccff);
    }
}
fn pixel(bytes: &mut [u8], width: usize, x: usize, y: usize, color: u32) {
    let offset = (y * width + x) * 4;
    bytes[offset..offset + 4].copy_from_slice(&color.to_le_bytes());
}
fn label(bytes: &mut [u8], width: usize, left: usize, top: usize, text: &[u8]) {
    for (i, &ch) in text.iter().enumerate() {
        for (y, bits) in cathedral_bitmap_font::glyph(ch).into_iter().enumerate() {
            for x in 0..5 {
                if bits & (1 << (4 - x)) != 0 {
                    for dy in 0..2 {
                        for dx in 0..2 {
                            pixel(
                                bytes,
                                width,
                                left + i * 12 + x * 2 + dx,
                                top + y * 2 + dy,
                                0xffffff,
                            );
                        }
                    }
                }
            }
        }
    }
}

fn accepted(status: u64) -> Result<(), u64> {
    if status == 0 { Ok(()) } else { Err(status) }
}
