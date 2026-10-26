//! Move the retained event exactly once; recheck connection identity at dispatch.
use super::*;
impl Host {
    pub(super) fn defer_event(&mut self) -> Result<(), Error> {
        let event = self.event.take().expect("retained blocked event");
        #[cfg(feature = "trace-storage")]
        deferred::trace_loss(&event, self.parent);
        #[cfg(feature = "interaction-delay")]
        if self.parent.refresh_late()
            && let Event::Request(r) = &event
            && wire::decode(&r.bytes[..r.len])
                .is_some_and(|w| w[0] == wire::POLL_TAGGED || w[0] == wire::POLL)
        {
            cathedral_user_runtime::time::storage_sample("matrix-poll-local", &[]);
        }
        let mut blocked = self.parent.admitting.map_or(0, |slot| 1 << slot);
        if self.parent.storing()
            && let Some(client) = self.parent.storing_client()
        {
            blocked |= 1 << client;
        }
        self.deferred.hold(event, self.server, self.world, blocked)
    }
    pub(super) fn dispatch(&mut self, command: Command) -> Result<(), Error> {
        let event = self.event.take().expect("retained dispatch event");
        match (command, event) {
            (
                Command::Disconnect,
                Event::Disconnected {
                    client,
                    incarnation,
                },
            ) => {
                if self
                    .server
                    .incarnation(client)
                    .is_some_and(|current| current != incarnation)
                {
                    return Ok(());
                }
                if client < CHILDREN {
                    self.children
                        .disconnected(client, self.server, self.world, self.parent)?;
                } else if client == SOURCE {
                    requests::source::cancel(self.world);
                } else if client >= PRODUCER_BASE {
                    self.children.producer_disconnected(
                        client - PRODUCER_BASE,
                        incarnation,
                        self.world,
                        self.server,
                    )?;
                } else {
                    self.parent
                        .storage_close(self.parent.storage_slot(client))?;
                }
            }
            (Command::ProducerMemory, Event::Memory(request, object)) => {
                drop(object);
                self.children
                    .producer_request(&request, None, self.world, self.server)?;
            }
            (Command::StorageMemory, Event::Memory(request, object)) => {
                requests::attachment(request, object, self.parent, self.server)?
            }
            (Command::ImageMemory, Event::Memory(request, object)) => {
                requests::image(request, object, self.parent, self.world, self.server)?
            }
            (Command::Request, Event::Request(request)) => {
                requests::dispatch(request, self.parent, self.world, self.children, self.server)?
            }
            _ => return Err(Error(abi::IO_ERROR as i64)),
        }
        Ok(())
    }
}
