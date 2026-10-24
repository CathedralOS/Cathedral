// One owner-granted capture lease: authenticate, demand a fresh frame, retain a
// completed result until transfer, then release. This is provider-side policy;
// IPC adapters must derive identities from accepted endpoints, never packet PIDs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Denied,
    Invalid,
    Busy,
    Exhausted,
    Pending,
    Expired,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Participant {
    pub slot: usize,
    pub incarnation: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Source {
    pub participant: Participant,
    pub epoch: u64,
}
#[derive(Clone, Copy)]
struct Grant {
    source: Source,
    receiver: Participant,
    plan: Plan,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Waiting,
    Ready,
    Expired,
}
enum Work<T> {
    Idle,
    Waiting {
        ticket: u64,
        start: u64,
        lifetime: u64,
    },
    Ready {
        ticket: u64,
        payload: T,
    },
    Expired {
        ticket: u64,
    },
}
pub struct Lease<T> {
    grant: Option<Grant>,
    last: u64,
    work: Work<T>,
}
impl<T> Default for Lease<T> {
    fn default() -> Self {
        Self::empty()
    }
}
impl<T> Lease<T> {
    pub const fn empty() -> Self {
        Self {
            grant: None,
            last: 0,
            work: Work::Idle,
        }
    }
    /// Owner operation, not a consumer request. Replacing a grant retires pending
    /// work/results. Tickets never restart, including after revoke/regrant.
    pub fn grant(
        &mut self,
        source: Source,
        receiver: Participant,
        plan: Plan,
    ) -> Result<(), Error> {
        if source.participant.incarnation == 0 || source.epoch == 0 || receiver.incarnation == 0 {
            return Err(Error::Invalid);
        }
        self.work = Work::Idle;
        self.grant = Some(Grant {
            source,
            receiver,
            plan,
        });
        Ok(())
    }
    pub fn revoke(&mut self) {
        self.grant = None;
        self.work = Work::Idle;
    }
    /// Reconcile authoritative endpoint/view lifetime before each dispatch.
    /// A timeout is not death; expiry below releases only this operation.
    pub fn reconcile(&mut self, source: Option<Source>, receiver: Option<Participant>) {
        if self
            .grant
            .is_some_and(|g| Some(g.source) != source || Some(g.receiver) != receiver)
        {
            self.revoke();
        }
    }
    pub fn plan(&self) -> Option<&Plan> {
        self.grant.as_ref().map(|g| &g.plan)
    }
    /// The provider funds/adopts capacity before beginning. Time/lifetime come
    /// from its clock/policy. One request and one retained result per lease.
    pub fn begin(&mut self, receiver: Participant, now: u64, lifetime: u64) -> Result<u64, Error> {
        self.authenticate(receiver)?;
        if lifetime == 0 || lifetime > cathedral_kernel_contracts::user::MAX_INTERVAL {
            return Err(Error::Invalid);
        }
        self.advance(now);
        if matches!(self.work, Work::Waiting { .. } | Work::Ready { .. }) {
            return Err(Error::Busy);
        }
        let ticket = self.last.checked_add(1).ok_or(Error::Exhausted)?;
        self.last = ticket;
        self.work = Work::Waiting {
            ticket,
            start: now,
            lifetime,
        };
        Ok(ticket)
    }
    /// Aggregate these demands per source independently of display visibility.
    /// The provider must wake the source and arrange a frame-ready handshake;
    /// merely observing a publication or the passage of time is insufficient.
    pub fn demand(&self) -> Option<(Source, u64)> {
        match (&self.grant, &self.work) {
            (Some(g), Work::Waiting { ticket, .. }) => Some((g.source, *ticket)),
            _ => None,
        }
    }
    pub fn advance(&mut self, now: u64) {
        if let Work::Waiting {
            ticket,
            start,
            lifetime,
        } = self.work
            && now.wrapping_sub(start) >= lifetime
        {
            self.work = Work::Expired { ticket };
        }
    }
    /// Provider-only commit after a matched source acknowledgment and successful
    /// masked rasterization under stable custody. T owns the completed *masked*
    /// result, never the unmasked source. Rejected payloads are dropped normally.
    pub fn publish(
        &mut self,
        source: Source,
        ticket: u64,
        now: u64,
        payload: T,
    ) -> Result<(), Error> {
        if self.grant.is_none_or(|g| g.source != source) {
            return Err(Error::Denied);
        }
        self.advance(now);
        match self.work {
            Work::Waiting {
                ticket: expected, ..
            } if expected == ticket => {
                self.work = Work::Ready { ticket, payload };
                Ok(())
            }
            Work::Expired { ticket: expected } if expected == ticket => Err(Error::Expired),
            _ => Err(Error::Denied),
        }
    }
    pub fn status(
        &mut self,
        receiver: Participant,
        ticket: u64,
        now: u64,
    ) -> Result<Status, Error> {
        self.authenticate(receiver)?;
        self.advance(now);
        match self.work {
            Work::Waiting { ticket: t, .. } if t == ticket => Ok(Status::Waiting),
            Work::Ready { ticket: t, .. } if t == ticket => Ok(Status::Ready),
            Work::Expired { ticket: t } if t == ticket => Ok(Status::Expired),
            _ => Err(Error::Denied),
        }
    }
    /// Borrow for attempted IPC transfer. A full peer queue keeps the one funded
    /// result here; it never causes re-rendering or mints another allocation.
    pub fn ready(&self, receiver: Participant, ticket: u64) -> Result<&T, Error> {
        self.authenticate(receiver)?;
        match &self.work {
            Work::Ready { ticket: t, payload } if *t == ticket => Ok(payload),
            Work::Waiting { ticket: t, .. } if *t == ticket => Err(Error::Pending),
            Work::Expired { ticket: t } if *t == ticket => Err(Error::Expired),
            _ => Err(Error::Denied),
        }
    }
    /// Cancel or acknowledge a successful transfer; cannot erase a newer request.
    pub fn release(&mut self, receiver: Participant, ticket: u64) -> Result<(), Error> {
        self.authenticate(receiver)?;
        let t = match self.work {
            Work::Idle => return Err(Error::Denied),
            Work::Waiting { ticket, .. }
            | Work::Ready { ticket, .. }
            | Work::Expired { ticket } => ticket,
        };
        if t != ticket {
            return Err(Error::Denied);
        }
        self.work = Work::Idle;
        Ok(())
    }
    fn authenticate(&self, receiver: Participant) -> Result<(), Error> {
        if self.grant.is_some_and(|g| g.receiver == receiver) {
            Ok(())
        } else {
            Err(Error::Denied)
        }
    }
}
