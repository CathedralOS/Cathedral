//! Paint bounded spans in scene order; compare snapshots to bound repaint damage.
use super::*;
impl Scene {
    pub fn hit(&self, x: u64, y: u64) -> bool {
        let regions = self.draws[..self.count]
            .iter()
            .filter(|draw| draw.kind == wire::INPUT);
        if regions.clone().next().is_none() {
            return self.pixel(x, y).is_some();
        }
        regions.into_iter().any(|draw| draw.clip.contains(x, y))
    }
    pub fn pixel(&self, x: u64, y: u64) -> Option<u32> {
        let mut color = None;
        for draw in &self.draws[..self.count] {
            if !draw.clip.contains(x, y) {
                continue;
            }
            let dx = x - draw.rect.x;
            let dy = y - draw.rect.y;
            match draw.kind {
                wire::RECT => color = Some(draw.value as u32),
                wire::IMAGE => color = Some(self.pixels[(dy * 16 + dx) as usize]),
                wire::TEXT => {
                    let scale = draw.rect.h / 7;
                    let (dx, dy) = (dx / scale, dy / scale);
                    let ch = draw.value.to_le_bytes()[(dx / 6) as usize];
                    if dx % 6 < 5
                        && cathedral_bitmap_font::glyph(ch)[dy as usize] & (1 << (4 - dx % 6)) != 0
                    {
                        color = Some(0xffffff);
                    }
                }
                _ => (),
            }
        }
        color
    }
    pub fn damage(&self, old: &Self) -> Rect {
        let mut damage = Rect::default();
        let image_changed = self.pixels != old.pixels;
        for index in 0..self.count.max(old.count) {
            let a = (index < self.count).then_some(self.draws[index]);
            let b = (index < old.count).then_some(old.draws[index]);
            if a != b || (image_changed && a.is_some_and(|d| d.kind == wire::IMAGE)) {
                for draw in [a, b].into_iter().flatten() {
                    if matches!(draw.kind, wire::RECT | wire::TEXT | wire::IMAGE) {
                        damage = damage.union(draw.clip);
                    }
                }
            }
        }
        damage
    }
    fn paint_row(&self, x: u64, y: u64, row: &mut [u32]) {
        for draw in &self.draws[..self.count] {
            if y < draw.clip.y || y - draw.clip.y >= draw.clip.h {
                continue;
            }
            let start = x.max(draw.clip.x);
            let end = (x + row.len() as u64).min(draw.clip.x + draw.clip.w);
            if start >= end {
                continue;
            }
            let target = &mut row[(start - x) as usize..(end - x) as usize];
            match draw.kind {
                wire::RECT => target.fill(draw.value as u32),
                wire::IMAGE => {
                    let at = ((y - draw.rect.y) * 16 + start - draw.rect.x) as usize;
                    target.copy_from_slice(&self.pixels[at..at + target.len()]);
                }
                wire::TEXT => {
                    let scale = draw.rect.h / 7;
                    let dy = (y - draw.rect.y) / scale;
                    for (i, pixel) in target.iter_mut().enumerate() {
                        let dx = (start + i as u64 - draw.rect.x) / scale;
                        let ch = draw.value.to_le_bytes()[(dx / 6) as usize];
                        if dx % 6 < 5
                            && cathedral_bitmap_font::glyph(ch)[dy as usize] & (1 << (4 - dx % 6))
                                != 0
                        {
                            *pixel = 0xffffff;
                        }
                    }
                }
                _ => (),
            }
        }
    }
}
impl Client {
    pub fn paint_row(&self, x: u64, y: u64, row: &mut [u32]) {
        if !self.visible || y < self.scope.y || y - self.scope.y >= self.scope.h {
            return;
        }
        let start = x.max(self.scope.x);
        let end = (x + row.len() as u64).min(self.scope.x + self.scope.w);
        if start < end {
            self.scene.paint_row(
                start - self.scope.x,
                y - self.scope.y,
                &mut row[(start - x) as usize..(end - x) as usize],
            );
        }
    }
}
