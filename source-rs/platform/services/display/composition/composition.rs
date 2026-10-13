// Retained state and row composition: background -> scene labels -> apps -> cursors.
// Scene owns snapshots/hosting; input owns pointer routing and keyboard focus.
use input::{Router, focus::Focus};
use scene::{Client, Scene, hosting::Tree, labels::Labels};

pub struct Composition<'a> {
    pub clients: &'a mut [Client; 3],
    pub scratch: &'a mut Scene,
    pub labels: &'a mut Labels,
    pub background: u32,
    pub pointers: Router,
    pub focus: Focus,
    pub tree: Tree,
    pub tiles: &'a mut scene::tiles::Tiles,
}
impl<'a> Composition<'a> {
    pub fn new(
        width: u64,
        height: u64,
        clients: &'a mut [Client; 3],
        scratch: &'a mut Scene,
        labels: &'a mut Labels,
        tiles: &'a mut scene::tiles::Tiles,
    ) -> Self {
        Self {
            clients,
            scratch,
            labels,
            background: 0,
            pointers: Router::new(width, height),
            focus: Focus::default(),
            tree: Tree::default(),
            tiles,
        }
    }
    pub fn paint_row(&self, x: u64, y: u64, row: &mut [u32]) {
        self.paint_images(
            x,
            y,
            row,
            &[&[]; cathedral_contracts::composition::MATRIX_TILES],
        );
    }
    pub fn paint_images(
        &self,
        x: u64,
        y: u64,
        row: &mut [u32],
        images: &[&[u8]; cathedral_contracts::composition::MATRIX_TILES],
    ) {
        row.fill(self.background);
        self.labels.paint_row(x, y, row);
        self.tiles.paint_images(x, y, row, images);
        for client in self.clients.iter() {
            client.paint_row(x, y, row);
        }
        for cursor in self.pointers.cursor_bounds() {
            if y < cursor.y || y - cursor.y >= cursor.h {
                continue;
            }
            for px in x.max(cursor.x)..(x + row.len() as u64).min(cursor.x + cursor.w) {
                if let Some(color) = self.pointers.pixel(px, y, self.clients) {
                    row[(px - x) as usize] = color;
                }
            }
        }
    }
}
