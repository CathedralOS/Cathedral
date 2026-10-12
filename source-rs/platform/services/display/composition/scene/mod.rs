//! Retained scene lifetime: bind incarnation, validate revision/snapshot, then publish by swap.
//! snapshot owns wire decoding; raster owns hit testing, pixels and damage; hosting owns clips.
//! No device access or IPC authority.
use cathedral_contracts::composition as wire;
use cathedral_contracts::user as abi;
mod geometry;
pub mod hosting;
pub mod labels;
mod raster;
mod snapshot;
pub mod tiles;
pub use geometry::Rect;
#[derive(Clone, Copy, Default, PartialEq, Eq)]
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
    pub fn hit(&self, x: u64, y: u64) -> bool {
        self.visible
            && self.scope.contains(x, y)
            && self.scene.hit(x - self.scope.x, y - self.scope.y)
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
