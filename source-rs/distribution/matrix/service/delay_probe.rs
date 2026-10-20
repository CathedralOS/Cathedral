//! Test-only physical F8/F7/F6 gates: hold one poll/image/commit for five
//! seconds while the root keeps dispatching. Never present in ordinary F5.
use super::*;
use cathedral_platform_contracts::input;
use cathedral_user_runtime::time;
#[derive(Default)]
pub(super) struct Gate {
    armed: u64,
    pending: Option<(Event, u64)>,
}
impl Gate {
    pub fn deadline(&self) -> Option<u64> {
        self.pending.as_ref().map(|p| p.1)
    }
    pub fn take(&mut self, server: &Server) -> Result<Option<Event>, Error> {
        let now = time::now()?;
        if self.deadline().is_none_or(|d| !time::reached(now, d)) {
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
            && [input::F8 as u64, input::F7 as u64, input::F6 as u64].contains(&key)
        {
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
