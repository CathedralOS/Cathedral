//! Separate broker/presentation and capture endpoints. Attachments sent by the
//! consumer cannot become source pixels or masks. No request supplies source IDs.
use super::*;
use cathedral_user_runtime::server::Request;
impl Capture {
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
                    incarnation
                        == if client == BROKER {
                            c.data.0.raw()
                        } else {
                            c.capture.unwrap().0.raw()
                        }
                }) {
                    self.stop(output, world, server)?;
                    output.header(false)?;
                }
            }
            Event::Request(mut request) => {
                let Some(child) = &mut self.child else {
                    return Ok(());
                };
                if request.client == Some(BROKER) && request.incarnation == child.data.0.raw() {
                    let Some(words) = wire::decode(&request.bytes[..request.len]) else {
                        return server
                            .reply(&request, &wire::encode(wire::error(abi::INVALID_ARGUMENT)));
                    };
                    if words[0] == services::ACQUIRE {
                        return child
                            .services
                            .as_mut()
                            .unwrap()
                            .reply(server, &request, words);
                    }
                    request.client = Some(SLOT);
                    let reply = if matches!(
                        words[0],
                        wire::SURFACE_CREATE | wire::PAINT | wire::ERASE | wire::SURFACE_CURSOR
                    ) {
                        output.surface(world, &request, words)?
                    } else {
                        world.request(SLOT, request.incarnation, words)
                    };
                    // World uses the participant slot; transport correlation must
                    // retain the actual broker slot that received this request.
                    request.client = Some(BROKER);
                    server.reply(&request, &wire::encode(reply))?;
                } else if request.client == Some(ENDPOINT)
                    && request.incarnation == child.capture.unwrap().0.raw()
                    && child.services.as_ref().unwrap().capture_ready()
                {
                    self.request(request, world, server)?;
                }
            }
            Event::Memory(mut request, object) => {
                let Some(child) = &self.child else {
                    return Ok(());
                };
                if request.client == Some(BROKER) && request.incarnation == child.data.0.raw() {
                    request.client = Some(SLOT);
                    let reply = output.image(world, &request, object)?;
                    // World uses the participant slot; transport correlation must
                    // retain the actual broker slot that received this request.
                    request.client = Some(BROKER);
                    server.reply(&request, &wire::encode(reply))?;
                } else if child.services.as_ref().unwrap().capture_ready() {
                    server.reply(&request, &wire::encode(wire::error(abi::DENIED)))?;
                }
            }
        }
        Ok(())
    }
    fn request(
        &mut self,
        request: Request,
        world: &mut World,
        server: &mut Server,
    ) -> Result<(), Error> {
        let reply = match wire::decode(&request.bytes[..request.len]) {
            Some([wire_capture::INFO, 0, 0, 0, 0, 0, 0, 0]) => [0, 200, 104, 200, 0, 0, 0, 0],
            Some([wire_capture::BEGIN, 0, 0, 0, 0, 0, 0, 0]) => {
                let now = time::now()?;
                match self.begin(world, now) {
                    Ok(t) => [0, t, 0, 0, 0, 0, 0, 0],
                    Err(e) => wire::error(e.0 as u64),
                }
            }
            Some([wire_capture::TAKE, ticket, 0, 0, 0, 0, 0, 0]) => {
                if let Some((t, code)) = self.failure
                    && t == ticket
                {
                    wire::error(code)
                } else {
                    match self.lease.ready(self.receiver.unwrap(), ticket) {
                        Ok(object) => {
                            match server.reply_memory(
                                &request,
                                &wire::encode([0, ticket, 200, 104, 200, 0, 0, 0]),
                                object,
                            ) {
                                Ok(()) => {
                                    self.lease.release(self.receiver.unwrap(), ticket).unwrap();
                                }
                                Err(Error(c)) if c == abi::WOULD_BLOCK as i64 => (),
                                Err(Error(c)) if c == abi::PEER_CLOSED as i64 => (),
                                Err(e) => {
                                    server
                                        .reply(&request, &wire::encode(wire::error(e.0 as u64)))?;
                                }
                            }
                            return Ok(());
                        }
                        Err(cathedral_matrix::capture::Error::Pending) => {
                            wire::error(wire_capture::PENDING)
                        }
                        Err(cathedral_matrix::capture::Error::Expired) => {
                            wire::error(wire_capture::EXPIRED)
                        }
                        Err(_) => wire::error(abi::DENIED),
                    }
                }
            }
            Some([wire_capture::CANCEL, ticket, 0, 0, 0, 0, 0, 0]) => {
                if self.lease.release(self.receiver.unwrap(), ticket).is_ok() {
                    self.release_frame(world);
                    [0; 8]
                } else {
                    wire::error(abi::DENIED)
                }
            }
            _ => wire::error(abi::DENIED),
        };
        server.reply(&request, &wire::encode(reply))
    }
    fn begin(&mut self, world: &mut World, now: u64) -> Result<u64, Error> {
        if self.pending.is_some() {
            return Err(Error(abi::BUSY as i64));
        }
        // Allocation precedes demand; a refused capture cannot wake an unfunded producer.
        let target = Private::allocate(self.lease.plan().unwrap().bytes().div_ceil(4096))?;
        let ticket = self
            .lease
            .begin(self.receiver.unwrap(), now, LIFETIME)
            .map_err(|_| Error(abi::BUSY as i64))?;
        let source = self.source.unwrap();
        let frame = match world.request_frame(SOURCE, source.participant.incarnation, source.epoch)
        {
            Ok(frame) => frame,
            Err(code) => {
                self.lease.release(self.receiver.unwrap(), ticket).ok();
                return Err(Error(code as i64));
            }
        };
        self.failure = None;
        self.pending = Some(Pending {
            ticket,
            frame,
            deadline: now.wrapping_add(LIFETIME),
            target,
        });
        Ok(ticket)
    }
}
