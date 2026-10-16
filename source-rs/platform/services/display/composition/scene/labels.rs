//! Bounded retained control labels. The distribution supplies text and appearance.
use crate::composition::scene::Rect;
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::display;
#[derive(Clone, Copy)]
struct Label {
    rect: Rect,
    scale: u64,
    color: u32,
    text: [u8; 16],
    len: usize,
}
pub struct Labels {
    entries: [Option<Label>; 32],
}
impl Labels {
    pub const fn empty() -> Self {
        Self {
            entries: [None; 32],
        }
    }
    /// Labels at the same origin replace each other; invalid/full updates are inert.
    pub fn set(&mut self, bytes: &[u8], width: u64, height: u64) -> Result<Rect, u64> {
        let t = display::decode_text(bytes, width, height).ok_or(abi::INVALID_ARGUMENT)?;
        let slot = self
            .entries
            .iter()
            .position(|e| e.is_some_and(|l| l.rect.x == t.x && l.rect.y == t.y))
            .or_else(|| self.entries.iter().position(Option::is_none))
            .ok_or(abi::BUSY)?;
        let rect = Rect {
            x: t.x,
            y: t.y,
            w: t.bytes.len() as u64 * 6 * t.scale,
            h: 7 * t.scale,
        };
        let damage = self.entries[slot].map_or(rect, |old| old.rect.union(rect));
        let mut text = [0; 16];
        text[..t.bytes.len()].copy_from_slice(t.bytes);
        self.entries[slot] = Some(Label {
            rect,
            scale: t.scale,
            color: t.color as u32,
            text,
            len: t.bytes.len(),
        });
        Ok(damage)
    }
    pub fn paint_row(&self, x: u64, y: u64, row: &mut [u32]) {
        for label in self.entries.iter().flatten() {
            let r = label.rect;
            if y < r.y || y - r.y >= r.h {
                continue;
            }
            for (i, &ch) in label.text[..label.len].iter().enumerate() {
                let bits = cathedral_bitmap_font::glyph(ch)[((y - r.y) / label.scale) as usize];
                for col in 0..5 {
                    if bits & (1 << (4 - col)) == 0 {
                        continue;
                    }
                    let px = r.x + (i as u64 * 6 + col) * label.scale;
                    let end = (px + label.scale).min(x + row.len() as u64);
                    for px in px.max(x)..end {
                        row[(px - x) as usize] = label.color;
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
    fn replacement_damages_old_extent_and_capacity_never_evicts_other_labels() {
        let mut labels = Labels::empty();
        let packet = |x, y, text: &[u8]| display::encode_text(x, y, 2, 0xabcdef, text).unwrap();
        let old = labels.set(&packet(10, 10, b"AAAA"), 1024, 768).unwrap();
        assert_eq!(labels.set(&packet(10, 10, b"A"), 1024, 768), Ok(old));
        let mut row = [0; 100];
        labels.paint_row(0, 10, &mut row);
        assert_eq!(&row[12..18], &[0xabcdef; 6]);
        assert!(row[22..].iter().all(|&pixel| pixel == 0));
        for i in 1..32 {
            labels
                .set(&packet(10, 10 + i * 20, b"A"), 1024, 768)
                .unwrap();
        }
        assert_eq!(
            labels.set(&packet(400, 10, b"B"), 1024, 768),
            Err(abi::BUSY)
        );
        assert!(labels.set(&packet(10, 10, b"B"), 1024, 768).is_ok());
        assert_eq!(
            labels.set(&packet(1020, 10, b"B"), 1024, 768),
            Err(abi::INVALID_ARGUMENT)
        );
    }
}
