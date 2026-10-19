//! Bind a transaction to its authenticated connection and current local view.
use super::*;
pub(super) fn dispatch(
    parent: &mut Parent,
    world: &mut World,
    request: &Request,
    mut words: wire::Packet,
) -> Result<Option<wire::Packet>, Error> {
    let slot = request.client.unwrap();
    let view = world.request(slot, request.incarnation, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
    parent.prune_batches(world);
    if view[0] != 0
        || view[1] != words[1]
        || (words[0] != wire::PRESENT_RECTS && words[3..] != [0; 5])
    {
        return Ok(Some(wire::error(abi::DENIED)));
    }
    parent.batches[slot].bind(request.incarnation, words[1]);
    let revision = words[2];
    if words[0] == wire::PRESENT_RECTS {
        if !parent.has_output() {
            return Ok(Some(wire::error(abi::DENIED)));
        }
        let Some((updates, count)) = wire::rectangle_updates(words) else {
            return Ok(Some(wire::error(abi::INVALID_ARGUMENT)));
        };
        if parent.batches[slot].prepare(revision) == Ok(false) {
            return Ok(Some([0; 8]));
        }
        if parent.batches[slot].active() {
            return Ok(Some(wire::error(abi::BUSY)));
        }
        // Validate every command before installing any stage or changing history.
        for update in &updates[..count] {
            if let Err(status) = world.preview_surface(slot, request.incarnation, *update) {
                return Ok(Some(wire::error(status)));
            }
        }
        if let Err(status) = parent.batches[slot].begin(revision) {
            return Ok(Some(wire::error(status)));
        }
        for update in &updates[..count] {
            parent.batches[slot]
                .stage(*update, None)
                .expect("validated distinct rectangles");
        }
        words[0] = wire::PRESENT_COMMIT;
    }
    let result = match words[0] {
        wire::PRESENT_BEGIN => parent.batches[slot].begin(revision),
        wire::PRESENT_ABORT => parent.batches[slot].abort(revision),
        wire::PRESENT_COMMIT => {
            if !parent.has_output() {
                return Ok(Some(wire::error(abi::DENIED)));
            }
            match parent.batches[slot].prepare(revision) {
                Err(status) => return Ok(Some(wire::error(status))),
                Ok(false) => return Ok(Some([0; 8])),
                Ok(true) => (),
            }
            let mut prepared = [[0; 8]; wire::PRESENT_LIMIT];
            for (entry, prepared) in parent.batches[slot].entries.iter().zip(&mut prepared) {
                if let Some((words, _)) = entry {
                    match world.preview_surface(slot, request.incarnation, *words) {
                        Ok(p) => *prepared = p,
                        Err(status) => {
                            parent.batches[slot].poison(status);
                            return Ok(Some(wire::error(status)));
                        }
                    }
                }
            }
            parent.present(*request, revision, prepared)?;
            return Ok(None);
        }
        _ => Err(abi::DENIED),
    };
    Ok(Some(wire::error(result.err().unwrap_or(0))))
}
