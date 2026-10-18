//! Colors owns its close button; the matrix reclaims the app after normal exit.
use super::*;
use cathedral_user_runtime::memory::Private;
#[derive(Default)]
pub(super) struct Button {
    pub confirm: bool,
    pressed: bool,
}
impl Button {
    pub fn close(&mut self, event: wire::Packet, width: u64, uncertain: bool) -> bool {
        if event[2] == wire::CANCEL {
            self.pressed = false;
        }
        if event[2] != wire::POINTER {
            return false;
        }
        let down = event[5] & 1 != 0;
        let clicked = down && !self.pressed;
        self.pressed = down;
        if !clicked {
            return false;
        }
        let size = if self.confirm { 144 } else { 20 };
        if (width.saturating_sub(size + 4)..width.saturating_sub(4)).contains(&event[3])
            && (2..18).contains(&event[4])
        {
            if !uncertain || self.confirm {
                return true;
            }
            self.confirm = true;
        } else {
            self.confirm = false;
        }
        false
    }
}
pub(super) fn paint(parent: Pair, view: wire::Packet, surface: u64, confirm: bool) {
    let (width, text): (usize, &[u8]) = if confirm {
        (144, b"CLOSE ANYWAY")
    } else {
        (20, b"X")
    };
    let mut pixels = Private::allocate((width * 16 * 4).div_ceil(4096)).unwrap();
    for pixel in pixels.bytes_mut()[..width * 16 * 4].chunks_exact_mut(4) {
        pixel.copy_from_slice(&0x993f46u32.to_le_bytes());
    }
    for (i, &ch) in text.iter().enumerate() {
        for (y, bits) in cathedral_bitmap_font::glyph(ch).into_iter().enumerate() {
            for x in 0..5 {
                if bits & (1 << (4 - x)) != 0 {
                    for dy in 0..2 {
                        for dx in 0..2 {
                            let at = ((1 + y * 2 + dy) * width + 5 + i * 12 + x * 2 + dx) * 4;
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
    parent
        .0
        .send_memory(
            &wire::encode([
                wire::SURFACE_IMAGE,
                view[1],
                surface,
                view[2].saturating_sub(width as u64 + 4),
                2,
                width as u64,
                16,
                width as u64,
            ]),
            &image,
        )
        .unwrap();
    let mut bytes = [0; 64];
    let len = parent
        .1
        .receive_until(&mut bytes, time::after(300).unwrap())
        .unwrap();
    assert_eq!(wire::decode(&bytes[..len]).unwrap()[0], 0);
}
