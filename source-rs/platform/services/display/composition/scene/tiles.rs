//! Small retained matrix surfaces. Clipping/authority live in the matrix provider;
//! the compositor retains flattened primitives and uses its ordinary scanout.
use super::Rect;
pub struct Tiles {
    entries: [Option<(Rect, Option<u32>, u64)>;
        cathedral_platform_contracts::display::composition::MATRIX_TILES],
}
impl Tiles {
    pub const fn empty() -> Self {
        Self {
            entries: [None; cathedral_platform_contracts::display::composition::MATRIX_TILES],
        }
    }
    pub fn set(&mut self, index: usize, rect: Rect, color: u32) -> Rect {
        self.replace(index, rect, Some(color), 0)
    }
    pub fn image(&mut self, index: usize, rect: Rect, stride: u64) -> Rect {
        // A new backing may have different pixels despite identical geometry.
        let old = self.entries[index].map_or(Rect::default(), |(r, _, _)| r);
        self.replace(index, rect, None, stride);
        old.union(rect)
    }
    fn replace(&mut self, index: usize, rect: Rect, color: Option<u32>, stride: u64) -> Rect {
        let next = rect.valid().then_some((rect, color, stride));
        if self.entries[index] == next {
            return Rect::default();
        }
        let before = self.entries[index].map_or(Rect::default(), |(r, _, _)| r);
        self.entries[index] = next;
        before.union(rect)
    }
    pub fn paint_row(&self, x: u64, y: u64, row: &mut [u32]) {
        self.paint_images(
            x,
            y,
            row,
            &[&[]; cathedral_platform_contracts::display::composition::MATRIX_TILES],
        );
    }
    pub fn paint_images(
        &self,
        x: u64,
        y: u64,
        row: &mut [u32],
        images: &[&[u8]; cathedral_platform_contracts::display::composition::MATRIX_TILES],
    ) {
        // Matrices reserve disjoint contiguous stacking ranges for subtrees.
        for (index, entry) in self.entries.iter().enumerate() {
            let Some((rect, color, stride)) = *entry else {
                continue;
            };
            if color.is_none()
                && !cathedral_platform_contracts::display::composition::valid_image(
                    rect.w,
                    rect.h,
                    stride,
                    images[index].len(),
                )
            {
                continue;
            }
            if y < rect.y || y - rect.y >= rect.h {
                continue;
            }
            for px in x.max(rect.x)..x.saturating_add(row.len() as u64).min(rect.x + rect.w) {
                if let Some(color) = color {
                    row[(px - x) as usize] = color;
                } else {
                    let at = ((y - rect.y) * stride + px - rect.x) as usize * 4;
                    if let Some(bytes) = images[index].get(at..at + 4) {
                        row[(px - x) as usize] =
                            u32::from_le_bytes(bytes.try_into().unwrap()) & 0xffffff;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn image_stride_clipping_z_order_and_invalid_backing() {
        let mut tiles = Tiles::default();
        let rect = Rect {
            x: 2,
            y: 3,
            w: 2,
            h: 2,
        };
        tiles.set(0, rect, 9);
        tiles.image(1, rect, 4);
        let mut pixels = [0u8; 24];
        for (i, pixel) in pixels.chunks_exact_mut(4).enumerate() {
            pixel.copy_from_slice(&(0xff000000u32 | i as u32).to_le_bytes());
        }
        let mut images =
            [&[][..]; cathedral_platform_contracts::display::composition::MATRIX_TILES];
        images[1] = &pixels;
        let mut row = [0; 6];
        tiles.paint_images(0, 4, &mut row, &images);
        assert_eq!(row, [0, 0, 4, 5, 0, 0]);
        tiles.set(
            2,
            Rect {
                x: 3,
                y: 4,
                w: 1,
                h: 1,
            },
            7,
        );
        tiles.paint_images(0, 4, &mut row, &images);
        assert_eq!(row, [0, 0, 4, 7, 0, 0]);
        tiles.image(1, rect, u64::MAX);
        tiles.paint_images(0, 4, &mut row, &images);
        assert_eq!(row, [0, 0, 9, 7, 0, 0]);
        tiles.image(1, rect, 4);
        images[1] = &pixels[..23];
        tiles.paint_images(0, 4, &mut row, &images);
        assert_eq!(row, [0, 0, 9, 7, 0, 0]);
    }
    #[test]
    fn sibling_z_order_and_removal_restore_underlying_subtree() {
        let mut tiles = Tiles::default();
        let rect = Rect {
            x: 2,
            y: 3,
            w: 4,
            h: 2,
        };
        tiles.set(5, rect, 1);
        tiles.set(8, rect, 2);
        let mut row = [0; 8];
        tiles.paint_row(0, 3, &mut row);
        assert_eq!(row, [0, 0, 2, 2, 2, 2, 0, 0]);
        assert_eq!(tiles.set(8, rect, 2), Rect::default());
        assert_eq!(tiles.set(8, Rect::default(), 0), rect);
        tiles.paint_row(0, 3, &mut row);
        assert_eq!(row, [0, 0, 1, 1, 1, 1, 0, 0]);
    }
}

impl Default for Tiles {
    fn default() -> Self {
        Self {
            entries: [None; cathedral_platform_contracts::display::composition::MATRIX_TILES],
        }
    }
}
