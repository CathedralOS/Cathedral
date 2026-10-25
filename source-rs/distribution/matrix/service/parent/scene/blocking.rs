//! Publish owner-driven visibility changes through the same atomic protocol as
//! child rendering. Cached images stay intact until the upstream scene commits.
use super::*;
use cathedral_platform_contracts::display::composition;
impl Parent {
    pub(in crate::service) fn switch_scene(
        &mut self,
        world: &mut World,
        previous: Option<usize>,
        next: usize,
    ) -> Result<u64, Error> {
        self.next_revision = self
            .next_revision
            .checked_add(1)
            .ok_or(Error(abi::NO_MEMORY as i64))?;
        let revision = self.next_revision;
        let status = self.scene_control(wire::PRESENT_BEGIN, revision)?;
        if status != 0 {
            return Ok(status);
        }
        let status = self.stage_apps(world, previous, next)?;
        let status = if status == 0 {
            self.scene_control(wire::PRESENT_COMMIT, revision)?
        } else {
            status
        };
        if status != 0 {
            time::storage_sample("scene-commit-refused", &[status]);
            let abort = self.scene_control(wire::PRESENT_ABORT, revision)?;
            if abort != 0 {
                return Err(Error(abort as i64));
            }
        }
        Ok(status)
    }
    fn stage_apps(
        &self,
        world: &mut World,
        previous: Option<usize>,
        next: usize,
    ) -> Result<u64, Error> {
        for slot in 1..BACKGROUND {
            if Some(slot) != previous && slot != next {
                continue;
            }
            for rank in 0..wire::MAX_SURFACES {
                let Some(words) = world.replay(slot, rank) else {
                    continue;
                };
                let response = if words[0] == wire::SURFACE_IMAGE {
                    self.image_call(words, self.images.get(words[2])?)?
                } else {
                    self.rectangle(words)?
                };
                if response[0] != 0 {
                    time::storage_sample(
                        "scene-stage-refused",
                        &[slot as u64, rank as u64, response[0]],
                    );
                    return Ok(response[0]);
                }
            }
        }
        Ok(0)
    }
    fn scene_control(&self, op: u64, revision: u64) -> Result<u64, Error> {
        if self.upstream {
            Ok(self.call([op, self.grant.epoch, revision, 0, 0, 0, 0, 0])?[0])
        } else {
            let op = match op {
                wire::PRESENT_BEGIN => composition::MATRIX_BEGIN,
                wire::PRESENT_COMMIT => composition::MATRIX_COMMIT,
                _ => composition::MATRIX_ABORT,
            };
            Ok(self.display_call([op, revision, 0, 0, 0, 0])?[0])
        }
    }
}
