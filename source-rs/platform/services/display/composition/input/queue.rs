//! Bounded pointer events: collapse adjacent motion while preserving transitions.
use cathedral_contracts::pointer::{self as wire, Event};

pub(super) struct Queue {
    pub(super) events: [Event; 8],
    pub(super) len: usize,
}
impl Default for Queue {
    fn default() -> Self {
        Self::empty()
    }
}
impl Queue {
    pub(super) const fn empty() -> Self {
        Self {
            events: [Event {
                pointer: 0,
                kind: 0,
                x: 0,
                y: 0,
                buttons: 0,
            }; 8],
            len: 0,
        }
    }
    pub(super) fn push(&mut self, event: Event) -> bool {
        // Motion may collapse; button transitions never silently disappear.
        if self.len > 0 && event.kind == wire::MOTION {
            let last = &mut self.events[self.len - 1];
            if last.kind == wire::MOTION && last.pointer == event.pointer {
                *last = event;
                return true;
            }
        }
        if self.len == self.events.len() {
            return false;
        }
        self.events[self.len] = event;
        self.len += 1;
        true
    }
    pub(super) fn pop(&mut self) -> Event {
        if self.len == 0 {
            return Event::default();
        }
        let event = self.events[0];
        self.events.copy_within(1..self.len, 0);
        self.len -= 1;
        event
    }
}
