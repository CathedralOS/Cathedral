use cathedral_contracts::{display as wire, user as abi};

pub struct Surface {
    address: u64,
    width: u64,
    height: u64,
    stride: u64,
    format: u64,
}
impl Surface {
    pub fn open() -> Self {
        let [address, bytes, width, height, stride, format] =
            cathedral_user_runtime::display::mapping().unwrap();
        assert_eq!(address, wire::USER_ADDRESS);
        assert!(
            wire::Framebuffer {
                physical: 4096,
                bytes,
                width,
                height,
                stride,
                format
            }
            .valid()
        );
        Self {
            address,
            width,
            height,
            stride,
            format,
        }
    }
    pub fn request(&mut self, bytes: &[u8]) -> [u64; 6] {
        let Some([op, x, y, width, height, color]) = wire::decode(bytes) else {
            return [abi::INVALID_ARGUMENT, 0, 0, 0, 0, 0];
        };
        let ok = match op {
            wire::INFO if [x, y, width, height, color] == [0; 5] => {
                return [0, self.width, self.height, self.stride, self.format, 0];
            }
            wire::CLEAR if [x, y, width, height] == [0; 4] && color <= 0xffffff => {
                self.fill(0, 0, self.width, self.height, color);
                true
            }
            wire::RECT
                if color <= 0xffffff
                    && wire::valid_rect(self.width, self.height, x, y, width, height) =>
            {
                self.fill(x, y, width, height, color);
                true
            }
            _ => false,
        };
        [if ok { 0 } else { abi::INVALID_ARGUMENT }, 0, 0, 0, 0, 0]
    }
    fn fill(&mut self, x: u64, y: u64, width: u64, height: u64, color: u64) {
        let pixel = wire::pixel(self.format, color);
        for row in y..y + height {
            for column in x..x + width {
                let pointer = (self.address + (row * self.stride + column) * 4) as *mut u32;
                // SAFETY: Exclusive boot-installed aperture lives for this task.
                // Validated geometry/rectangles bound every aligned 32-bit access.
                unsafe {
                    pointer.write_volatile(pixel);
                }
            }
        }
        // Read back both corners before acknowledging the draw. These accesses
        // also check the provider's stride and channel ordering against its writes.
        for (column, row) in [(x, y), (x + width - 1, y + height - 1)] {
            let pointer = (self.address + (row * self.stride + column) * 4) as *const u32;
            // SAFETY: Same bounded live device mapping as above, not ordinary RAM.
            assert_eq!(unsafe { pointer.read_volatile() } & 0xffffff, pixel);
        }
    }
}
