//! A brokered producer endpoint authorizes only its own reserved input grant.
//! Identity comes from the installed endpoint; interaction/storage stay separate.
use super::*;
impl Children {
    pub(in crate::service) fn separate_producer(&self, slot: usize) -> bool {
        self.slots
            .get(slot)
            .and_then(Option::as_ref)
            .is_some_and(|c| c.producer_started.is_some())
    }
    pub(in crate::service) fn producer_request(
        &mut self,
        request: &Request,
        words: Option<wire::Packet>,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        let slot = request.client.unwrap() - PRODUCER_BASE;
        let child = self
            .slots
            .get(slot)
            .and_then(Option::as_ref)
            .filter(|c| c.producer.is_some_and(|p| p.0.raw() == request.incarnation));
        let Some(child) = child else {
            return server.reply(request, &wire::encode(wire::error(abi::DENIED)));
        };
        // Do not fill an endpoint queue that the broker has not transferred yet.
        // Early messages convey no production authority and receive no reply.
        if !child.services.as_ref().is_some_and(|o| o.producer_ready()) {
            return Ok(());
        }
        let response = match words {
            Some(words)
                if wire::producer_operation(words[0]) && child.producer_started == Some(true) =>
            {
                world.request(slot, child.data.0.raw(), words)
            }
            _ => wire::error(abi::DENIED),
        };
        self.notify(world)?;
        server.reply(request, &wire::encode(response))
    }
    pub(in crate::service) fn producer_disconnected(
        &mut self,
        slot: usize,
        incarnation: u64,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        let Some(child) = self
            .slots
            .get_mut(slot)
            .and_then(Option::as_mut)
            .filter(|c| c.producer.is_some_and(|p| p.0.raw() == incarnation))
        else {
            return Ok(());
        };
        world.revoke_input_producer(slot);
        server.remove(PRODUCER_BASE + slot);
        let (input, output) = child.producer.take().unwrap();
        input.close()?;
        output.close()?;
        if let Some(offer) = &mut child.services {
            offer.close_producer()?;
        }
        Ok(())
    }
}
