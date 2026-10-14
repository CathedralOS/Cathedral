//! One bounded child exchange. Nested matrices serialize their shared upstream
//! stream; the root waits on a distinct provider link without blocking display.
use super::*;
pub(in crate::service::parent) struct Pending {
    request: Request,
    words: wire::Packet,
    receive: Option<Handle>,
    refresh: bool,
    source: usize,
    deadline: u64,
}
impl Pending {
    pub(super) fn new(
        request: Request,
        words: wire::Packet,
        upstream: bool,
    ) -> Result<Self, Error> {
        Ok(Self {
            request,
            words,
            receive: None,
            refresh: upstream,
            source: 0,
            deadline: time::after(if upstream { 250 } else { 150 })?,
        })
    }
}
impl Parent {
    pub(in crate::service) fn storage_wait(&self) -> Option<(Handle, u64)> {
        self.storage_pending
            .as_ref()
            .and_then(|p| p.receive.map(|h| (h, p.deadline)))
    }
    pub(in crate::service) fn storing(&self) -> bool {
        self.storage_pending.is_some()
    }
    pub(in crate::service) fn storing_client(&self) -> usize {
        self.storage_pending
            .as_ref()
            .unwrap()
            .request
            .client
            .unwrap()
    }
    pub(in crate::service) fn upstream_busy(&self) -> bool {
        self.publishing() || (self.upstream && self.storing())
    }
    pub(in crate::service) fn progress_storage(
        &mut self,
        server: &mut Server,
    ) -> Result<(), Error> {
        let Some(mut p) = self.storage_pending.take() else {
            return Ok(());
        };
        let mut reply = None;
        if let Some(receive) = p.receive {
            let now = time::now()?;
            let mut bytes = [0; 64];
            match receive.receive_until(&mut bytes, now) {
                Ok(len) => {
                    let words = wire::decode(&bytes[..len]).ok_or(Error(abi::IO_ERROR as i64))?;
                    if p.refresh {
                        let epoch = if words[0] == 0 { words[1] } else { 0 };
                        if epoch != self.storage_epoch {
                            self.scopes.clear();
                            self.storage_epoch = epoch;
                        }
                        p.refresh = false;
                        p.receive = None;
                    } else {
                        reply = Some(words);
                    }
                }
                Err(Error(code))
                    if code == abi::TIMED_OUT as i64 && !time::reached(now, p.deadline) =>
                {
                    self.storage_pending = Some(p);
                    return Ok(());
                }
                Err(error) => {
                    // A late reply on the shared parent connection would corrupt
                    // other protocols. Root provider links can be retired alone.
                    if self.upstream {
                        return Err(error);
                    }
                    self.storage_links[p.source].as_mut().unwrap().invalidate();
                    self.scopes.revoke(p.request.client.unwrap());
                    reply = Some(wire::error(error.0 as u64));
                }
            }
        }
        if reply.is_none() && p.receive.is_none() {
            if p.refresh {
                let (send, receive) = self.pair.ok_or(Error(abi::PEER_CLOSED as i64))?;
                send.send(&wire::encode([wire::STORAGE_INFO, 0, 0, 0, 0, 0, 0, 0]))?;
                p.receive = Some(receive);
            } else {
                match self.send_storage(&mut p) {
                    Ok(result) => reply = result,
                    Err(error) if self.upstream => return Err(error),
                    Err(error) => {
                        if let Some(client) = &mut self.storage_links[p.source] {
                            client.invalidate();
                        }
                        self.scopes.revoke(p.request.client.unwrap());
                        reply = Some(wire::error(error.0 as u64));
                    }
                }
            }
        }
        if let Some(reply) = reply {
            server.reply(&p.request, &wire::encode(reply))?;
        } else {
            self.storage_pending = Some(p);
        }
        Ok(())
    }
    fn send_storage(&mut self, p: &mut Pending) -> Result<Option<wire::Packet>, Error> {
        let slot = p.request.client.unwrap();
        let incarnation = p.request.incarnation;
        let source = match self.scopes.source(slot, incarnation) {
            Ok(source) => source,
            Err(status) => return Ok(Some(wire::error(status))),
        };
        p.source = source;
        if p.words == [wire::STORAGE_INFO, 0, 0, 0, 0, 0, 0, 0] {
            return Ok(Some([
                0,
                self.scopes.revision(slot, incarnation).unwrap(),
                0,
                0,
                0,
                0,
                0,
                0,
            ]));
        }
        let op = p.words[0] & 0xffff;
        if self
            .scopes
            .authorize(slot, incarnation, p.words[0] >> wire::STORAGE_SCOPE_BITS)
            .is_err()
            || !(wire::STORAGE_BASE + protocol::READ..=wire::STORAGE_BASE + protocol::ABORT)
                .contains(&op)
        {
            return Ok(Some(wire::error(abi::DENIED)));
        }
        let mut words = p.words;
        let pair = if self.upstream {
            words[0] = op | (self.storage_epoch << wire::STORAGE_SCOPE_BITS);
            self.pair.ok_or(Error(abi::PEER_CLOSED as i64))?
        } else {
            words[0] = op - wire::STORAGE_BASE;
            self.storage_links[source]
                .as_mut()
                .ok_or(Error(abi::DENIED as i64))?
                .pair()?
        };
        pair.0.send(&wire::encode(words))?;
        p.receive = Some(pair.1);
        Ok(None)
    }
}
