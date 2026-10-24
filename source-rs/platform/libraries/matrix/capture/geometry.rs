//! Validate bounded source-local crops and masks before admitting capture work.
use super::Error;
pub const MAX_MASKS: usize = 8;
pub const MAX_PIXELS: u64 = 32_768;
pub const MAX_SIDE: u64 = 4096;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: u64,
    pub y: u64,
    pub width: u64,
    pub height: u64,
}
impl Rect {
    pub(super) fn valid(self) -> bool {
        self.width != 0
            && self.height != 0
            && self
                .x
                .checked_add(self.width)
                .is_some_and(|v| v <= MAX_SIDE)
            && self
                .y
                .checked_add(self.height)
                .is_some_and(|v| v <= MAX_SIDE)
    }
    pub(super) fn contains(self, x: u64, y: u64) -> bool {
        x >= self.x && y >= self.y && x - self.x < self.width && y - self.y < self.height
    }
}
/// Immutable owner-selected geometry. Changing source extent or masks requires a
/// new grant; an outstanding result must never acquire a replacement mask set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Plan {
    source: [u64; 2],
    crop: Rect,
    masks: [Rect; MAX_MASKS],
    count: usize,
}
impl Plan {
    pub fn new(width: u64, height: u64, crop: Rect, masks: &[Rect]) -> Result<Self, Error> {
        if width == 0
            || height == 0
            || width > MAX_SIDE
            || height > MAX_SIDE
            || !crop.valid()
            || crop.x + crop.width > width
            || crop.y + crop.height > height
            || crop.width * crop.height > MAX_PIXELS
            || masks.len() > MAX_MASKS
            || masks
                .iter()
                .any(|m| !m.valid() || m.x + m.width > width || m.y + m.height > height)
        {
            return Err(Error::Invalid);
        }
        let mut stored = [Rect::default(); MAX_MASKS];
        stored[..masks.len()].copy_from_slice(masks);
        Ok(Self {
            source: [width, height],
            crop,
            masks: stored,
            count: masks.len(),
        })
    }
    pub fn source_extent(&self) -> [u64; 2] {
        self.source
    }
    pub fn crop(&self) -> Rect {
        self.crop
    }
    pub fn bytes(&self) -> usize {
        (self.crop.width * self.crop.height * 4) as usize
    }
    pub(super) fn masked(&self, x: u64, y: u64) -> bool {
        self.masks[..self.count].iter().any(|m| m.contains(x, y))
    }
}
