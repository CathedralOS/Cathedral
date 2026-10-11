//! Focus/source changes insert cancellation barriers and suppress held-key transfer.
use super::*;
impl Focus {
    pub(super) fn clear(&mut self, client: usize) {
        let token = self.grants[client].map_or(0, |g| g.token);
        self.queues[client] = Queue::default();
        if token != 0 {
            self.queues[client].events[0] = wire::focus_lost(token);
            self.queues[client].len = 1;
        }
    }
    pub(super) fn switch(&mut self, target: Option<usize>) {
        if self.active == target {
            return;
        }
        if let Some(old) = self.active {
            self.clear(old);
        }
        self.suppressed |= self.held;
        self.active = target;
    }
    pub fn source_lost(&mut self) {
        self.switch(None);
        self.suppressed |= self.held;
    }
    pub fn key(&mut self, event: input::Event, tree: &Tree, clients: &[Client; 3]) {
        if event.key == input::RESET {
            self.source_lost();
            return;
        }
        if !(input::UP..=input::ENTER).contains(&event.key) {
            return;
        }
        let bit = 1u64 << (event.key - 1);
        if event.state == input::RELEASE {
            self.held &= !bit;
            if self.suppressed & bit != 0 {
                self.suppressed &= !bit;
                return;
            }
        } else {
            self.held |= bit;
            if self.suppressed & bit != 0 {
                return;
            }
        }
        let Some(client) = self.active else {
            return;
        };
        if !self.valid(client, tree, clients) {
            self.switch(None);
            return;
        }
        let grant = self.grants[client].unwrap();
        if grant.mask & bit == 0 {
            return;
        }
        let q = &mut self.queues[client];
        if q.len == q.events.len() {
            self.clear(client);
            self.suppressed |= self.held;
            return;
        }
        q.events[q.len] = wire::key_event(event.key, event.state, grant.token);
        q.len += 1;
    }
    pub fn next(&mut self, client: usize) -> Option<[u64; 6]> {
        let q = &mut self.queues[client];
        if q.len == 0 {
            return None;
        }
        let event = q.events[0];
        q.events.copy_within(1..q.len, 0);
        q.len -= 1;
        Some(event)
    }
}
