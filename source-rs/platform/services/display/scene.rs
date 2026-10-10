//! Fixed-budget snapshots and recursive clips. No device access or IPC authority.
use cathedral_contracts::composition as wire;
use cathedral_contracts::user as abi;
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
    fn intersect(self, other: Self) -> Self {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        Self {
            x,
            y,
            w: (self.x + self.w).min(other.x + other.w).saturating_sub(x),
            h: (self.y + self.h).min(other.y + other.h).saturating_sub(y),
        }
    }
}
#[derive(Clone, Copy, Default)]
struct Draw {
    kind: u64,
    rect: Rect,
    clip: Rect,
    value: u64,
}
#[derive(Clone)]
pub struct Scene {
    draws: [Draw; wire::MAX_NODES],
    count: usize,
    pixels: [u32; wire::IMAGE_PIXELS],
}
impl Scene {
    pub const fn empty() -> Self {
        Self {
            draws: [Draw {
                kind: 0,
                rect: Rect {
                    x: 0,
                    y: 0,
                    w: 0,
                    h: 0,
                },
                clip: Rect {
                    x: 0,
                    y: 0,
                    w: 0,
                    h: 0,
                },
                value: 0,
            }; wire::MAX_NODES],
            count: 0,
            pixels: [0; wire::IMAGE_PIXELS],
        }
    }
    pub fn decode(&mut self, bytes: &[u8]) -> Result<(), u64> {
        if bytes.len() < wire::SNAPSHOT_BYTES {
            return Err(abi::INVALID_ARGUMENT);
        }
        let word = |at| u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap());
        if word(0) != wire::MAGIC || word(8) > wire::MAX_NODES as u64 {
            return Err(abi::INVALID_ARGUMENT);
        }
        let scene = self;
        scene.count = word(8) as usize;
        for index in 0..scene.count {
            let base = 16 + index * 56;
            let [parent, kind, x, y, w, h, value] = core::array::from_fn(|i| word(base + i * 8));
            let (ox, oy, clip) = if parent == wire::ROOT {
                (
                    0,
                    0,
                    Rect {
                        x: 0,
                        y: 0,
                        w: 256,
                        h: 256,
                    },
                )
            } else {
                let ancestor = scene
                    .draws
                    .get(parent as usize)
                    .filter(|_| parent < index as u64)
                    .ok_or(abi::INVALID_ARGUMENT)?;
                if ancestor.kind != wire::GROUP {
                    return Err(abi::INVALID_ARGUMENT);
                }
                (ancestor.rect.x, ancestor.rect.y, ancestor.clip)
            };
            let rect = Rect {
                x: ox.checked_add(x).ok_or(abi::INVALID_ARGUMENT)?,
                y: oy.checked_add(y).ok_or(abi::INVALID_ARGUMENT)?,
                w,
                h,
            };
            if !rect.valid() {
                return Err(abi::INVALID_ARGUMENT);
            }
            match kind {
                wire::GROUP if value == 0 => (),
                wire::RECT if value <= 0xffffff => (),
                wire::TEXT => {
                    let text = value.to_le_bytes();
                    let len = text.iter().position(|&b| b == 0).unwrap_or(8);
                    if len == 0
                        || text[..len].iter().any(|b| !(32..=126).contains(b))
                        || text[len..].iter().any(|&b| b != 0)
                        || w != len as u64 * 6
                        || h != 7
                    {
                        return Err(abi::INVALID_ARGUMENT);
                    }
                }
                wire::IMAGE if w == 16 && h == 16 && value == 0 => (),
                _ => return Err(abi::INVALID_ARGUMENT),
            }
            scene.draws[index] = Draw {
                kind,
                rect,
                clip: rect.intersect(clip),
                value,
            };
        }
        // Canonical unused nodes prevent hidden trailing commands.
        if bytes[16 + scene.count * 56..16 + wire::MAX_NODES * 56]
            .iter()
            .any(|&b| b != 0)
        {
            return Err(abi::INVALID_ARGUMENT);
        }
        for (pixel, chunk) in scene
            .pixels
            .iter_mut()
            .zip(bytes[16 + wire::MAX_NODES * 56..wire::SNAPSHOT_BYTES].chunks_exact(4))
        {
            *pixel = u32::from_le_bytes(chunk.try_into().unwrap());
            if *pixel > 0xffffff {
                return Err(abi::INVALID_ARGUMENT);
            }
        }
        Ok(())
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
}
pub struct Client {
    pub incarnation: u64,
    pub revision: u64,
    pub scope: Rect,
    pub scene: Scene,
    pub visible: bool,
}
impl Client {
    pub const fn empty() -> Self {
        Self {
            incarnation: 0,
            revision: 0,
            scope: Rect {
                x: 0,
                y: 0,
                w: 0,
                h: 0,
            },
            scene: Scene::empty(),
            visible: false,
        }
    }

    pub fn bind(&mut self, incarnation: u64) {
        if self.incarnation != incarnation {
            self.visible = false;
            self.revision = 0;
            self.incarnation = incarnation;
        }
    }
    pub fn disconnect(&mut self, incarnation: u64) {
        if self.incarnation == incarnation {
            self.visible = false;
            self.revision = 0;
            self.incarnation = 0;
        }
    }
    pub fn submit(
        &mut self,
        incarnation: u64,
        revision: u64,
        bytes: &[u8],
        scratch: &mut Scene,
    ) -> Result<(), u64> {
        self.bind(incarnation);
        if revision == 0 || revision <= self.revision {
            return Err(abi::BAD_HANDLE);
        }
        scratch.decode(bytes)?;
        core::mem::swap(&mut self.scene, scratch);
        self.visible = true;
        self.revision = revision;
        Ok(())
    }
    pub fn pixel(&self, x: u64, y: u64) -> Option<u32> {
        if !self.visible || !self.scope.contains(x, y) {
            return None;
        }
        self.scene.pixel(x - self.scope.x, y - self.scope.y)
    }
}
impl Default for Client {
    fn default() -> Self {
        Self::empty()
    }
}
#[cfg(test)]
mod tests;
