//! Bind a transaction to its authenticated connection and current local view.
use super::*;
pub(super) fn dispatch(
    parent: &mut Parent,
    world: &mut World,
    request: &Request,
    words: wire::Packet,
) -> Result<wire::Packet, Error> {
    let slot = request.client.unwrap();
    let view = world.request(slot, request.incarnation, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
    parent.prune_batches(world);
    if view[0] != 0 || view[1] != words[1] || words[3..] != [0; 5] {
        return Ok(wire::error(abi::DENIED));
    }
    parent.batches[slot].bind(request.incarnation, words[1]);
    let revision = words[2];
    let result = match words[0] {
        wire::PRESENT_BEGIN => parent.batches[slot].begin(revision),
        wire::PRESENT_ABORT => parent.batches[slot].abort(revision),
        wire::PRESENT_COMMIT => {
            match parent.batches[slot].prepare(revision) {
                Err(status) => return Ok(wire::error(status)),
                Ok(false) => return Ok([0; 8]),
                Ok(true) => (),
            }
            let mut prepared = [[0; 8]; wire::PRESENT_LIMIT];
            for (entry, prepared) in parent.batches[slot].entries.iter().zip(&mut prepared) {
                if let Some((words, _)) = entry {
                    match world.preview_surface(slot, request.incarnation, *words) {
                        Ok(p) => *prepared = p,
                        Err(status) => {
                            parent.batches[slot].poison(status);
                            return Ok(wire::error(status));
                        }
                    }
                }
            }
            let mut batch = core::mem::take(&mut parent.batches[slot]);
            let reply = parent.present(&mut batch, &prepared)?;
            if reply[0] == 0 {
                for (words, _) in batch.entries.iter().flatten() {
                    world
                        .surface(slot, request.incarnation, *words)
                        .expect("validated complete batch");
                }
                batch.committed(revision);
            }
            parent.batches[slot] = batch;
            return Ok(reply);
        }
        _ => Err(abi::DENIED),
    };
    Ok(wire::error(result.err().unwrap_or(0)))
}
