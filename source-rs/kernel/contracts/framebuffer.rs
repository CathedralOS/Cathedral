//! Experimental framebuffer aperture and DISPLAY_INFO syscall record.
//! Geometry alone conveys no device custody or authority.
// Keep the device aperture above every guarded page-object mapping slot.
pub const USER_ADDRESS: u64 = 0x0000_0080_0400_0000;
const _: () = assert!(crate::memory::address(crate::memory::MAX_MAPPING_SLOTS) <= USER_ADDRESS);
pub const MAX_BYTES: u64 = 16 * 1024 * 1024;
pub const INFO_BYTES: usize = 48;
pub const RGB: u64 = 0;
pub const BGR: u64 = 1;
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
pub fn encode(words: [u64; 6]) -> [u8; INFO_BYTES] {
    let mut bytes = [0; INFO_BYTES];
    for (word, output) in words.into_iter().zip(bytes.chunks_exact_mut(8)) {
        output.copy_from_slice(&word.to_le_bytes());
    }
    bytes
}
pub fn decode(bytes: &[u8]) -> Option<[u64; 6]> {
    if bytes.len() != INFO_BYTES {
        return None;
    }
    let mut words = [0; 6];
    for (word, input) in words.iter_mut().zip(bytes.chunks_exact(8)) {
        *word = u64::from_le_bytes(input.try_into().ok()?);
    }
    Some(words)
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
    fn mapping_record_and_native_pixel_encoding() {
        assert_eq!(
            decode(&screen().user_info()).unwrap(),
            [USER_ADDRESS, 4 * 1024 * 768, 1024, 768, 1024, BGR]
        );
        assert!(decode(&[0; 47]).is_none());
        assert_eq!(pixel(RGB, 0x123456), 0x563412);
        assert_eq!(pixel(BGR, 0x123456), 0x123456);
    }
}
