//! Test-only physical F8/F7/F6/F4 gates: hold one poll/image/commit for five
//! seconds while the root keeps dispatching. F4 travels on: the matrix on the
//! focused chain that receives it from its parent (parent.rs) arms its next
//! close to count its storage as busy for two seconds, exchange or none, and
//! to send that close's LOSS only when the window ends (lifecycle.rs).
//! F5 holds an image and additionally requires an independent HEALTH answer;
//! this stronger probe is used by the Omega continuation test. The original F7
//! fixture also covers the Rust host's synchronous image path. Never present
//! in ordinary F5.
use super::children::Children;
use super::*;
use cathedral_platform_contracts::input;
use cathedral_user_runtime::time;
#[derive(Default)]
pub(super) struct Gate {
    armed: u64,
    pending: Option<(Event, u64)>,
    health: Option<(usize, u64, bool)>,
    health_kind: u64,
    image_polled: bool,
    probe_image: bool,
}
impl Gate {
    pub fn deadline(&self) -> Option<u64> {
        if self.health.is_some() {
            time::after(1).ok()
        } else {
            self.pending.as_ref().map(|p| p.1)
        }
    }
    pub fn take(
        &mut self,
        server: &Server,
        children: &mut Children,
    ) -> Result<Option<Event>, Error> {
        let now = time::now()?;
        if let Some((slot, incarnation, asked)) = self.health {
            assert_eq!(server.incarnation(slot), Some(incarnation));
            let until = self.pending.as_ref().expect("held request").1;
            assert!(
                !time::reached(now, until),
                "child did not answer HEALTH while its request was held"
            );
            if let Some(reply) = children.poll_health_probe(slot, !asked, now, until)? {
                assert_eq!(reply[0], 0);
                time::storage_sample(
                    if self.health_kind == wire::SURFACE_IMAGE {
                        "image-health-during-hold"
                    } else {
                        "poll-health-during-hold"
                    },
                    &[],
                );
                self.health = None;
            } else {
                self.health = Some((slot, incarnation, true));
            }
        }
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
        if !self.image_polled
            && self.probe_image
            && self.health_kind == wire::SURFACE_IMAGE
            && words[0] == wire::POLL
            && let Some((Event::Memory(image, _), _)) = &self.pending
            && image.client == request.client
            && image.incarnation == request.incarnation
        {
            time::storage_sample("image-poll-during-hold", &[]);
            self.image_polled = true;
        }
        if request.client == Some(SOURCE)
            && let [wire::KEY_SOURCE, key, state, 0, 0, 0, 0, 0] = words
        {
            if key == input::F4 as u64 {
                // Not the root's to act on: the key travels to the focused
                // child, and the matrix that receives it from its parent arms.
                return Ok(Some(event));
            }
            if [
                input::F5 as u64,
                input::F8 as u64,
                input::F7 as u64,
                input::F6 as u64,
            ]
            .contains(&key)
            {
                if state == input::PRESS as u64 && self.pending.is_none() {
                    self.probe_image = key == input::F5 as u64;
                    self.armed = if key == input::F8 as u64 {
                        wire::POLL_TAGGED
                    } else if key == input::F7 as u64 || self.probe_image {
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
        if self.armed != 0
            && (words[0] == self.armed
                || (self.armed == wire::POLL_TAGGED && words[0] == wire::POLL))
            && request.client.is_some_and(|c| c < CHILDREN)
        {
            // Retain this exact request without replying. An unrelated tag
            // cannot serve as a provisional response to the same request ID.
            // Plain polls and explicitly selected F5 images also probe the
            // child's independent owner connection before the hold expires.
            time::storage_sample("interaction-delay-held", &[self.armed]);
            if words[0] == wire::POLL || (words[0] == wire::SURFACE_IMAGE && self.probe_image) {
                self.health = Some((request.client.unwrap(), request.incarnation, false));
            }
            self.health_kind = words[0];
            self.image_polled = false;
            self.armed = 0;
            self.pending = Some((event, time::after(500)?));
            return Ok(None);
        }
        Ok(Some(event))
    }
}
