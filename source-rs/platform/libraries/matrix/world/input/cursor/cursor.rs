// Resolve one pointer's current presentation preference, independently of key
// focus. Owners translate inherited names when forwarding; children never see
// these names or targets. Re-evaluate against geometry even without new motion.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cursor {
    pub channel: u32,
    pub inherited: bool,
    pub cycle: u64,
    pub mode: u64,
}
impl<const N: usize> World<N> {
    pub(in crate::world::input) fn hit_pointer(&self, x: u64, y: u64) -> Option<usize> {
        self.children
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                c.visible
                    && c.grant.valid()
                    && c.rect.is_some_and(|r| {
                        x >= r.x && y >= r.y && x - r.x < c.grant.width && y - r.y < c.grant.height
                    })
            })
            .max_by_key(|(slot, c)| (c.surface_base, *slot))
            .map(|(slot, _)| slot)
    }
    /// Owner observation, not a child-visible registry. One row per bounded
    /// routing slot; replacement/cancellation must invalidate forwarding caches.
    pub fn cursor(&self, index: usize) -> Option<Cursor> {
        let stream = self.input.streams.get(index)?;
        if !self.parent.valid() || !stream.id.valid() || stream.retiring || stream.cursor_cycle == 0
        {
            return None;
        }
        let (x, y) = stream.point?;
        let target = if stream.blocked {
            None
        } else if stream.buttons != 0 {
            stream
                .capture
                .filter(|slot| self.children[*slot].grant.valid())
        } else {
            self.hit_pointer(x, y)
        };
        Some(Cursor {
            channel: stream.id.value,
            inherited: stream.id.inherited,
            cycle: stream.cursor_cycle,
            mode: target
                .and_then(|slot| self.children[slot].cursor[index])
                .map(u64::from)
                .unwrap_or(wire::CURSOR_DEFAULT),
        })
    }
    /// Owner lookup for a cursor renderer paired with an admitted producer.
    pub fn producer_cursor(&self, slot: usize, ordinal: usize) -> Option<Cursor> {
        let producer = self.producers.get(slot)?;
        let name = producer.cursor_channel(ordinal)?;
        let index = self
            .input
            .streams
            .iter()
            .position(|s| s.id == Name::local(name))?;
        self.cursor(index)
    }
}
