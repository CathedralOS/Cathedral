//! One bounded object exchange, independent of the interaction stream. The
//! matrix holds a forwarded attachment only until the next hop has accepted it.
//! A child's request is authorized by its binding and stamped with its scope.
//! The matrix's own reads, which fetch a program it was asked to run, take the
//! same path on the matrix's own binding: HEAD the object, then BULK it.
//! Legacy shared-stream fixtures still serialize their upstream requests.
use super::*;
use cathedral_matrix::world::storage::Proof;
use cathedral_platform_contracts::storage as protocol;
/// A request to run a program from this matrix's realm, carried while the
/// object is read and answered once the child is admitted. The owner's names
/// the realm to grant; a child's is read with the requester's scope stamped,
/// so a launcher can only launch what it can see.
#[derive(Clone, Copy)]
pub(in crate::service) struct LaunchRequest {
    pub slot: usize,
    pub object: u64,
    pub scope: u64,
    pub argument: u64,
    pub channels: u64,
    pub descendants: u64,
    pub cpu: u64,
    /// A child's launch carries the slot's standing pages; the owner's uses
    /// what RUN_PAGES and RUN_MEMORY staged, consumed only on admission.
    pub pages: Option<[u64; 2]>,
    pub stamp: u64,
    /// Offered services after owner policy and caller-requested withholding.
    pub imports: u64,
}
/// A program read that finished: the external request to answer (none for local
/// launcher recovery) and the sealed image, or why there is none.
pub(in crate::service) struct Launched {
    pub plan: LaunchRequest,
    pub request: Option<Request>,
    pub result: Result<Object, u64>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Step {
    /// Refresh the upstream binding before anything else.
    Refresh,
    /// Learn the program's revision and length.
    Head,
    /// Prove a full scope packet's intersection before forwarding its mutation.
    Validate,
    /// The request itself: a child's, or the program's bulk read.
    Forward,
}
pub(in crate::service::parent) struct Pending {
    request: Option<Request>,
    words: wire::Packet,
    attachment: Option<Object>,
    receive: Option<Handle>,
    step: Step,
    source: usize,
    deadline: u64,
    launch: Option<LaunchRequest>,
    validated: Option<(wire::Packet, Proof)>,
}
impl Pending {
    pub(super) fn new(
        request: Request,
        words: wire::Packet,
        attachment: Option<Object>,
        upstream: bool,
    ) -> Result<Self, Error> {
        let now = time::now()?;
        let deadline = protocol::deadline(
            now,
            words[7],
            if upstream {
                protocol::EXCHANGE_TICKS
            } else {
                wire::STORAGE_DEADLINE_TICKS
            },
        );
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
            request: Some(request),
            words,
            attachment,
            receive: None,
            step: if upstream && deadline.is_some() {
                Step::Refresh
            } else {
                Step::Forward
            },
            source: 0,
            deadline: deadline.unwrap_or(now),
            launch: None,
            validated: None,
        })
    }
    /// Read a program on this matrix's own binding: a head and a bulk read,
    /// behind a refresh when nested, each bounded like a forwarded request.
    pub(super) fn launch(
        request: Option<Request>,
        plan: LaunchRequest,
        upstream: bool,
    ) -> Result<Self, Error> {
        Ok(Self {
            request,
            words: [
                wire::STORAGE_BASE + protocol::HEAD,
                plan.object,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
            attachment: None,
            receive: None,
            step: if upstream { Step::Refresh } else { Step::Head },
            source: 0,
            deadline: time::after(
                if upstream { 3 } else { 2 } * wire::STORAGE_DEADLINE_TICKS + 100,
            )?,
            launch: Some(plan),
            validated: None,
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
    /// The requesting child, if any; owner launches and local recovery have none.
    pub(in crate::service) fn storing_client(&self) -> Option<usize> {
        self.storage_pending
            .as_ref()
            .and_then(|p| p.request.and_then(|r| r.client))
    }
    pub(in crate::service) fn upstream_busy(&self) -> bool {
        self.publishing() || (self.upstream && !self.separate_storage && self.storing())
    }
    /// Advance the pending exchange. A finished program read is handed back
    /// for admission; a finished child request is answered here.
    pub(in crate::service) fn progress_storage(
        &mut self,
        server: &mut Server,
    ) -> Result<Option<Launched>, Error> {
        let Some(mut p) = self.storage_pending.take() else {
            return Ok(None);
        };
        let mut reply: Option<(wire::Packet, Option<Object>)> = None;
        if let Some(receive) = p.receive {
            match cathedral_user_runtime::matrix::upstream::exchange::poll(receive, p.deadline) {
                Ok(Some((words, object))) => match p.step {
                    Step::Refresh => {
                        drop(object);
                        self.adopt_upstream(words);
                        p.step = if p.launch.is_some() {
                            Step::Head
                        } else {
                            Step::Forward
                        };
                        p.receive = None;
                    }
                    Step::Head => {
                        drop(object);
                        if words[0] != 0 {
                            reply = Some((words, None));
                        } else {
                            // Fetch exactly the revision just seen, whole.
                            p.words = [
                                wire::STORAGE_BASE + protocol::BULK,
                                words[1],
                                words[2],
                                0,
                                words[3],
                                0,
                                0,
                                0,
                            ];
                            p.step = Step::Forward;
                            p.receive = None;
                        }
                    }
                    Step::Validate => {
                        drop(object);
                        if words[0] != 0 {
                            reply = Some((words, None));
                        } else if let Err(code) =
                            self.scopes.confirm(p.validated.as_ref().unwrap().1)
                        {
                            reply = Some((wire::error(code), None));
                        } else {
                            p.step = Step::Forward;
                            p.receive = None;
                        }
                    }
                    Step::Forward => reply = Some((words, object)),
                },
                Ok(None) => {
                    self.storage_pending = Some(p);
                    return Ok(None);
                }
                Err(error) => {
                    self.retire_after(&p, error, &mut reply)?;
                }
            }
        }
        if reply.is_none() && p.receive.is_none() {
            if p.step == Step::Refresh {
                let result = self
                    .storage_pair
                    .ok_or(Error(abi::PEER_CLOSED as i64))
                    .and_then(|(send, receive)| {
                        send.send(&wire::encode([
                            wire::STORAGE_INFO,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            p.deadline,
                        ]))?;
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
        let Some((words, object)) = reply else {
            self.storage_pending = Some(p);
            return Ok(None);
        };
        if let Some(plan) = p.launch {
            let result = match (words[0], object) {
                (0, Some(object)) => Ok(object),
                (0, None) => Err(abi::IO_ERROR),
                (status, object) => {
                    drop(object);
                    Err(status)
                }
            };
            return Ok(Some(Launched {
                plan,
                request: p.request,
                result,
            }));
        }
        let request = p.request.expect("forwarded storage request");
        #[cfg(feature = "trace-storage")]
        time::storage_trace(
            if self.upstream {
                "nested-reply"
            } else {
                "root-reply"
            },
            request.client.unwrap() as u64,
            p.words[0] & 0xffff,
        );
        match object {
            // The child must have reader funding; otherwise it gets the same
            // refusal the provider gives an unfunded reader.
            Some(object) => {
                if let Err(error) = server.reply_memory(&request, &wire::encode(words), &object) {
                    time::storage_sample("storage-reader-refused", &[error.0 as u64]);
                    server.reply(&request, &wire::encode(wire::error(error.0 as u64)))?;
                }
            }
            None => server.reply(&request, &wire::encode(words))?,
        }
        Ok(None)
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
        #[cfg(feature = "trace-storage")]
        time::storage_sample(
            "storage-retire",
            &[
                u64::from(self.upstream),
                p.words[0] & 0xffff,
                p.deadline,
                time::now()?,
                error.0 as u64,
            ],
        );
        self.retire_storage(p.source)?;
        *reply = Some((wire::error(error.0 as u64), None));
        Ok(())
    }
    fn send_storage(&mut self, p: &mut Pending) -> Result<Option<wire::Packet>, Error> {
        if time::reached(time::now()?, p.deadline) {
            return Ok(Some(wire::error(abi::TIMED_OUT)));
        }
        let mut words = p.words;
        if let Some(plan) = p.launch {
            // The matrix's own read: its upstream binding, or at the root its
            // first provider root. A child's launch is narrowed to that child's
            // realm; the parent stamps this matrix's scope on top.
            p.source = 0;
            if protocol::stamp(&mut words, plan.stamp).is_err() {
                return Ok(Some(wire::error(abi::DENIED)));
            }
        } else if let Some(request) = p.request {
            let client = request.client.expect("forwarded child request");
            let slot = self.storage_slot(client);
            use cathedral_matrix::world::storage::Prepared;
            if let Some((validated, _)) = p.validated.take() {
                if let Err(code) = self.scopes.authorize(
                    slot,
                    request.incarnation,
                    words[0] >> wire::STORAGE_SCOPE_BITS,
                ) {
                    return Ok(Some(wire::error(code)));
                }
                words = validated;
            } else {
                match self
                    .scopes
                    .prepare(slot, request.incarnation, words, self.storage_realm)
                {
                    Ok(Prepared::Reply(reply)) => return Ok(Some(reply)),
                    Ok(Prepared::Forward {
                        source,
                        words: stamped,
                    }) => {
                        p.source = source;
                        words = stamped;
                    }
                    Ok(Prepared::Validate {
                        source,
                        check,
                        words: folded,
                        proof,
                    }) => {
                        p.source = source;
                        p.validated = Some((folded, proof));
                        p.step = Step::Validate;
                        words = check;
                    }
                    Err(code) => return Ok(Some(wire::error(code))),
                }
            }
        }
        words[7] = p.deadline;
        let op = words[0] & 0xffff;
        let pair = if self.upstream {
            words[0] = op | (self.storage_epoch << wire::STORAGE_SCOPE_BITS);
            self.storage_pair.ok_or(Error(abi::PEER_CLOSED as i64))?
        } else {
            words[0] = op - wire::STORAGE_BASE;
            self.storage_links[p.source]
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
            match (p.launch, p.request.and_then(|r| r.client)) {
                (Some(_), _) => 2,
                (None, Some(c)) => self.storage_slot(c) as u64,
                (None, None) => 2,
            },
            words[0] & 0xffff,
        );
        let attachment = if p.step == Step::Validate {
            None
        } else {
            p.attachment.take()
        };
        match attachment {
            Some(object) => pair.0.send_memory(&wire::encode(words), &object)?,
            None => pair.0.send(&wire::encode(words))?,
        }
        p.receive = Some(pair.1);
        Ok(None)
    }
}
