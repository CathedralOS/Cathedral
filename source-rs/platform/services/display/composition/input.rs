//! Logical pointer authority and routing over the compositor's accepted scene.
//! The lab has one physical source and one revocable, app-scoped synthetic source.
mod cursor;
pub mod focus;
mod routing;
#[cfg(test)]
mod tests;
use crate::composition::scene::{Client, Rect};
use cathedral_contracts::{pointer as wire, user as abi};
use wire::Event;

#[derive(Clone, Copy, Default)]
struct Pointer {
    x: u64,
    y: u64,
    buttons: u8,
    press: Option<usize>,
    capture: Option<usize>,
    blocked: bool,
}
#[derive(Clone, Copy)]
struct Lease {
    client: usize,
    incarnation: u64,
    epoch: u64,
}
#[derive(Default)]
struct Queue {
    events: [Event; 8],
    len: usize,
}
impl Queue {
    fn push(&mut self, event: Event) -> bool {
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
    fn pop(&mut self) -> Event {
        if self.len == 0 {
            return Event::default();
        }
        let event = self.events[0];
        self.events.copy_within(1..self.len, 0);
        self.len -= 1;
        event
    }
}
pub struct Router {
    enabled: bool,
    width: u64,
    height: u64,
    pointers: [Pointer; 2],
    queues: [Queue; 3],
    lease: Option<Lease>,
    epoch: u64,
}
impl Router {
    pub fn new(width: u64, height: u64) -> Self {
        assert!(width > 0 && height > 0 && width <= i64::MAX as u64 && height <= i64::MAX as u64);
        Self {
            enabled: false,
            width,
            height,
            pointers: [Pointer {
                x: 40.min(width - 1),
                y: 40.min(height - 1),
                ..Pointer::default()
            }; 2],
            queues: Default::default(),
            lease: None,
            epoch: 0,
        }
    }
    pub fn enable(&mut self, enabled: bool, clients: &[Client; 3]) {
        self.cancel(0, clients);
        self.cancel(1, clients);
        self.lease = None;
        self.enabled = enabled;
    }
    pub fn lease(&mut self, client: usize, grant: bool, clients: &[Client; 3]) -> Result<u64, u64> {
        if client >= 3
            || !self.enabled
            || !clients[client].visible
            || !clients[client].scope.valid()
        {
            return Err(abi::DENIED);
        }
        if !grant && self.lease.is_some_and(|l| l.client != client) {
            return Err(abi::DENIED);
        }
        self.cancel(1, clients);
        self.lease = None;
        if !grant {
            return Ok(0);
        }
        self.epoch = self.epoch.checked_add(1).ok_or(abi::DENIED)?;
        self.lease = Some(Lease {
            client,
            incarnation: clients[client].incarnation,
            epoch: self.epoch,
        });
        self.pointers[1] = Pointer {
            x: clients[client].scope.x + 24,
            y: clients[client].scope.y + 24,
            ..Pointer::default()
        };
        self.clamp_lease(clients);
        Ok(self.epoch)
    }
    pub fn drive(
        &mut self,
        client: usize,
        epoch: u64,
        x: u64,
        y: u64,
        buttons: u8,
        clients: &[Client; 3],
    ) -> Result<(), u64> {
        let lease = self
            .lease
            .filter(|l| {
                l.client == client
                    && l.epoch == epoch
                    && l.incarnation == clients[client].incarnation
                    && clients[client].visible
                    && clients[client].scope.valid()
            })
            .ok_or(abi::DENIED)?;
        let scope = clients[lease.client].scope;
        self.update(
            1,
            scope.x + x.min(scope.w - 1),
            scope.y + y.min(scope.h - 1),
            buttons,
            clients,
        );
        Ok(())
    }
    pub fn device(&mut self, dx: i64, dy: i64, buttons: u8, reset: bool, clients: &[Client; 3]) {
        if !self.enabled {
            return;
        }
        if reset {
            self.cancel(0, clients);
            return;
        }
        let p = self.pointers[0];
        let x = (p.x as i64)
            .saturating_add(dx)
            .clamp(0, self.width as i64 - 1) as u64;
        let y = (p.y as i64)
            .saturating_add(dy)
            .clamp(0, self.height as i64 - 1) as u64;
        self.update(0, x, y, buttons, clients);
    }
    pub fn source_lost(&mut self, clients: &[Client; 3]) {
        self.cancel(0, clients);
    }
    pub fn capture(&mut self, client: usize, id: usize, take: bool) -> Result<(), u64> {
        let p = self.pointers.get_mut(id).ok_or(abi::INVALID_ARGUMENT)?;
        if !self.enabled
            || p.press != Some(client)
            || p.buttons == 0
            || (!take && p.capture != Some(client))
        {
            return Err(abi::DENIED);
        }
        p.capture = if take { Some(client) } else { None };
        Ok(())
    }
    pub fn next(&mut self, client: usize) -> Event {
        self.queues[client].pop()
    }
    pub fn state(&self, id: usize) -> [u64; 6] {
        let p = self.pointers[id];
        [
            0,
            p.x,
            p.y,
            u64::from(p.buttons),
            p.capture.map_or(0, |c| c as u64 + 1),
            if id == 1 {
                self.lease.map_or(0, |l| l.epoch)
            } else {
                0
            },
        ]
    }
    pub fn scope_lost(&mut self, client: usize, clients: &[Client; 3]) {
        for id in 0..2 {
            if self.pointers[id].press == Some(client)
                || (id == 1 && self.lease.is_some_and(|l| l.client == client))
            {
                self.cancel(id, clients);
            }
        }
        if self.lease.is_some_and(|l| l.client == client) {
            self.lease = None;
        }
    }
    pub fn client_lost(&mut self, client: usize, clients: &[Client; 3]) {
        self.scope_lost(client, clients);
        self.queues[client] = Queue::default();
    }
    pub fn moved(&mut self, client: usize, clients: &[Client; 3]) {
        for id in 0..2 {
            if self.pointers[id].press == Some(client) {
                self.cancel(id, clients);
            }
        }
        self.clamp_lease(clients);
    }
    fn clamp_lease(&mut self, clients: &[Client; 3]) {
        if let Some(l) = self.lease {
            let r = clients[l.client].scope;
            self.pointers[1].x = self.pointers[1].x.clamp(r.x, r.x + r.w - 1);
            self.pointers[1].y = self.pointers[1].y.clamp(r.y, r.y + r.h - 1);
        }
    }
}
