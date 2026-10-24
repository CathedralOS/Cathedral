//! Produce only the cropped, masked result. Caller holds stable read custody for
//! every input and keeps destination private until success, then seals it before
//! sharing. This algorithm neither grants capture nor snapshots mutable memory.
use super::{Error, Plan, Rect};
use cathedral_platform_contracts::display::composition as wire;
pub const MAX_LAYERS: usize = wire::MATRIX_TILES;
#[derive(Clone, Copy)]
pub enum Paint<'a> {
    Solid(u32),
    Image { bytes: &'a [u8], stride: u64 },
    Cursor(u32),
}
#[derive(Clone, Copy)]
pub struct Layer<'a> {
    pub rect: Rect,
    pub paint: Paint<'a>,
}
impl Plan {
    /// Bottom-to-top layers in source-local coordinates. Invalid input leaves
    /// destination untouched. Unused output bytes and RGBX padding are zeroed,
    /// including page rounding, so a transferred object contains no stale data.
    pub fn render(&self, layers: &[Layer<'_>], destination: &mut [u8]) -> Result<(), Error> {
        if layers.len() > MAX_LAYERS
            || destination.len() < self.bytes()
            || destination.len() > (super::geometry::MAX_PIXELS * 4) as usize
            || layers.iter().any(|l| {
                !l.rect.valid()
                    || match l.paint {
                        Paint::Image { bytes, stride } => {
                            !wire::valid_image(l.rect.width, l.rect.height, stride, bytes.len())
                        }
                        Paint::Cursor(_) => {
                            l.rect.width > wire::CURSOR_WIDTH || l.rect.height > wire::CURSOR_HEIGHT
                        }
                        Paint::Solid(_) => false,
                    }
            })
        {
            return Err(Error::Invalid);
        }
        destination.fill(0);
        let crop = self.crop();
        for y in 0..crop.height {
            for x in 0..crop.width {
                let (sx, sy) = (crop.x + x, crop.y + y);
                // Masking happens before reading any source pixels. A later layer
                // cannot paint over redaction, including a cursor/overlay.
                if self.masked(sx, sy) {
                    continue;
                }
                let mut color = 0;
                for layer in layers {
                    if !layer.rect.contains(sx, sy) {
                        continue;
                    }
                    let (dx, dy) = (sx - layer.rect.x, sy - layer.rect.y);
                    match layer.paint {
                        Paint::Solid(rgb) => color = rgb,
                        Paint::Image { bytes, stride } => {
                            let at = ((dy * stride + dx) * 4) as usize;
                            color = u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
                        }
                        Paint::Cursor(rgb) => {
                            if (dy < 9 && dx <= dy / 2)
                                || ((9..12).contains(&dy) && (2..4).contains(&dx))
                            {
                                color = rgb;
                            }
                        }
                    }
                }
                let at = ((y * crop.width + x) * 4) as usize;
                destination[at..at + 4].copy_from_slice(&(color & 0xffffff).to_le_bytes());
            }
        }
        Ok(())
    }
}
