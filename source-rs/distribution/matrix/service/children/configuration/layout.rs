// Omega plans the layout; apply it under the exact current grant, then publish.
use super::*;
impl Children {
    pub(in crate::service) fn apply_layout(
        &mut self,
        request: &Request,
        words: wire::Packet,
        parent: &mut Parent,
        world: &mut World,
    ) -> Result<wire::Packet, Error> {
        let grant = parent.grant;
        let standing = self
            .standing
            .iter()
            .enumerate()
            .fold(0, |mask, (slot, entry)| {
                mask | (u64::from(entry.is_some_and(|s| s.place.is_some())) << slot)
            });
        let plan = match policy::layout::prepare(
            words,
            world.layout_allowed(request.client.unwrap(), request.incarnation, words[1]),
            self.loss.is_some() || self.pending.iter().any(Option::is_some),
            [grant.width, grant.height],
            standing,
        ) {
            Ok(plan) => plan,
            Err(code) => return Ok(wire::error(code)),
        };
        let rects = match world.layout(
            request.client.unwrap(),
            request.incarnation,
            words[1],
            plan.rectangles.map(rect),
        ) {
            Ok(rects) => rects,
            Err(code) => return Ok(wire::error(code)),
        };
        for (slot, next) in rects.iter().enumerate() {
            if plan.remember & (1 << slot) != 0
                && let Some(standing) = &mut self.standing[slot]
                && let Some((rect, _)) = &mut standing.place
            {
                *rect = *next;
            }
        }
        super::super::super::requests::flush(parent, world)?;
        for slot in 0..CHILDREN {
            super::super::super::requests::replay(parent, world, slot)?;
        }
        Ok([0; 8])
    }
}
