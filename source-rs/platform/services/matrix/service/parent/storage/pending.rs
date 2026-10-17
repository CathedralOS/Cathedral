//! One bounded object exchange, independent of the interaction stream. The
//! matrix holds a forwarded attachment only until the next hop has accepted it.
//! A child's request is authorized by its binding and stamped with its scope.
//! Legacy shared-stream fixtures still serialize their upstream requests.
use super::*;
use cathedral_platform_contracts::storage as protocol;
pub(in crate::service::parent) struct Pending {
    request: Request,
    words: wire::Packet,
    attachment: Option<Object>,
    receive: Option<Handle>,
    refresh: bool,
    source: usize,
    deadline: u64,
}
impl Pending {
    pub(super) fn new(
        request: Request,
        words: wire::Packet,
        attachment: Option<Object>,
        upstream: bool,
    ) -> Result<Self, Error> {
        #[cfg(feature = "trace-storage")]
        time::storage_trace(
            if upstream {
                "nested-admit"
            } else {
                "root-admit"
            },
            request.client.unwrap() as u64,
            words[0] & 0xffff,
        );
        Ok(Self {
            request,
            words,
            attachment,
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
        self.publishing() || (self.upstream && !self.separate_storage && self.storing())
    }
    pub(in crate::service) fn progress_storage(
        &mut self,
        server: &mut Server,
    ) -> Result<(), Error> {
        let Some(mut p) = self.storage_pending.take() else {
            return Ok(());
        };
        let mut reply: Option<(wire::Packet, Option<Object>)> = None;
        if let Some(receive) = p.receive {
            let now = time::now()?;
            let mut bytes = [0; 64];
            let result = receive
                .try_receive_memory(&mut bytes)
                .and_then(|(len, object)| {
                    wire::decode(&bytes[..len])
                        .map(|words| (words, object))
                        .ok_or(Error(abi::IO_ERROR as i64))
                });
            match result {
                Ok((words, object)) => {
                    if p.refresh {
                        drop(object);
                        self.adopt_upstream(words);
                        p.refresh = false;
                        p.receive = None;
                    } else {
                        reply = Some((words, object));
                    }
                }
                Err(Error(code))
                    if (code == abi::TIMED_OUT as i64 || code == abi::WOULD_BLOCK as i64)
                        && !time::reached(now, p.deadline) =>
                {
                    self.storage_pending = Some(p);
                    return Ok(());
                }
                Err(Error(code)) if code == abi::WOULD_BLOCK as i64 => {
                    self.retire_after(&p, Error(abi::TIMED_OUT as i64), &mut reply)?;
                }
                Err(error) => {
                    self.retire_after(&p, error, &mut reply)?;
                }
            }
        }
        if reply.is_none() && p.receive.is_none() {
            if p.refresh {
                let result = self
                    .storage_pair
                    .ok_or(Error(abi::PEER_CLOSED as i64))
                    .and_then(|(send, receive)| {
                        send.send(&wire::encode([wire::STORAGE_INFO, 0, 0, 0, 0, 0, 0, 0]))?;
                        Ok(receive)
                    });
                match result {
                    Ok(receive) => p.receive = Some(receive),
                    Err(error) => self.retire_after(&p, error, &mut reply)?,
                }
            } else {
                match self.send_storage(&mut p) {
                    Ok(result) => reply = result.map(|words| (words, None)),
                    Err(error) => self.retire_after(&p, error, &mut reply)?,
                }
            }
        }
        if let Some((words, object)) = reply {
            #[cfg(feature = "trace-storage")]
            time::storage_trace(
                if self.upstream {
                    "nested-reply"
                } else {
                    "root-reply"
                },
                p.request.client.unwrap() as u64,
                p.words[0] & 0xffff,
            );
            match object {
                // The child must have reader funding; otherwise it gets the same
                // refusal the provider gives an unfunded reader.
                Some(object) => {
                    if server
                        .reply_memory(&p.request, &wire::encode(words), &object)
                        .is_err()
                    {
                        server.reply(&p.request, &wire::encode(wire::error(abi::BUSY)))?;
                    }
                }
                None => server.reply(&p.request, &wire::encode(words))?,
            }
        } else {
            self.storage_pending = Some(p);
        }
        Ok(())
    }
    /// A separate catalog stream can retire without losing input. Legacy shared
    /// streams must retire the whole service.
    fn retire_after(
        &mut self,
        p: &Pending,
        error: Error,
        reply: &mut Option<(wire::Packet, Option<Object>)>,
    ) -> Result<(), Error> {
        if self.upstream && !self.separate_storage {
            return Err(error);
        }
        self.retire_storage(p.source, p.request.client.unwrap())?;
        *reply = Some((wire::error(error.0 as u64), None));
        Ok(())
    }
    fn send_storage(&mut self, p: &mut Pending) -> Result<Option<wire::Packet>, Error> {
        let slot = self.storage_slot(p.request.client.unwrap());
        let incarnation = p.request.incarnation;
        let (source, scope) = match self.scopes.binding(slot, incarnation) {
            Ok(binding) => binding,
            Err(status) => return Ok(Some(wire::error(status))),
        };
        p.source = source;
        if p.words == [wire::STORAGE_INFO, 0, 0, 0, 0, 0, 0, 0] {
            // A binding without a scope of its own is rooted wherever this
            // matrix's own binding is.
            let realm = if scope != 0 {
                scope
            } else {
                self.storage_realm
            };
            let revision = self.scopes.revision(slot, incarnation).unwrap();
            return Ok(Some([0, revision, realm, 0, 0, 0, 0, 0]));
        }
        let mut words = p.words;
        let op = words[0] & 0xffff;
        // The child's scope narrows the request; the provider checks every stamp.
        if self
            .scopes
            .authorize(slot, incarnation, p.words[0] >> wire::STORAGE_SCOPE_BITS)
            .is_err()
            || !(wire::STORAGE_BASE + protocol::HEAD..=wire::STORAGE_BASE + protocol::LAST)
                .contains(&op)
            || protocol::stamp(&mut words, scope).is_err()
        {
            return Ok(Some(wire::error(abi::DENIED)));
        }
        let pair = if self.upstream {
            words[0] = op | (self.storage_epoch << wire::STORAGE_SCOPE_BITS);
            self.storage_pair.ok_or(Error(abi::PEER_CLOSED as i64))?
        } else {
            words[0] = op - wire::STORAGE_BASE;
            self.storage_links[source]
                .as_mut()
                .ok_or(Error(abi::DENIED as i64))?
                .pair()?
        };
        #[cfg(feature = "trace-storage")]
        time::storage_trace(
            if self.upstream {
                "nested-send"
            } else {
                "root-send"
            },
            slot as u64,
            words[0] & 0xffff,
        );
        match p.attachment.take() {
            Some(object) => pair.0.send_memory(&wire::encode(words), &object)?,
            None => pair.0.send(&wire::encode(words))?,
        }
        p.receive = Some(pair.1);
        Ok(None)
    }
}
