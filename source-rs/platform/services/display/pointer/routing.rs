//! Target painted content in the same clip/stack order used by scanout.
use super::*;
impl Router {
    fn hit(&self, id: usize, x: u64, y: u64, clients: &[Client; 2]) -> Option<usize> {
        let hit = clients.iter().rposition(|c| c.pixel(x, y).is_some());
        // A synthetic lease cannot target another app, including an app covering it.
        hit.filter(|&c| id == 0 || self.lease.is_some_and(|l| l.client == c))
    }
    pub(super) fn update(&mut self, id: usize, x: u64, y: u64, buttons: u8, clients: &[Client; 2]) {
        let old = self.pointers[id];
        self.pointers[id].x = x;
        self.pointers[id].y = y;
        if old.blocked {
            if buttons == 0 {
                self.pointers[id].blocked = false;
            }
            return;
        }
        let hit = self.hit(id, x, y, clients);
        if old.buttons == 0 && buttons != 0 {
            self.pointers[id].press = hit;
        }
        self.pointers[id].buttons = buttons;
        let press = self.pointers[id].press;
        let target = if old.buttons != 0 || buttons != 0 {
            if self.pointers[id].capture.is_some() || buttons != old.buttons {
                press
            } else {
                press.filter(|c| Some(*c) == hit)
            }
        } else {
            hit
        };
        if let Some(client) = target {
            self.deliver(
                client,
                id,
                if buttons != old.buttons {
                    wire::BUTTONS
                } else {
                    wire::MOTION
                },
                clients,
            );
        }
        if buttons == 0 {
            self.pointers[id].press = None;
            self.pointers[id].capture = None;
        }
    }
    fn event(&self, client: usize, id: usize, kind: u64, clients: &[Client; 2]) -> Event {
        let p = self.pointers[id];
        Event {
            pointer: id as u64,
            kind,
            x: p.x as i64 - clients[client].scope.x as i64,
            y: p.y as i64 - clients[client].scope.y as i64,
            buttons: if kind == wire::CANCEL { 0 } else { p.buttons },
        }
    }
    fn deliver(&mut self, client: usize, id: usize, kind: u64, clients: &[Client; 2]) {
        let event = self.event(client, id, kind, clients);
        if self.queues[client].push(event) {
            return;
        }
        // An unresponsive app gets a bounded cancellation barrier, never a truncated
        // click sequence. Other clients retain their queues and pointer ownership.
        self.queues[client] = Queue::default();
        for pointer in 0..2 {
            if self.pointers[pointer].press == Some(client) {
                self.pointers[pointer].press = None;
                self.pointers[pointer].capture = None;
                self.pointers[pointer].buttons = 0;
                self.pointers[pointer].blocked = true;
            }
            let cancel = self.event(client, pointer, wire::CANCEL, clients);
            self.queues[client].push(cancel);
        }
    }
    pub(super) fn cancel(&mut self, id: usize, clients: &[Client; 2]) {
        let owner = self.pointers[id].press;
        self.pointers[id].buttons = 0;
        self.pointers[id].press = None;
        self.pointers[id].capture = None;
        self.pointers[id].blocked = true;
        for client in 0..2 {
            // Revocation discards undelivered events from the retired binding,
            // including a queued release whose app may already have seen its press.
            let q = &mut self.queues[client];
            let previous = q.len;
            let mut len = 0;
            for index in 0..q.len {
                if q.events[index].pointer != id as u64 {
                    q.events[len] = q.events[index];
                    len += 1;
                }
            }
            q.len = len;
            if owner == Some(client) || previous != len {
                self.deliver(client, id, wire::CANCEL, clients);
            }
        }
    }
}
