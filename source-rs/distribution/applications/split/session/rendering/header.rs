//! Compact owner controls; app content stays in its assigned columns.
use super::*;
impl Output {
    pub fn header(&mut self, dirty: bool) -> Result<(), Error> {
        if self.header == 0 {
            return Ok(());
        }
        let width = (self.grant.width as usize).min(512);
        let mut pixels = Private::allocate((width * 28 * 4).div_ceil(4096))?;
        for p in pixels.bytes_mut()[..width * 28 * 4].chunks_exact_mut(4) {
            p.copy_from_slice(&0x202d43u32.to_le_bytes());
        }
        let base = width.saturating_sub(256);
        for (left, text) in [
            (
                8,
                if width >= 512 && !self.nested && self.failure.is_none() && !dirty {
                    if self.capturing {
                        b"CLEAR".as_slice()
                    } else {
                        b"CAPTURE".as_slice()
                    }
                } else {
                    b""
                },
            ),
            (
                width.saturating_sub(384) + 8,
                if width >= 384 && !self.nested && self.failure.is_none() && !dirty {
                    if self.hidden_right {
                        b"SHOW RIGHT".as_slice()
                    } else {
                        b"HIDE RIGHT".as_slice()
                    }
                } else {
                    b""
                },
            ),
            (
                base + 8,
                if let Some(message) = self.failure {
                    message
                } else if dirty {
                    b"UNSAVED NOTES".as_slice()
                } else if self.nested {
                    b"NESTED".as_slice()
                } else {
                    b"SPLIT"
                },
            ),
            (
                base + 100,
                if !self.nested && !dirty && self.failure.is_none() {
                    b"DEEPER".as_slice()
                } else {
                    b""
                },
            ),
            (
                base + 180,
                if !self.nested && self.failure.is_none() {
                    if self.automatic {
                        b"STOP".as_slice()
                    } else {
                        b"AUTO".as_slice()
                    }
                } else {
                    b""
                },
            ),
            (width.saturating_sub(20), b"X"),
        ] {
            for (i, ch) in text.iter().enumerate() {
                for (y, bits) in cathedral_bitmap_font::glyph(*ch).into_iter().enumerate() {
                    for x in 0..5 {
                        if bits & (1 << (4 - x)) != 0 {
                            for dy in 0..2 {
                                for dx in 0..2 {
                                    let px = left + i * 12 + x * 2 + dx;
                                    let py = 6 + y * 2 + dy;
                                    if px < width {
                                        let at = (py * width + px) * 4;
                                        pixels.bytes_mut()[at..at + 4]
                                            .copy_from_slice(&0xffffffu32.to_le_bytes());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        let mut object = pixels.freeze()?;
        object.unmap()?;
        let reply = upstream::exchange(
            self.pair,
            [
                wire::SURFACE_IMAGE,
                self.grant.epoch,
                self.header,
                self.grant.width - width as u64,
                0,
                width as u64,
                28,
                width as u64,
            ],
            Some(&object),
        )?;
        if reply[0] != 0 {
            return Err(Error(reply[0] as i64));
        }
        Ok(())
    }
}
