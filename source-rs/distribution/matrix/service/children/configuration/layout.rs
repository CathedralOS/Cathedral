// Apply geometry under the exact current layout grant.
use super::*;
impl Children {
    pub(in crate::service) fn apply_layout(
        &mut self,
        request: &Request,
        words: wire::Packet,
        parent: &mut Parent,
        world: &mut World,
    ) -> Result<wire::Packet, Error> {
        if !world.layout_allowed(request.client.unwrap(), request.incarnation, words[1]) {
            return Ok(wire::error(abi::DENIED));
        }
        if self.loss.is_some() || self.pending.iter().any(Option::is_some) {
            return Ok(wire::error(abi::BUSY));
        }
        let [wire::APPLY_LAYOUT, epoch, margin, bar, gap, 0, 0, 0] = words else {
            return Ok(wire::error(abi::INVALID_ARGUMENT));
        };
        let grant = parent.grant;
        let Some(rects) = policy::rectangles(margin, bar, gap, grant.width, grant.height) else {
            return Ok(wire::error(abi::INVALID_ARGUMENT));
        };
        let pair = rects.map(rect);
        let rects = core::array::from_fn(|i| {
            if i == 0 {
                pair[0]
            } else if i == BACKGROUND {
                Rect {
                    x: 0,
                    y: 0,
                    width: grant.width,
                    height: grant.height,
                }
            } else {
                pair[1]
            }
        });
        let rects = match world.layout(request.client.unwrap(), request.incarnation, epoch, rects) {
            Ok(rects) => rects,
            Err(code) => return Ok(wire::error(code)),
        };
        for (slot, next) in rects.iter().enumerate().take(BACKGROUND).skip(1) {
            if let Some(standing) = &mut self.standing[slot]
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
