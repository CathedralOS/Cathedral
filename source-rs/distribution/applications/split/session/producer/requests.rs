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
                    if let Some(words) = words
                        .filter(|w| (wire::PRODUCER_INFO..=wire::PRODUCER_RELEASE).contains(&w[0]))
                    {
                        let reply = world.request(SLOT, child.data.0.raw(), words);
                        if reply[0] == 0 && words[0] == wire::PRODUCER_POINTER {
                            let (epoch, cursor) = self.cursor.unwrap();
                            let rendered = upstream::call(
                                output.pair,
                                [
                                    wire::SURFACE_CURSOR,
                                    epoch,
                                    cursor,
                                    words[3],
                                    words[4],
                                    7,
                                    12,
                                    0xffd166,
                                ],
                            )?;
                            if matches!(rendered[0], abi::DENIED | abi::BAD_HANDLE) {
                                // Placement can change upstream before its wake
                                // reaches us. End this run; do not crash siblings
                                // or silently recreate authority for the producer.
                                server.reply(&request, &wire::encode(wire::error(abi::DENIED)))?;
                                self.stop(output, world, server)?;
                                return Ok(());
                            }
                            if rendered[0] != 0 {
                                return Err(Error(rendered[0] as i64));
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
