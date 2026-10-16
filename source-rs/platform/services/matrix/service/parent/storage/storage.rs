// Authenticate a local scope, then relay the unchanged catalog protocol one hop.
// Owner grants/abort use the binding path here; child requests resume in pending.rs.
// Only the root adapter names provider links and retires them independently.
use super::*;
use cathedral_platform_contracts::storage as protocol;
use cathedral_user_runtime::server::Request;
pub(super) use pending::Pending;
impl Parent {
    pub(in crate::service) fn storage_client(&self, slot: usize) -> usize {
        if self.separate_storage {
            slot + 3
        } else {
            slot
        }
    }
    pub(in crate::service) fn storage_slot(&self, client: usize) -> usize {
        if self.separate_storage {
            client - 3
        } else {
            client
        }
    }
    pub(in crate::service) fn storage_stream(&self, client: usize) -> bool {
        if self.separate_storage {
            matches!(client, 3 | 4)
        } else {
            client < 2
        }
    }
    fn storage_exchange(&mut self, words: wire::Packet) -> Result<wire::Packet, Error> {
        let (send, receive) = self.storage_pair.ok_or(Error(abi::PEER_CLOSED as i64))?;
        let mut bytes = [0; 64];
        let result = send
            .send(&wire::encode(words))
            .and_then(|()| receive.receive_until(&mut bytes, time::after(100)?))
            .and_then(|len| wire::decode(&bytes[..len]).ok_or(Error(abi::IO_ERROR as i64)));
        if result.is_err() && self.separate_storage {
            self.retire_storage(0, 0)?;
        }
        result
    }
    fn storage_refresh(&mut self) -> Result<usize, Error> {
        if !self.upstream {
            return Ok(self.storage_links.len());
        }
        if self.storage_pair.is_none() {
            self.scopes.clear();
            self.storage_epoch = 0;
            return Ok(0);
        }
        let reply = match self.storage_exchange([wire::STORAGE_INFO, 0, 0, 0, 0, 0, 0, 0]) {
            Ok(reply) => reply,
            Err(_) if self.separate_storage => return Ok(0),
            Err(error) => return Err(error),
        };
        let epoch = if reply[0] == 0 { reply[1] } else { 0 };
        if epoch != self.storage_epoch {
            self.scopes.clear(); // Ancestor already aborted its old binding.
            self.storage_epoch = epoch;
        }
        Ok(usize::from(epoch != 0))
    }
    pub(in crate::service) fn storage_owner(
        &mut self,
        server: &Server,
        words: wire::Packet,
    ) -> Result<wire::Packet, Error> {
        let available = self.storage_refresh()?;
        let status = match words {
            [wire::STORAGE_GRANT, slot @ 0..=1, source, 0, 0, 0, 0, 0] => {
                match server.incarnation(self.storage_client(slot as usize)) {
                    Some(inc) => {
                        if !self.upstream && source < available as u64 {
                            let result = self.storage_links[source as usize]
                                .as_mut()
                                .ok_or(Error(abi::DENIED as i64))
                                .and_then(|client| client.pair());
                            if let Err(error) = result {
                                return Ok(wire::error(error.0 as u64));
                            }
                        }
                        self.scopes
                            .grant(slot as usize, inc, source as usize, available)
                    }
                    None => Err(abi::DENIED),
                }
            }
            [wire::STORAGE_REVOKE, slot @ 0..=1, 0, 0, 0, 0, 0, 0] => {
                self.storage_close(slot as usize)?;
                Ok(())
            }
            _ => Err(abi::DENIED),
        };
        Ok(wire::error(status.err().unwrap_or(0)))
    }
    pub(in crate::service) fn storage_close(&mut self, slot: usize) -> Result<(), Error> {
        self.storage_refresh()?;
        if let Some(source) = self.scopes.revoke(slot) {
            let reply = match self.storage_call(
                source,
                [wire::STORAGE_BASE + protocol::ABORT, 0, 0, 0, 0, 0, 0, 0],
            ) {
                Ok(reply) => reply,
                Err(error) if self.upstream && !self.separate_storage => return Err(error),
                // Binding is already revoked. A retired provider stream cannot
                // serve another child until explicit grant after provider restart.
                Err(_) => return Ok(()),
            };
            if reply[0] != 0 {
                return Err(Error(reply[0] as i64));
            }
        }
        Ok(())
    }
    pub(in crate::service) fn storage_request(
        &mut self,
        request: Request,
        words: wire::Packet,
    ) -> Result<(), Error> {
        assert!(self.storage_pending.is_none());
        self.storage_pending = Some(Pending::new(request, words, self.upstream)?);
        Ok(())
    }
    fn retire_storage(&mut self, source: usize, client: usize) -> Result<(), Error> {
        if self.upstream {
            if let Some((send, receive)) = self.storage_pair.take() {
                send.close()?;
                receive.close()?;
            }
            self.storage_epoch = 0;
            self.scopes.clear();
        } else {
            if let Some(link) = &mut self.storage_links[source] {
                link.invalidate();
            }
            self.scopes.revoke(self.storage_slot(client));
        }
        Ok(())
    }
    fn storage_call(
        &mut self,
        source: usize,
        mut words: wire::Packet,
    ) -> Result<wire::Packet, Error> {
        words[0] &= 0xffff;
        if self.upstream {
            words[0] |= self.storage_epoch << wire::STORAGE_SCOPE_BITS;
            return self.storage_exchange(words);
        }
        let bytes = wire::encode(words);
        if words[3] > protocol::CAPACITY as u64
            || bytes[32 + words[3] as usize..].iter().any(|b| *b != 0)
        {
            return Ok(wire::error(abi::INVALID_ARGUMENT));
        }
        let client = self.storage_links[source]
            .as_mut()
            .ok_or(Error(abi::DENIED as i64))?;
        match client.call(
            words[0] - wire::STORAGE_BASE,
            words[1],
            words[2],
            &bytes[32..32 + words[3] as usize],
        ) {
            Ok(record) => Ok(wire::decode(&record.reply()).unwrap()),
            // Provider protocol failures preserve the connection. Transport failures
            // are distinguished by the runtime so the service can retire safely.
            Err(error) if client.connected() => Ok(wire::error(error.0 as u64)),
            Err(error) => Err(error),
        }
    }
}
