// Authenticate a local scope, then relay the object protocol one hop, with
// sealed attachments travelling up (publish) and down (bulk reads) unchanged
// and the child's granted scope stamped on the way. Owner grants/revokes act
// here; child requests and the matrix's own program reads resume in
// pending.rs; binding.rs resumes ordinary launch binding refreshes. Only the root adapter names provider links and retires them
// independently.
use super::*;
pub(super) use binding::Binding;
pub(in crate::service) use binding::Bound;
use cathedral_user_runtime::{memory::Object, server::Request};
pub(super) use pending::Pending;
pub(in crate::service) use pending::{LaunchOrigin, LaunchRequest, Launched};

impl Parent {
    /// Retire a failed local exchange just as forwarded requests do: never
    /// reuse a stream whose late reply could be mistaken for the next request.
    pub(in crate::service) fn local_storage<T>(
        &mut self,
        operation: impl FnOnce(&mut cathedral_user_runtime::storage::Scope) -> Result<T, Error>,
    ) -> Result<T, Error> {
        let (result, connected) = {
            let mut scope = self.own_scope()?;
            let result = operation(&mut scope);
            (result, scope.connected())
        };
        if !connected {
            self.retire_storage(0)?;
        }
        result
    }
    fn own_scope(&mut self) -> Result<cathedral_user_runtime::storage::Scope<'_>, Error> {
        if self.storing() {
            return Err(Error(abi::BUSY as i64));
        }
        if !self.upstream {
            return Ok(cathedral_user_runtime::storage::Scope::provider(
                self.storage_links[0]
                    .as_mut()
                    .ok_or(Error(abi::DENIED as i64))?
                    .client(time::after(300)?)?,
            ));
        }
        if !self.separate_storage {
            return Err(Error(abi::DENIED as i64));
        }
        Ok(cathedral_user_runtime::storage::Scope::new(
            self.storage_pair.ok_or(Error(abi::PEER_CLOSED as i64))?,
        ))
    }

    pub(in crate::service) fn storage_client(&self, slot: usize) -> usize {
        if self.separate_storage {
            slot + STORAGE_BASE
        } else {
            slot
        }
    }
    pub(in crate::service) fn storage_slot(&self, client: usize) -> usize {
        if self.separate_storage {
            client - STORAGE_BASE
        } else {
            client
        }
    }
    pub(in crate::service) fn storage_stream(&self, client: usize) -> bool {
        if self.separate_storage {
            (STORAGE_BASE..STORAGE_BASE + CHILDREN).contains(&client)
        } else {
            client < CHILDREN
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
            self.retire_storage(0)?;
        }
        result
    }
    /// Take the upstream binding's current revision and realm. A changed
    /// revision means the ancestor revoked the old binding, which empties
    /// every grant made under it.
    pub(super) fn adopt_upstream(&mut self, reply: wire::Packet) {
        let (epoch, realm) = if reply[0] == 0 {
            (reply[1], reply[2])
        } else {
            (0, 0)
        };
        if epoch != self.storage_epoch {
            self.scopes.clear();
            self.storage_epoch = epoch;
        }
        self.storage_realm = realm;
    }
    fn storage_refresh(&mut self) -> Result<usize, Error> {
        if !self.upstream {
            return Ok(self.storage_links.len());
        }
        if self.storage_pair.is_none() {
            self.scopes.clear();
            self.storage_epoch = 0;
            self.storage_realm = 0;
            return Ok(0);
        }
        let reply = match self.storage_exchange([wire::STORAGE_INFO, 0, 0, 0, 0, 0, 0, 0]) {
            Ok(reply) => reply,
            Err(_) if self.separate_storage => return Ok(0),
            Err(error) => return Err(error),
        };
        self.adopt_upstream(reply);
        Ok(usize::from(self.storage_epoch != 0))
    }
    pub(in crate::service) fn storage_owner(
        &mut self,
        server: &Server,
        words: wire::Packet,
    ) -> Result<wire::Packet, Error> {
        if words == [wire::STORAGE_STATUS, 0, 0, 0, 0, 0, 0, 0] && !self.upstream {
            return Ok([0, self.storage_failed, 0, 0, 0, 0, 0, 0]);
        }
        if let [
            wire::STORAGE_BINDING,
            slot @ 0..=LAST_CHILD,
            0,
            0,
            0,
            0,
            0,
            0,
        ] = words
        {
            return Ok(match self.storage_binding(server, slot as usize) {
                Some((source, scope)) => [0, source as u64, scope, 0, 0, 0, 0, 0],
                None => wire::error(abi::DENIED),
            });
        }
        let status = match words {
            [
                wire::STORAGE_GRANT,
                slot @ 0..=LAST_CHILD,
                source,
                scope,
                0,
                0,
                0,
                0,
            ] => self.storage_grant(server, slot as usize, source as usize, scope)?,
            [
                wire::STORAGE_REVOKE,
                slot @ 0..=LAST_CHILD,
                0,
                0,
                0,
                0,
                0,
                0,
            ] => {
                self.storage_close(slot as usize)?;
                Ok(())
            }
            _ => Err(abi::DENIED),
        };
        Ok(wire::error(status.err().unwrap_or(0)))
    }
    /// Bind a child's live catalog connection to a source and scope. At the
    /// root this also re-establishes the provider link, so a grant after
    /// provider replacement connects to the fresh incarnation.
    pub(in crate::service) fn storage_grant(
        &mut self,
        server: &Server,
        slot: usize,
        source: usize,
        scope: u64,
    ) -> Result<Result<(), u64>, Error> {
        let available = self.storage_refresh()?;
        Ok(self.install_storage_binding(server, slot, source, scope, available))
    }
    /// Local commit, after the caller has validated the asynchronous refresh result.
    pub(in crate::service) fn install_storage_binding(
        &mut self,
        server: &Server,
        slot: usize,
        source: usize,
        scope: u64,
        available: usize,
    ) -> Result<(), u64> {
        let Some(incarnation) = server.incarnation(self.storage_client(slot)) else {
            return Err(abi::DENIED);
        };
        if !self.upstream && source < available {
            let result = self.storage_links[source]
                .as_mut()
                .ok_or(Error(abi::DENIED as i64))
                .and_then(|provider| provider.refresh());
            match result {
                Ok(true) => self.scopes.revoke_source(source),
                Ok(false) => (),
                Err(error) => return Err(error.0 as u64),
            }
        }
        self.scopes
            .grant(slot, incarnation, source, scope, available)
            .map(|()| {
                self.storage_failed &= !(1 << source);
            })
    }
    /// A child's live storage binding: its source and the scope it was granted.
    pub(in crate::service) fn storage_binding(
        &self,
        server: &Server,
        slot: usize,
    ) -> Option<(usize, u64)> {
        let incarnation = server.incarnation(self.storage_client(slot))?;
        self.scopes.binding(slot, incarnation).ok()
    }
    /// Begin reading a program someone asked to run; their request is answered
    /// once the child is admitted.
    pub(in crate::service) fn storage_launch(
        &mut self,
        request: Request,
        plan: LaunchRequest,
        ticket: cathedral_stock_matrix::policy::operation::Ticket,
    ) -> Result<(), Error> {
        assert!(self.storage_pending.is_none());
        self.storage_pending = Some(Pending::launch(
            LaunchOrigin::Caller(request, ticket),
            plan,
            self.upstream,
            self.storage_epoch != 0,
        )?);
        Ok(())
    }
    /// A configured launcher recovery uses our own authority and has no caller
    /// to answer. The child service consumes the sealed result in its event loop.
    pub(in crate::service) fn storage_recover(
        &mut self,
        plan: LaunchRequest,
        ticket: cathedral_stock_matrix::policy::recovery::Ticket,
    ) -> Result<(), Error> {
        assert!(self.storage_pending.is_none());
        self.storage_pending = Some(Pending::launch(
            LaunchOrigin::Recovery(ticket),
            plan,
            self.upstream,
            self.storage_epoch != 0,
        )?);
        Ok(())
    }
    /// Internal identity only. Recovery must not reinterpret a saved object ID
    /// under a different provider connection or upstream realm incarnation.
    pub(in crate::service) fn storage_identity(&self) -> Option<[u64; 4]> {
        let pair = if self.upstream {
            if self.storage_epoch == 0 {
                return None;
            }
            let pair = self.storage_pair?;
            [pair.0.raw(), pair.1.raw()]
        } else {
            self.storage_links[0].as_ref()?.identity()?
        };
        Some([pair[0], pair[1], self.storage_epoch, self.storage_realm])
    }
    /// Every object operation is atomic at the provider, so revoking a binding
    /// leaves nothing staged upstream; the stale scope simply stops authorizing.
    pub(in crate::service) fn storage_close(&mut self, slot: usize) -> Result<(), Error> {
        // Revocation is local. Never wait on the provider to withdraw authority;
        // subsequent operations still validate their stamped upstream binding.
        self.scopes.revoke(slot);
        Ok(())
    }
    pub(in crate::service) fn storage_request(
        &mut self,
        request: Request,
        words: wire::Packet,
        attachment: Option<Object>,
    ) -> Result<(), Error> {
        assert!(self.storage_pending.is_none());
        self.storage_pending = Some(Pending::new(
            request,
            words,
            attachment,
            self.upstream,
            self.storage_epoch != 0,
        )?);
        Ok(())
    }
    fn retire_storage(&mut self, source: usize) -> Result<(), Error> {
        if self.upstream {
            if let Some((send, receive)) = self.storage_pair.take() {
                send.close()?;
                receive.close()?;
            }
            self.storage_epoch = 0;
            self.storage_realm = 0;
            self.scopes.clear();
        } else {
            self.storage_failed |= 1 << source;
            if let Some(provider) = &mut self.storage_links[source] {
                provider.invalidate();
            }
            self.scopes.revoke_source(source);
        }
        Ok(())
    }
}
