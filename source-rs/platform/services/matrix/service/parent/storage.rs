//! Authenticate a local scope, then relay the unchanged catalog protocol one hop.
//! Only the root adapter can name actual provider links. Transport loss retires
//! this matrix so a late mutation reply can never become another child's reply.
use super::*;
use cathedral_contracts::storage;
impl Parent {
    fn storage_refresh(&mut self) -> Result<usize, Error> {
        if !self.upstream {
            return Ok(self.storage_links.iter().flatten().count());
        }
        let reply = self.call([wire::STORAGE_INFO, 0, 0, 0, 0, 0, 0, 0])?;
        let epoch = if reply[0] == 0 { reply[1] } else { 0 };
        if epoch != self.storage_epoch {
            self.scopes.clear(); // Ancestor already aborted its old binding.
            self.storage_epoch = epoch;
        }
        Ok(usize::from(epoch != 0))
    }
    pub(in super::super) fn storage_owner(
        &mut self,
        server: &Server,
        words: wire::Packet,
    ) -> Result<wire::Packet, Error> {
        let available = self.storage_refresh()?;
        let status = match words {
            [wire::STORAGE_GRANT, slot @ 0..=1, source, 0, 0, 0, 0, 0] => {
                match server.incarnation(slot as usize) {
                    Some(inc) => self
                        .scopes
                        .grant(slot as usize, inc, source as usize, available),
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
    pub(in super::super) fn storage_close(&mut self, slot: usize) -> Result<(), Error> {
        self.storage_refresh()?;
        if let Some(source) = self.scopes.revoke(slot) {
            let reply = self.storage_call(
                source,
                [wire::STORAGE_BASE + storage::ABORT, 0, 0, 0, 0, 0, 0, 0],
            )?;
            if reply[0] != 0 {
                return Err(Error(reply[0] as i64));
            }
        }
        Ok(())
    }
    pub(in super::super) fn storage_request(
        &mut self,
        slot: usize,
        incarnation: u64,
        words: wire::Packet,
    ) -> Result<wire::Packet, Error> {
        self.storage_refresh()?;
        let source = match self.scopes.source(slot, incarnation) {
            Ok(source) => source,
            Err(status) => return Ok(wire::error(status)),
        };
        if words == [wire::STORAGE_INFO, 0, 0, 0, 0, 0, 0, 0] {
            return Ok([
                0,
                self.scopes.revision(slot, incarnation).unwrap(),
                0,
                0,
                0,
                0,
                0,
                0,
            ]);
        }
        if self
            .scopes
            .authorize(slot, incarnation, words[0] >> wire::STORAGE_SCOPE_BITS)
            .is_err()
            || !(wire::STORAGE_BASE + storage::READ..=wire::STORAGE_BASE + storage::ABORT)
                .contains(&(words[0] & 0xffff))
        {
            return Ok(wire::error(abi::DENIED));
        }
        self.storage_call(source, words)
    }
    fn storage_call(
        &mut self,
        source: usize,
        mut words: wire::Packet,
    ) -> Result<wire::Packet, Error> {
        words[0] &= 0xffff;
        if self.upstream {
            words[0] |= self.storage_epoch << wire::STORAGE_SCOPE_BITS;
            return self.call(words);
        }
        let bytes = wire::encode(words);
        if words[3] > storage::CAPACITY as u64
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
