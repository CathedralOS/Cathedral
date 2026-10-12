//! Small retained matrix surfaces. Clipping/authority live in the matrix provider;
//! the compositor retains flattened primitives and uses its ordinary scanout.
use super::Rect;
#[derive(Default)]
pub struct Tiles {
    entries: [Option<(Rect, u32)>; cathedral_contracts::composition::MATRIX_TILES],
}
impl Tiles {
    pub const fn empty() -> Self {
        Self {
            entries: [None; cathedral_contracts::composition::MATRIX_TILES],
        }
    }
    pub fn set(&mut self, index: usize, rect: Rect, color: u32) -> Rect {
        let next = rect.valid().then_some((rect, color));
        if self.entries[index] == next {
            return Rect::default();
        }
        let before = self.entries[index].map_or(Rect::default(), |(r, _)| r);
        self.entries[index] = next;
        before.union(rect)
    }
    pub fn paint_row(&self, x: u64, y: u64, row: &mut [u32]) {
        // Matrices reserve disjoint contiguous stacking ranges for subtrees.
        for entry in &self.entries {
            let Some((rect, color)) = *entry else {
                continue;
            };
            if y < rect.y || y - rect.y >= rect.h {
                continue;
            }
            for px in x.max(rect.x)..(x + row.len() as u64).min(rect.x + rect.w) {
                row[(px - x) as usize] = color;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
