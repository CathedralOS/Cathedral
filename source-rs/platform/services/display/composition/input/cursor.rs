//! Root-owned cursor pixels and bounded damage, independent of app drawing.
use super::*;
impl Router {
    pub fn cursor_bounds(&self) -> [Rect; 2] {
        self.pointers.map(|p| Rect {
            x: p.x,
            y: p.y,
            w: 7.min(self.width - p.x),
            h: 17.min(self.height - p.y),
        })
    }
    pub fn pixel(&self, x: u64, y: u64, clients: &[Client; 3]) -> Option<u32> {
        if !self.enabled {
            return None;
        }
        let mut color = None;
        for (id, p) in self.pointers.iter().enumerate() {
            if id == 1
                && !self.lease.is_some_and(|l| {
                    clients[l.client].visible
                        && clients[l.client].scope.contains(x, y)
                        && clients
                            .iter()
                            .skip(l.client + 1)
                            .all(|client| client.pixel(x, y).is_none())
                })
            {
                continue;
            }
            if x < p.x || y < p.y {
                continue;
            }
            let (dx, dy) = (x - p.x, y - p.y);
            let arrow = dy < 8 && dx <= dy / 2;
            let tag = dx < 5
                && (10..17).contains(&dy)
                && cathedral_bitmap_font::glyph(b'1' + id as u8)[(dy - 10) as usize]
                    & (1 << (4 - dx))
                    != 0;
            if arrow || tag {
                color = Some(if id == 0 { 0xffffff } else { 0xffd166 });
            }
        }
        color
    }
}
