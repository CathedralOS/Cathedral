//! One bounded object exchange, independent of the interaction stream. The
//! matrix holds a forwarded attachment only until the next hop has accepted it.
//! A child's request is authorized by its binding and stamped with its scope.
//! The matrix's own reads, which fetch a program it was asked to run, take the
//! same path on the matrix's own binding: one BULK of the whole object.
//! A nested matrix learns its upstream binding once, before its first
//! exchange, and again only after the parent refuses a stamped revision; it
//! does not ask before every request, because each refresh would itself be
//! answered by the parent only after the parent refreshed, all the way up.
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
/// Completion custody: an authenticated caller or the exact local recovery operation.
#[derive(Clone, Copy)]
pub(in crate::service) enum LaunchOrigin {
    Caller(Request, cathedral_stock_matrix::policy::operation::Ticket),
    Recovery(cathedral_stock_matrix::policy::recovery::Ticket),
}
/// A completed program read, with its origin and sealed image or refusal.
pub(in crate::service) struct Launched {
    pub origin: LaunchOrigin,
    pub result: Result<Object, u64>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Step {
    /// Learn the upstream binding revision: before this matrix's first
    /// exchange, or once more after the parent refuses a stamped revision.
    Refresh,
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
    launch: Option<(LaunchRequest, LaunchOrigin)>,
    validated: Option<(wire::Packet, Proof)>,
    /// The parent refused this exchange's stamped revision once already; one
    /// refresh follows, never a loop.
    refreshed: bool,
    /// Where an own program read continues after that refresh.
    resume: Option<Step>,
    /// A child's refusal, answered after the refresh; its attachment is gone.
    denied: Option<wire::Packet>,
    #[cfg(any(feature = "recovery-delay", feature = "launch-delay"))]
    hold_until: Option<u64>,
}
impl Pending {
    /// `bound` says this matrix already holds its upstream binding revision;
    /// otherwise the exchange begins by learning it.
    pub(super) fn new(
        request: Request,
        words: wire::Packet,
        attachment: Option<Object>,
        upstream: bool,
        bound: bool,
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
            step: if upstream && deadline.is_some() && !bound {
                Step::Refresh
            } else {
                Step::Forward
            },
            source: 0,
            deadline: deadline.unwrap_or(now),
            launch: None,
            validated: None,
            refreshed: false,
            resume: None,
            denied: None,
            #[cfg(any(feature = "recovery-delay", feature = "launch-delay"))]
            hold_until: None,
        })
    }
    /// Read a program on this matrix's own binding: one bulk read of the whole
    /// object at its current revision, behind a refresh when nested and not
    /// yet bound, bounded like a forwarded request.
    pub(super) fn launch(
        origin: LaunchOrigin,
        plan: LaunchRequest,
        upstream: bool,
        bound: bool,
    ) -> Result<Self, Error> {
        Ok(Self {
            request: match origin {
                LaunchOrigin::Caller(r, _) => Some(r),
                LaunchOrigin::Recovery(_) => None,
            },
            words: [
                wire::STORAGE_BASE + protocol::BULK,
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
            step: if upstream && !bound {
                Step::Refresh
            } else {
                Step::Forward
            },
            source: 0,
            deadline: time::after(
                if upstream { 3 } else { 2 } * wire::STORAGE_DEADLINE_TICKS + 100,
            )?,
            launch: Some((plan, origin)),
            validated: None,
            refreshed: false,
            resume: None,
            denied: None,
            #[cfg(any(feature = "recovery-delay", feature = "launch-delay"))]
            hold_until: if (cfg!(feature = "recovery-delay")
                && matches!(origin, LaunchOrigin::Recovery(_)))
                || (cfg!(feature = "launch-delay")
                    && matches!(origin, LaunchOrigin::Caller(_, _))
                    && plan.slot == 2)
            {
                time::storage_sample(
                    if matches!(origin, LaunchOrigin::Recovery(_)) {
                        "launcher-read-held"
                    } else {
                        "launch-read-held"
                    },
                    &[],
                );
                Some(time::after(300)?)
            } else {
                None
            },
        })
    }
}
impl Parent {
    #[cfg(any(feature = "recovery-delay", feature = "launch-delay"))]
    pub(in crate::service) fn recovery_delay_deadline(&self) -> Option<u64> {
        self.storage_pending.as_ref().and_then(|p| p.hold_until)
    }
    pub(in crate::service) fn storage_wait(&self) -> Option<(Handle, u64)> {
        if self.binding_pending.is_some() {
            return self.binding_wait();
        }
        self.storage_pending.as_ref().and_then(|p| {
            #[cfg(any(feature = "recovery-delay", feature = "launch-delay"))]
            if p.hold_until.is_some() {
                return None;
            }
            p.receive.map(|h| (h, p.deadline))
        })
    }
    pub(in crate::service) fn storage_wake(
        &self,
    ) -> Option<cathedral_user_runtime::event::Interest> {
        if self.upstream || self.storage_wait().is_none() {
            return None;
        }
        self.storage_links[self.storage_pending.as_ref()?.source]
            .as_ref()?
            .wake()
    }
    pub(in crate::service) fn storage_transfer_deadline(&self) -> Option<u64> {
        if self.upstream || self.storage_wait().is_none() {
            return None;
        }
        self.storage_links[self.storage_pending.as_ref()?.source]
            .as_ref()?
            .deadline()
    }
    pub(in crate::service) fn storing(&self) -> bool {
        #[cfg(feature = "interaction-delay")]
        if self.hold_storage_active {
            // The test-only probe holds this matrix's storage as a slow
            // provider would, exchange or none.
            return true;
        }
        self.storage_pending.is_some() || self.binding_pending.is_some()
    }
    /// The requesting child, if any; owner launches and local recovery have none.
    pub(in crate::service) fn storing_client(&self) -> Option<usize> {
        if self.binding_pending.is_some() {
            return self.binding_client();
        }
        self.storage_pending
            .as_ref()
            .and_then(|p| p.request.and_then(|r| r.client))
    }
    pub(in crate::service) fn upstream_busy(&self) -> bool {
        self.refreshing()
            || self.publishing()
            || (self.upstream && !self.separate_storage && self.storing())
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
        #[cfg(any(feature = "recovery-delay", feature = "launch-delay"))]
        if p.receive.is_some()
            && let Some(until) = p.hold_until
        {
            if !time::reached(time::now()?, until) {
                self.storage_pending = Some(p);
                return Ok(None);
            }
            p.hold_until = None;
            time::storage_sample(
                if matches!(p.launch, Some((_, LaunchOrigin::Recovery(_)))) {
                    "launcher-read-released"
                } else {
                    "launch-read-released"
                },
                &[],
            );
        }
        let mut reply: Option<(wire::Packet, Option<Object>)> = None;
        if let Some(receive) = p.receive {
            let response = if self.upstream {
                cathedral_user_runtime::matrix::upstream::exchange::poll(receive, p.deadline)
            } else {
                self.storage_links[p.source]
                    .as_mut()
                    .ok_or(Error(abi::DENIED as i64))?
                    .poll(p.deadline)
            };
            match response {
                // The parent may have regranted this matrix's binding since it
                // was learned. Learn the current revision once; a child's
                // refusal stands (its attachment already travelled), while
                // this matrix's own program read repeats its step.
                Ok(Some((words, object)))
                    if self.upstream
                        && p.step != Step::Refresh
                        && !p.refreshed
                        && words[0] == abi::DENIED =>
                {
                    drop(object);
                    p.refreshed = true;
                    if p.launch.is_some() {
                        p.resume = Some(p.step);
                    } else {
                        p.denied = Some(words);
                    }
                    p.step = Step::Refresh;
                    p.receive = None;
                }
                Ok(Some((words, object))) => match p.step {
                    Step::Refresh => {
                        drop(object);
                        self.adopt_upstream(words);
                        if let Some(denied) = p.denied.take() {
                            reply = Some((denied, None));
                        } else {
                            p.step = p.resume.take().unwrap_or(Step::Forward);
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
                    Ok(receive) => {
                        p.receive = Some(receive);
                        // The next hop is runnable now; the rest of this turn
                        // can wait for it rather than the other way round.
                        cathedral_user_runtime::task::yield_now();
                    }
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
        if let Some((_, origin)) = p.launch {
            let result = match (words[0], object) {
                (0, Some(object)) => Ok(object),
                (0, None) => Err(abi::IO_ERROR),
                (status, object) => {
                    drop(object);
                    Err(status)
                }
            };
            return Ok(Some(Launched { origin, result }));
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
        // The answered child is runnable now; let it take the turn.
        cathedral_user_runtime::task::yield_now();
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
        if let Some((plan, _)) = p.launch {
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
        if self.upstream {
            words[0] = op | (self.storage_epoch << wire::STORAGE_SCOPE_BITS);
        } else {
            words[0] = op - wire::STORAGE_BASE;
        }
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
        p.receive = Some(if self.upstream {
            let pair = self.storage_pair.ok_or(Error(abi::PEER_CLOSED as i64))?;
            match attachment {
                Some(object) => pair.0.send_memory(&wire::encode(words), &object)?,
                None => pair.0.send(&wire::encode(words))?,
            }
            pair.1
        } else {
            self.storage_links[p.source]
                .as_mut()
                .ok_or(Error(abi::DENIED as i64))?
                .begin(words, attachment.as_ref(), p.deadline)?
        });
        Ok(None)
    }
}
