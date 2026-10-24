//! Authenticate the installed child connection before touching its startup offer.
use super::*;
impl Children {
    pub(in crate::service) fn acquire(
        &mut self,
        request: &Request,
        words: wire::Packet,
        parent: &mut Parent,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        let offer = request
            .client
            .and_then(|slot| self.slots.get_mut(slot))
            .and_then(Option::as_mut)
            .filter(|child| child.data.0.raw() == request.incarnation)
            .filter(|child| child.services.is_some());
        match offer {
            Some(child) => {
                if child
                    .services
                    .as_ref()
                    .unwrap()
                    .select(words)
                    .is_ok_and(|part| part >= 3)
                    && child.producer_started == Some(false)
                {
                    // The explicit offer authorizes this stock policy, not a
                    // caller-selected scope. Negotiate before handing off a part.
                    let slot = request.client.unwrap();
                    let view =
                        world.request(slot, request.incarnation, [wire::VIEW, 0, 0, 0, 0, 0, 0, 0]);
                    let result = if view[0] != 0 {
                        Err(Error(view[0] as i64))
                    } else if parent.upstream {
                        parent.grant_input_producer(world, slot, 2, wire::PRODUCER_FOCUS, [0, 0])
                    } else {
                        world
                            .grant_input_producer(slot, 2, wire::PRODUCER_FOCUS, 0, 0)
                            .map_err(|c| Error(c as i64))
                    };
                    if let Err(error) = result {
                        return server.reply(request, &wire::encode(wire::error(error.0 as u64)));
                    }
                    child.producer_started = Some(true);
                }
                child
                    .services
                    .as_mut()
                    .unwrap()
                    .reply(server, request, words)
            }
            None => server.reply(request, &wire::encode(wire::error(abi::DENIED))),
        }
    }
}
