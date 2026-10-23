//! Test-only physical F8/F7/F6/F4 gates: hold one poll/image/commit for five
//! seconds while the root keeps dispatching, or, on F4 at every matrix the
//! key reaches, that matrix's next storage exchange for two. Never present in
//! ordinary F5.
use super::*;
use cathedral_platform_contracts::input;
use cathedral_user_runtime::time;
#[derive(Default)]
pub(super) struct Gate {
    armed: u64,
    pending: Option<(Event, u64)>,
    /// A storage hold: until when the root leaves its exchange alone, and
    /// whether that has delayed one yet.
    storage: Option<(u64, bool)>,
}
impl Gate {
    pub fn deadline(&self) -> Option<u64> {
        let held = self.pending.as_ref().map(|p| p.1);
        let storage = self.storage.map(|s| s.0);
        match (held, storage) {
            (Some(a), Some(b)) => Some(if time::reached(a, b) { b } else { a }),
            (a, b) => a.or(b),
        }
    }
    /// Hold this matrix's storage exchanges for two seconds from now: longer
    /// than a close answer's deadline, shorter than the shell's wait for one.
    pub fn arm_storage(&mut self) -> Result<(), Error> {
        if self.storage.is_none() {
            self.storage = Some((time::after(200)?, false));
            time::storage_sample("interaction-delay-armed", &[wire::STORAGE_BASE]);
        }
        Ok(())
    }
    /// Whether this matrix must leave its storage exchange alone this turn.
    /// The hold ends on its own deadline; `storing` says an exchange is in
    /// flight, so a close answered meanwhile waits behind it exactly as it
    /// would behind a slow provider.
    pub fn holds_storage(&mut self, storing: bool) -> Result<bool, Error> {
        let Some((until, delayed)) = self.storage else {
            return Ok(false);
        };
        if time::reached(time::now()?, until) {
            self.storage = None;
            time::storage_sample("interaction-delay-released", &[]);
            return Ok(false);
        }
        if storing && !delayed {
            time::storage_sample("interaction-delay-held", &[wire::STORAGE_BASE]);
            self.storage = Some((until, true));
        }
        Ok(true)
    }
    pub fn take(&mut self, server: &Server) -> Result<Option<Event>, Error> {
        let now = time::now()?;
        if self
            .pending
            .as_ref()
            .is_none_or(|p| !time::reached(now, p.1))
        {
            return Ok(None);
        }
        let (event, _) = self.pending.take().unwrap();
        let request = match &event {
            Event::Request(r) | Event::Memory(r, _) => r,
            _ => unreachable!(),
        };
        if request.client.and_then(|c| server.incarnation(c)) != Some(request.incarnation) {
            return Ok(None);
        }
        time::storage_sample("interaction-delay-released", &[]);
        Ok(Some(event))
    }
    pub fn intercept(
        &mut self,
        event: Event,
        server: &mut Server,
        world: &mut World,
    ) -> Result<Option<Event>, Error> {
        let request = match &event {
            Event::Request(r) | Event::Memory(r, _) => r,
            _ => return Ok(Some(event)),
        };
        let Some(words) = wire::decode(&request.bytes[..request.len]) else {
            return Ok(Some(event));
        };
        if request.client == Some(SOURCE)
            && let [wire::KEY_SOURCE, key, state, 0, 0, 0, 0, 0] = words
        {
            if key == input::F4 as u64 {
                // Arm here and let the key travel on: every matrix on the
                // focused chain arms itself when the key reaches it in a poll.
                if state == input::PRESS as u64 {
                    self.arm_storage()?;
                }
                return Ok(Some(event));
            }
            if [input::F8 as u64, input::F7 as u64, input::F6 as u64].contains(&key) {
                if state == input::PRESS as u64 && self.pending.is_none() {
                    self.armed = if key == input::F8 as u64 {
                        wire::POLL_TAGGED
                    } else if key == input::F7 as u64 {
                        wire::SURFACE_IMAGE
                    } else {
                        wire::PRESENT_RECTS
                    };
                    for slot in 0..CHILDREN {
                        world.notify(slot);
                    }
                    time::storage_sample("interaction-delay-armed", &[self.armed]);
                }
                server.reply(request, &wire::encode([0; 8]))?;
                return Ok(None);
            }
        }
        if self.armed != 0 && words[0] == self.armed && request.client.is_some_and(|c| c < CHILDREN)
        {
            if self.armed == wire::POLL_TAGGED {
                // An unrelated refusal must not revoke the live request's view.
                let other = (words[1] as u32).checked_add(1).unwrap();
                server.reply(
                    request,
                    &wire::encode(wire::tag_poll(wire::error(abi::DENIED), other)),
                )?;
            }
            time::storage_sample("interaction-delay-held", &[self.armed]);
            self.armed = 0;
            self.pending = Some((event, time::after(500)?));
            return Ok(None);
        }
        Ok(Some(event))
    }
}
