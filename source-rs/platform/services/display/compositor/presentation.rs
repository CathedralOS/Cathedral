//! Hold validated updates and mapped readers; publish all retained tiles before
//! the first rasterization. This is scene atomicity, not a vblank guarantee.
use super::*;
use cathedral_user_runtime::memory::Object;
pub(super) struct Update {
    pub index: usize,
    pub rect: Rect,
    pub value: u64,
    pub cursor: bool,
    pub object: Option<Object>,
}
pub(super) struct Presentation {
    revision: Option<u64>,
    last: u64,
    failed: Option<u64>,
    updates: [Option<Update>; cathedral_platform_contracts::matrix::PRESENT_LIMIT],
}
impl Presentation {
    pub const fn empty() -> Self {
        Self {
            revision: None,
            last: 0,
            failed: None,
            updates: [const { None }; cathedral_platform_contracts::matrix::PRESENT_LIMIT],
        }
    }
}
impl Compositor {
    pub(super) fn publish_or_stage(&mut self, update: Update) -> Result<(), u64> {
        let batch = &mut *self.presentation;
        if batch.revision.is_some() {
            let result = (|| {
                if let Some(status) = batch.failed {
                    return Err(status);
                }
                let mut empty = None;
                for i in 0..batch.updates.len() {
                    match &batch.updates[i] {
                        Some(old) if old.index == update.index => {
                            return Err(abi::INVALID_ARGUMENT);
                        }
                        None if empty.is_none() => empty = Some(i),
                        _ => (),
                    }
                }
                batch.updates[empty.ok_or(abi::NO_MEMORY)?] = Some(update);
                Ok(())
            })();
            if let Err(status) = result {
                batch.failed.get_or_insert(status);
            }
            result
        } else {
            let damage = self.install(update);
            self.redraw(damage);
            Ok(())
        }
    }
    fn install(&mut self, update: Update) -> Rect {
        let damage = if update.object.is_some() {
            self.composition
                .tiles
                .image(update.index, update.rect, update.value)
        } else if update.cursor {
            self.composition
                .tiles
                .cursor(update.index, update.rect, update.value as u32)
        } else {
            self.composition
                .tiles
                .set(update.index, update.rect, update.value as u32)
        };
        self.images[update.index] = update.object;
        damage
    }
    pub(super) fn poison_presentation(&mut self, status: u64) {
        if self.presentation.revision.is_some() {
            self.presentation.failed.get_or_insert(status);
        }
    }
    pub(super) fn presentation_request(&mut self, words: [u64; 6]) -> Result<[u64; 6], u64> {
        let [op, revision, a, b, c, d] = words;
        if revision == 0 || [a, b, c, d] != [0; 4] {
            return Err(abi::INVALID_ARGUMENT);
        }
        let batch = &mut *self.presentation;
        match op {
            wire::MATRIX_BEGIN => {
                if batch.revision == Some(revision) {
                    return Ok([0; 6]);
                }
                if batch.revision.is_some() {
                    return Err(abi::BUSY);
                }
                if revision <= batch.last {
                    return Err(abi::DENIED);
                }
                batch.revision = Some(revision);
            }
            wire::MATRIX_ABORT if batch.revision == Some(revision) => {
                *batch = Presentation {
                    last: batch.last,
                    ..Presentation::empty()
                };
            }
            wire::MATRIX_COMMIT => {
                if batch.revision.is_none() && batch.last == revision {
                    return Ok([0; 6]);
                }
                if batch.revision != Some(revision) {
                    return Err(abi::DENIED);
                }
                if let Some(status) = batch.failed {
                    return Err(status);
                }
                let mut damage = Rect::default();
                for i in 0..cathedral_platform_contracts::matrix::PRESENT_LIMIT {
                    if let Some(update) = self.presentation.updates[i].take() {
                        damage = damage.union(self.install(update));
                    }
                }
                self.presentation.last = revision;
                self.presentation.revision = None;
                self.redraw(damage);
            }
            _ => return Err(abi::DENIED),
        }
        Ok([0; 6])
    }
}
