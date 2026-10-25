//! Hiding removes display demand, retaining the producer's private input route.
//! Replay uses retained sealed images. This lab does not promise an atomic scene
//! transition through every parent: each surface is forwarded independently.
use super::*;
impl Output {
    pub fn capture_hit(&self, x: u64, y: u64) -> bool {
        let left = self.grant.width.saturating_sub(512);
        self.grant.width >= 512
            && !self.nested
            && self.failure.is_none()
            && y < 28
            && (left..left + 128).contains(&x)
    }

    pub fn visibility_hit(&self, x: u64, y: u64) -> bool {
        let left = self.grant.width.saturating_sub(384);
        self.grant.width >= 384
            && !self.nested
            && self.failure.is_none()
            && y < 28
            && (left..left + 128).contains(&x)
    }
    pub fn toggle_right(&mut self, world: &mut World) -> Result<(), Error> {
        self.hidden_right = !self.hidden_right;
        world
            .visible(1, !self.hidden_right)
            .map_err(|c| Error(c as i64))?;
        for rank in 0..4 {
            let Some(words) = world.replay(1, rank) else {
                continue;
            };
            let image = if words[0] == wire::SURFACE_IMAGE {
                Some(self.images.get(words[2])?)
            } else {
                None
            };
            let reply = self.services.request_interaction(words, image)?;
            if reply[0] != 0 {
                return Err(Error(reply[0] as i64));
            }
        }
        self.header(false)
    }
}
