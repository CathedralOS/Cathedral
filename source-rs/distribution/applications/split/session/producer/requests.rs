//! Authenticate each installed endpoint. The broker cannot inject; the producer
//! pair cannot acquire services, draw, access storage or name another participant.
use super::*;
impl Producer {
    pub fn dispatch(
        &mut self,
        event: Event,
        output: &mut Output,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        match event {
            Event::Disconnected {
                client,
                incarnation,
            } => {
                if self.child.as_ref().is_some_and(|c| {
                    let pair = if client == BROKER {
                        c.data
                    } else {
                        c.producer.unwrap()
                    };
                    pair.0.raw() == incarnation
                }) {
                    self.stop(output, world, server)?;
                    output.header(false)?;
                }
            }
            Event::Memory(request, _object) => {
                // A transferred first half may send any packet kind. Do not
                // fill the reply queue while its receiver remains in the offer.
                if request.client == Some(INPUT)
                    && self
                        .child
                        .as_ref()
                        .is_some_and(|c| !c.services.as_ref().unwrap().producer_ready())
                {
                    return Ok(());
                }
                server.reply(&request, &wire::encode(wire::error(abi::DENIED)))?
            }
            Event::Request(request) => {
                let Some(child) = &mut self.child else {
                    return server.reply(&request, &wire::encode(wire::error(abi::DENIED)));
                };
                let words = wire::decode(&request.bytes[..request.len]);
                if request.client == Some(BROKER) && request.incarnation == child.data.0.raw() {
                    if let Some(words) = words.filter(|w| w[0] == services::ACQUIRE) {
                        return child
                            .services
                            .as_mut()
                            .unwrap()
                            .reply(server, &request, words);
                    }
                } else if request.client == Some(INPUT)
                    && request.incarnation == child.producer.unwrap().0.raw()
                {
                    // Do not queue replies before both service halves transferred.
                    if !child.services.as_ref().unwrap().producer_ready() {
                        return Ok(());
                    }
                    if let Some(words) = words.filter(|w| wire::producer_operation(w[0])) {
                        let reply = world.request(SLOT, child.data.0.raw(), words);
                        if reply[0] == 0 && words[0] == wire::PRODUCER_POINTER {
                            self.point = Some(((words[2] - 1) as usize, words[3], words[4]));
                            if let Err(error) = self.render_cursor(output, world) {
                                if matches!(error.0 as u64, abi::DENIED | abi::BAD_HANDLE) {
                                    // Placement may retire before its wake arrives.
                                    server
                                        .reply(&request, &wire::encode(wire::error(abi::DENIED)))?;
                                    self.stop(output, world, server)?;
                                    return Ok(());
                                }
                                return Err(error);
                            }
                        }
                        return server.reply(&request, &wire::encode(reply));
                    }
                }
                server.reply(&request, &wire::encode(wire::error(abi::DENIED)))?;
            }
        }
        Ok(())
    }
}
