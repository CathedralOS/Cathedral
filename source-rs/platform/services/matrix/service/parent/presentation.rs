//! Publish the complete clipped batch upstream before changing retained readers.
use super::*;
use cathedral_contracts::composition;
use cathedral_matrix_service::world::presentation::Batch;
use cathedral_user_runtime::memory::Object;
impl Parent {
    pub(in super::super) fn prune_batches(&mut self, world: &World) {
        for (slot, batch) in self.batches.iter_mut().enumerate() {
            if !world.binding_matches(slot, batch.incarnation, batch.epoch) {
                *batch = Batch::default();
            }
        }
    }
    pub(in super::super) fn present(
        &mut self,
        batch: &mut Batch<Option<Object>>,
        prepared: &[wire::Packet; wire::PRESENT_LIMIT],
    ) -> Result<wire::Packet, Error> {
        self.next_revision = self
            .next_revision
            .checked_add(1)
            .ok_or(Error(abi::NO_MEMORY as i64))?;
        let revision = self.next_revision;
        let reply = self.presentation_control(wire::PRESENT_BEGIN, revision)?;
        if reply[0] != 0 {
            return Ok(reply);
        }
        for (entry, words) in batch.entries.iter().zip(prepared) {
            let Some((_, object)) = entry else {
                continue;
            };
            let reply = if let Some(object) = object {
                self.image_call(*words, object)?
            } else {
                self.rectangle(*words)?
            };
            if reply[0] != 0 {
                self.presentation_control(wire::PRESENT_ABORT, revision)?;
                return Ok(reply);
            }
        }
        let reply = self.presentation_control(wire::PRESENT_COMMIT, revision)?;
        if reply[0] != 0 {
            self.presentation_control(wire::PRESENT_ABORT, revision)?;
            return Ok(reply);
        }
        for (entry, words) in batch.entries.iter_mut().zip(prepared) {
            let Some((_, object)) = entry else {
                continue;
            };
            let previous = self
                .images
                .iter()
                .position(|i| i.as_ref().is_some_and(|(key, _)| *key == words[2]));
            if let Some(object) = object.take() {
                let slot = previous
                    .or_else(|| self.images.iter().position(Option::is_none))
                    .expect("one reader per live surface");
                self.images[slot] = Some((words[2], object));
            } else if let Some(slot) = previous {
                self.images[slot] = None;
            }
        }
        Ok(reply)
    }
    fn presentation_control(&self, op: u64, revision: u64) -> Result<wire::Packet, Error> {
        if self.upstream {
            self.call([op, self.grant.epoch, revision, 0, 0, 0, 0, 0])
        } else if self.display.is_some() {
            let op = match op {
                wire::PRESENT_BEGIN => composition::MATRIX_BEGIN,
                wire::PRESENT_COMMIT => composition::MATRIX_COMMIT,
                _ => composition::MATRIX_ABORT,
            };
            let reply = self.display_call([op, revision, 0, 0, 0, 0])?;
            Ok(wire::error(reply[0]))
        } else {
            Ok(wire::error(abi::DENIED))
        }
    }
}
