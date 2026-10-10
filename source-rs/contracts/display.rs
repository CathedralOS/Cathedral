//! Experimental linear framebuffer geometry and a tiny copied-message protocol.
//! Geometry is data, never proof of device custody or a grant of authority.
pub const USER_ADDRESS: u64 = 0x0000_0080_0100_0000;
pub const MAX_BYTES: u64 = 16 * 1024 * 1024;
pub const INFO_BYTES: usize = 48;
pub const RGB: u64 = 0;
pub const BGR: u64 = 1;
pub const INFO: u64 = 0;
pub const CLEAR: u64 = 1;
pub const RECT: u64 = 2;
pub const TEXT: u64 = 3;
pub const TEXT_BYTES: usize = 64;
/// Header: TEXT, x, y, scale, color, byte count; then at most 16 printable ASCII bytes.
pub fn encode_text(
    x: u64,
    y: u64,
    scale: u64,
    color: u64,
    text: &[u8],
) -> Option<[u8; TEXT_BYTES]> {
    if text.is_empty() || text.len() > 16 || !text.iter().all(|byte| (32..=126).contains(byte)) {
        return None;
    }
    let mut bytes = [0; TEXT_BYTES];
    bytes[..48].copy_from_slice(&encode([TEXT, x, y, scale, color, text.len() as u64]));
    bytes[48..48 + text.len()].copy_from_slice(text);
    Some(bytes)
}
pub const REQUEST_BYTES: usize = 48;

/// Fully checked text geometry; providers must validate before any device write.
pub struct Text<'a> {
    pub x: u64,
    pub y: u64,
    pub scale: u64,
    pub color: u64,
    pub bytes: &'a [u8],
}
pub fn decode_text(bytes: &[u8], width: u64, height: u64) -> Option<Text<'_>> {
    if bytes.len() != TEXT_BYTES {
        return None;
    }
    let [op, x, y, scale, color, length] = decode(&bytes[..48])?;
    if op != TEXT
        || !(1..=4).contains(&scale)
        || !(1..=16).contains(&length)
        || color > 0xffffff
        || !valid_rect(width, height, x, y, length * 6 * scale, 7 * scale)
    {
        return None;
    }
    let bytes = &bytes[48..48 + length as usize];
    if !bytes.iter().all(|byte| (32..=126).contains(byte)) {
        return None;
    }
    Some(Text {
        x,
        y,
        scale,
        color,
        bytes,
    })
}

#[derive(Clone, Copy, Debug)]
pub struct Framebuffer {
    pub physical: u64,
    pub bytes: u64,
    pub width: u64,
    pub height: u64,
    pub stride: u64,
    pub format: u64,
}
impl Framebuffer {
    pub fn valid(&self) -> bool {
        self.physical != 0
            && self.physical.is_multiple_of(4096)
            && self.bytes != 0
            && self.bytes <= MAX_BYTES
            && self.bytes.is_multiple_of(4096)
            && self
                .physical
                .checked_add(self.bytes)
                .is_some_and(|end| end <= 1 << 48)
            && self.width != 0
            && self.height != 0
            && self.width <= self.stride
            && self
                .stride
                .checked_mul(self.height)
                .and_then(|pixels| pixels.checked_mul(4))
                .is_some_and(|bytes| bytes <= self.bytes)
            && matches!(self.format, RGB | BGR)
    }
    pub fn user_info(&self) -> [u8; INFO_BYTES] {
        encode([
            USER_ADDRESS,
            self.bytes,
            self.width,
            self.height,
            self.stride,
            self.format,
        ])
    }
}
pub fn encode(words: [u64; 6]) -> [u8; REQUEST_BYTES] {
    let mut bytes = [0; REQUEST_BYTES];
    for (word, output) in words.into_iter().zip(bytes.chunks_exact_mut(8)) {
        output.copy_from_slice(&word.to_le_bytes());
    }
    bytes
}
pub fn decode(bytes: &[u8]) -> Option<[u64; 6]> {
    if bytes.len() != REQUEST_BYTES {
        return None;
    }
    let mut words = [0; 6];
    for (word, input) in words.iter_mut().zip(bytes.chunks_exact(8)) {
        *word = u64::from_le_bytes(input.try_into().ok()?);
    }
    Some(words)
}
pub fn valid_rect(width: u64, height: u64, x: u64, y: u64, w: u64, h: u64) -> bool {
    w != 0
        && h != 0
        && x.checked_add(w).is_some_and(|end| end <= width)
        && y.checked_add(h).is_some_and(|end| end <= height)
}
pub fn pixel(format: u64, rgb: u64) -> u32 {
    let rgb = rgb as u32 & 0x00ff_ffff;
    if format == RGB {
        (rgb & 0xff00) | ((rgb & 0xff) << 16) | (rgb >> 16)
    } else {
        rgb
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn text_validates_the_entire_packet_before_rendering() {
        let good = encode_text(1000, 740, 4, 0xffffff, b"A").unwrap();
        assert!(decode_text(&good, 1024, 768).is_some());
        assert!(decode_text(&good[..63], 1024, 768).is_none());
        for (field, value) in [
            (0, INFO),
            (1, 1001),
            (2, 741),
            (1, u64::MAX),
            (3, 0),
            (3, 5),
            (3, u64::MAX),
            (4, 0x1000000),
            (5, 0),
            (5, 17),
            (5, u64::MAX),
        ] {
            let mut bad = good;
            bad[field * 8..field * 8 + 8].copy_from_slice(&value.to_le_bytes());
            assert!(decode_text(&bad, 1024, 768).is_none());
        }
        let mut bad = good;
        bad[48] = 127;
        assert!(decode_text(&bad, 1024, 768).is_none());
        for text in [&b""[..], &b"01234567890123456x"[..], &b"\n"[..]] {
            assert!(encode_text(0, 0, 1, 0, text).is_none());
        }
    }
    fn screen() -> Framebuffer {
        Framebuffer {
            physical: 0xc000_0000,
            bytes: 4 * 1024 * 768,
            width: 1024,
            height: 768,
            stride: 1024,
            format: BGR,
        }
    }
    #[test]
    fn reject_overflow_and_unsupported_device_geometry() {
        let good = screen();
        assert!(good.valid());
        for bad in [
            Framebuffer {
                physical: 1,
                ..good
            },
            Framebuffer { bytes: 1, ..good },
            Framebuffer {
                stride: 1023,
                ..good
            },
            Framebuffer {
                height: u64::MAX,
                ..good
            },
            Framebuffer {
                physical: u64::MAX - 4095,
                ..good
            },
            Framebuffer {
                bytes: MAX_BYTES + 4096,
                ..good
            },
            Framebuffer { format: 2, ..good },
        ] {
            assert!(!bad.valid());
        }
    }
    #[test]
    fn rectangles_and_wire_records_are_bounded() {
        assert!(valid_rect(1024, 768, 1023, 767, 1, 1));
        assert!(!valid_rect(1024, 768, 1023, 767, 2, 1));
        assert!(!valid_rect(1024, 768, u64::MAX, 0, 2, 1));
        assert!(!valid_rect(1024, 768, 0, 0, 0, 1));
        assert_eq!(decode(&screen().user_info()).unwrap()[0], USER_ADDRESS);
        assert!(decode(&[0; 47]).is_none());
        assert_eq!(pixel(RGB, 0x123456), 0x563412);
        assert_eq!(pixel(BGR, 0x123456), 0x123456);
    }
}
