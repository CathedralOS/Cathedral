//! Charges bound logical payload plus fixed per-object overhead, not allocator RSS
//! or engine caches. Reservations outlive queue removal and release on every exit.
use super::Client;
use std::{
    fmt,
    sync::{Arc, Condvar, Mutex},
    time::{Duration, Instant},
};

pub const OBJECT_CHARGE: usize = 256;

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub bytes: usize,
    pub requests: usize,
    pub client_bytes: usize,
    pub client_requests: usize,
    pub transaction_bytes: usize,
    pub transaction_objects: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            bytes: 512 * 1024,
            requests: 16,
            client_bytes: 256 * 1024,
            client_requests: 8,
            transaction_bytes: 128 * 1024,
            transaction_objects: 8,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Denied {
    TooLarge,
    Busy,
    Closed,
    Mismatch,
    Timeout,
}
impl fmt::Display for Denied {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "admission {self:?}")
    }
}
impl std::error::Error for Denied {}

#[derive(Clone, Copy, Debug, Default)]
pub struct Usage {
    pub bytes: usize,
    pub requests: usize,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Snapshot {
    pub current: Usage,
    pub peak: Usage,
    pub clients: [Usage; 2],
    pub client_peaks: [Usage; 2],
    pub closed: bool,
}
#[derive(Debug)]
struct Shared {
    limits: Limits,
    state: Mutex<Snapshot>,
    changed: Condvar,
}
#[derive(Clone, Debug)]
pub struct Budget(Arc<Shared>);
#[derive(Debug)]
pub struct Reservation {
    budget: Budget,
    client: Client,
    bytes: usize,
    objects: usize,
}

fn charge(lengths: &[usize], limits: Limits) -> std::result::Result<usize, Denied> {
    if lengths.is_empty() || lengths.len() > limits.transaction_objects {
        return Err(Denied::TooLarge);
    }
    let bytes = lengths
        .iter()
        .try_fold(0usize, |total, bytes| {
            total.checked_add(*bytes)?.checked_add(OBJECT_CHARGE)
        })
        .ok_or(Denied::TooLarge)?;
    if bytes > limits.transaction_bytes || bytes > limits.client_bytes || bytes > limits.bytes {
        return Err(Denied::TooLarge);
    }
    Ok(bytes)
}

impl Budget {
    pub fn new(limits: Limits) -> Self {
        assert!(limits.requests > 0 && limits.client_requests > 0);
        Self(Arc::new(Shared {
            limits,
            state: Mutex::new(Snapshot::default()),
            changed: Condvar::new(),
        }))
    }
    pub fn snapshot(&self) -> Snapshot {
        *self.0.state.lock().unwrap()
    }
    pub fn owns(&self, reservation: &Reservation) -> bool {
        Arc::ptr_eq(&self.0, &reservation.budget.0)
    }
    pub fn close(&self) {
        self.0.state.lock().unwrap().closed = true;
        self.0.changed.notify_all();
    }
    pub fn reserve(
        &self,
        client: Client,
        lengths: &[usize],
    ) -> std::result::Result<Reservation, Denied> {
        let bytes = charge(lengths, self.0.limits)?;
        self.acquire(
            &mut self.0.state.lock().unwrap(),
            client,
            bytes,
            lengths.len(),
        )
    }
    pub fn wait(
        &self,
        client: Client,
        lengths: &[usize],
        timeout: Duration,
    ) -> std::result::Result<Reservation, Denied> {
        let bytes = charge(lengths, self.0.limits)?;
        let deadline = Instant::now() + timeout;
        let mut state = self.0.state.lock().unwrap();
        loop {
            match self.acquire(&mut state, client, bytes, lengths.len()) {
                Err(Denied::Busy) => {
                    let remaining = deadline.saturating_duration_since(Instant::now());
                    if remaining.is_zero() {
                        return Err(Denied::Timeout);
                    }
                    state = self.0.changed.wait_timeout(state, remaining).unwrap().0;
                }
                result => return result,
            }
        }
    }
    fn acquire(
        &self,
        state: &mut Snapshot,
        client: Client,
        bytes: usize,
        objects: usize,
    ) -> std::result::Result<Reservation, Denied> {
        if state.closed {
            return Err(Denied::Closed);
        }
        let limits = self.0.limits;
        let own = state.clients[client.index()];
        if bytes > limits.bytes - state.current.bytes
            || bytes > limits.client_bytes - own.bytes
            || state.current.requests >= limits.requests
            || own.requests >= limits.client_requests
        {
            return Err(Denied::Busy);
        }
        state.current.bytes += bytes;
        state.current.requests += 1;
        state.clients[client.index()].bytes += bytes;
        state.clients[client.index()].requests += 1;
        state.peak.bytes = state.peak.bytes.max(state.current.bytes);
        state.peak.requests = state.peak.requests.max(state.current.requests);
        let own = state.clients[client.index()];
        let peak = &mut state.client_peaks[client.index()];
        peak.bytes = peak.bytes.max(own.bytes);
        peak.requests = peak.requests.max(own.requests);
        Ok(Reservation {
            budget: self.clone(),
            client,
            bytes,
            objects,
        })
    }
}
impl Reservation {
    pub fn client(&self) -> Client {
        self.client
    }
    pub fn bytes(&self) -> usize {
        self.bytes
    }
    pub fn validate(&self, lengths: &[usize]) -> std::result::Result<(), Denied> {
        if lengths.len() != self.objects || charge(lengths, self.budget.0.limits)? != self.bytes {
            return Err(Denied::Mismatch);
        }
        Ok(())
    }
}
impl Drop for Reservation {
    fn drop(&mut self) {
        let mut state = self.budget.0.state.lock().unwrap();
        state.current.bytes -= self.bytes;
        state.current.requests -= 1;
        state.clients[self.client.index()].bytes -= self.bytes;
        state.clients[self.client.index()].requests -= 1;
        drop(state);
        self.budget.0.changed.notify_all();
    }
}
