//! Authenticate the installed child connection before touching its startup offer.
use super::*;
impl Children {
    pub(in crate::service) fn acquire(
        &mut self,
        request: &Request,
        words: wire::Packet,
        server: &mut Server,
    ) -> Result<(), Error> {
        let offer = request
            .client
            .and_then(|slot| self.slots.get_mut(slot))
            .and_then(Option::as_mut)
            .filter(|child| child.data.0.raw() == request.incarnation)
            .and_then(|child| child.services.as_mut());
        match offer {
            Some(offer) => offer.reply(server, request, words),
            None => server.reply(request, &wire::encode(wire::error(abi::DENIED))),
        }
    }
}
