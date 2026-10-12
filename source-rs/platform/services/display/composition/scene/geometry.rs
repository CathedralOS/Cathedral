//! Rectangles shared by snapshot validation, hosting clips and raster damage.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: u64,
    pub y: u64,
    pub w: u64,
    pub h: u64,
}
impl Rect {
    pub fn valid(self) -> bool {
        self.w != 0
            && self.h != 0
            && self.x.checked_add(self.w).is_some()
            && self.y.checked_add(self.h).is_some()
    }
    pub fn contains(self, x: u64, y: u64) -> bool {
        x >= self.x && y >= self.y && x - self.x < self.w && y - self.y < self.h
    }
    pub fn intersect(self, other: Self) -> Self {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        Self {
            x,
            y,
            w: (self.x + self.w).min(other.x + other.w).saturating_sub(x),
            h: (self.y + self.h).min(other.y + other.h).saturating_sub(y),
        }
    }
    pub fn union(self, other: Self) -> Self {
        if !self.valid() {
            return other;
        }
        if !other.valid() {
            return self;
        }
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        Self {
            x,
            y,
            w: (self.x + self.w).max(other.x + other.w) - x,
            h: (self.y + self.h).max(other.y + other.h) - y,
        }
    }
}
