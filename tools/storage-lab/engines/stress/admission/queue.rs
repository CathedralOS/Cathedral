//! FIFO per assigned producer, alternating when both are ready. Dequeue transfers
//! the reservation to the active writer; it does not release the charge.
use super::{
    Request,
    budget::{Budget, Denied},
};
use std::{
    collections::VecDeque,
    sync::{Condvar, Mutex},
    time::{Duration, Instant},
};

struct State {
    ready: [VecDeque<Request>; 2],
    next: usize,
    closed: bool,
}
pub struct Queue {
    budget: Budget,
    state: Mutex<State>,
    changed: Condvar,
}
impl Queue {
    pub fn new(budget: Budget) -> Self {
        Self {
            budget,
            state: Mutex::new(State {
                ready: [VecDeque::new(), VecDeque::new()],
                next: 0,
                closed: false,
            }),
            changed: Condvar::new(),
        }
    }
    pub fn push(&self, mut request: Request) -> std::result::Result<(), Denied> {
        if !self.budget.owns(&request.reservation) {
            return Err(Denied::Mismatch);
        }
        let mut state = self.state.lock().unwrap();
        if state.closed {
            return Err(Denied::Closed);
        }
        request.queued = Instant::now();
        let lane = request.reservation.client().index();
        state.ready[lane].push_back(request);
        drop(state);
        self.changed.notify_one();
        Ok(())
    }
    pub fn pop(&self, timeout: Duration) -> std::result::Result<Request, Denied> {
        let deadline = Instant::now() + timeout;
        let mut state = self.state.lock().unwrap();
        loop {
            if state.closed {
                return Err(Denied::Closed);
            }
            for lane in [state.next, 1 - state.next] {
                if let Some(request) = state.ready[lane].pop_front() {
                    state.next = 1 - lane;
                    return Ok(request);
                }
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(Denied::Timeout);
            }
            state = self.changed.wait_timeout(state, remaining).unwrap().0;
        }
    }
    pub fn close(&self) {
        let ready = {
            let mut state = self.state.lock().unwrap();
            state.closed = true;
            std::mem::take(&mut state.ready)
        };
        drop(ready); // Cancel queued requests and release their reservations.
        self.changed.notify_all();
    }
}
