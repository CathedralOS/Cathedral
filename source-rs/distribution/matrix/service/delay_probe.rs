//! Test-only physical F8/F7/F6/F4 gates: hold one poll/image/commit for five
//! seconds while the root keeps dispatching. F4 travels on: the matrix on the
//! focused chain that receives it from its parent (parent.rs) arms its next
//! close to count its storage as busy for two seconds, exchange or none, and
//! to send that close's LOSS only when the window ends (lifecycle.rs).
//! F5 holds an image and additionally requires an independent HEALTH answer;
//! this stronger probe is used by the Omega continuation test. The original F7
//! fixture also covers the Rust host's synchronous image path. Never present
//! in ordinary F5.
//! F3 holds a full COMMIT with the same liveness probe. Pressing F3 again during
//! that hold refuses the commit, so the client must abort its staged transaction.
//! With loss-query-probe, F5 asks LOSS instead of HEALTH and records the
//! reply for the host to check both dirty and saved children during drawing.
//! F6 during a probed hold asks LOSS, then STOP of the child's Colors slot.
//! F7 during a probed hold routes those requests through two more matrix levels.
use super::cancellation_probe::Cancellation;
use super::children::Children;
use super::*;
use cathedral_platform_contracts::input;
use cathedral_user_runtime::time;
#[derive(Default)]
pub(super) struct Gate {
    construction: super::construction_probe::Gate,
    storage: super::storage_probe::Gate,
    funding: super::funding_probe::Gate,
    armed: u64,
    pending: Option<(Event, u64)>,
    health: Option<(usize, u64, bool)>,
    health_kind: u64,
    polled: bool,
    probe: bool,
    reject_commit: bool,
    abort_expected: Option<(usize, u64, u64)>,
    cancellation: Option<Cancellation>,
}
impl Gate {
    pub fn deadline(&self) -> Option<u64> {
        if self.construction.active()
            || self.health.is_some()
            || self.cancellation.is_some()
            || self.storage.active()
            || self.funding.active()
        {
            time::after(1).ok()
        } else {
            self.pending.as_ref().map(|p| p.1)
        }
    }
    pub fn take(
        &mut self,
        server: &mut Server,
        children: &mut Children,
    ) -> Result<Option<Event>, Error> {
        self.construction.step(server, children)?;
        if let Some(event) = self.funding.take(server, children)? {
            return Ok(Some(event));
        }
        if let Some(event) = self.storage.take(server, children)? {
            return Ok(Some(event));
        }
        let now = time::now()?;
        if let Some(cancellation) = &mut self.cancellation
            && cancellation.step(server, children, now, self.pending.is_some())?
        {
            self.cancellation = None;
        }
        if let Some((slot, incarnation, asked)) = self.health {
            assert_eq!(server.incarnation(slot), Some(incarnation));
            let until = self.pending.as_ref().expect("held request").1;
            assert!(
                !time::reached(now, until),
                "child did not answer its owner while its request was held"
            );
            let loss =
                cfg!(feature = "loss-query-probe") && self.health_kind == wire::SURFACE_IMAGE;
            let op = if loss { wire::LOSS } else { wire::HEALTH };
            let question = (!asked).then_some([op, 0, 0, 0, 0, 0, 0, 0]);
            if let Some(reply) = children.poll_control_probe(slot, question, now, until)? {
                if loss {
                    time::storage_sample("image-loss-during-hold", &reply[..2]);
                } else {
                    assert_eq!(reply[0], 0);
                    time::storage_sample(
                        if self.health_kind == wire::PRESENT_COMMIT {
                            "commit-health-during-hold"
                        } else if self.health_kind == wire::SURFACE_IMAGE {
                            "image-health-during-hold"
                        } else {
                            "poll-health-during-hold"
                        },
                        &[],
                    );
                }
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
        if self.reject_commit {
            let words = wire::decode(&request.bytes[..request.len]).unwrap();
            match server.reply(request, &wire::encode([abi::DENIED, 0, 0, 0, 0, 0, 0, 0])) {
                Ok(()) => {
                    self.abort_expected =
                        Some((request.client.unwrap(), request.incarnation, words[2]));
                    self.reject_commit = false;
                    time::storage_sample("commit-refused", &[]);
                    return Ok(None);
                }
                Err(error) if error.0 == abi::WOULD_BLOCK as i64 => {
                    self.pending = Some((event, now));
                    return Ok(None);
                }
                Err(error) => return Err(error),
            }
        }
        time::storage_sample("interaction-delay-released", &[]);
        Ok(Some(event))
    }
    pub fn intercept(
        &mut self,
        event: Event,
        server: &mut Server,
        world: &mut World,
        parent: &Parent,
    ) -> Result<Option<Event>, Error> {
        let Some(event) = self.construction.intercept(event, server, world)? else {
            return Ok(None);
        };
        let Some(event) = self.funding.intercept(event, server)? else {
            return Ok(None);
        };
        let Some(event) = self.storage.intercept(event, server, parent, world)? else {
            return Ok(None);
        };
        let request = match &event {
            Event::Request(r) | Event::Memory(r, _) => r,
            _ => return Ok(Some(event)),
        };
        let Some(words) = wire::decode(&request.bytes[..request.len]) else {
            return Ok(Some(event));
        };
        if let Some((slot, incarnation, revision)) = self.abort_expected
            && request.client == Some(slot)
            && request.incarnation == incarnation
            && words[0] == wire::PRESENT_ABORT
            && words[2] == revision
        {
            time::storage_sample("commit-abort-received", &[]);
            self.abort_expected = None;
        }
        if !self.polled
            && self.probe
            && words[0] == wire::POLL
            && let Some((Event::Memory(image, _) | Event::Request(image), _)) = &self.pending
            && image.client == request.client
            && image.incarnation == request.incarnation
        {
            time::storage_sample(
                if self.health_kind == wire::PRESENT_COMMIT {
                    "commit-poll-during-hold"
                } else {
                    "image-poll-during-hold"
                },
                &[],
            );
            self.polled = true;
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
                input::F3 as u64,
                input::F5 as u64,
                input::F8 as u64,
                input::F7 as u64,
                input::F6 as u64,
            ]
            .contains(&key)
            {
                if (key == input::F6 as u64 || key == input::F7 as u64)
                    && state == input::PRESS as u64
                    && self.probe
                    && self.health.is_none()
                    && self.cancellation.is_none()
                    && let Some((Event::Request(held) | Event::Memory(held, _), until)) =
                        &self.pending
                {
                    self.cancellation = Some(Cancellation::new(
                        held.client.unwrap(),
                        held.incarnation,
                        *until,
                        if key == input::F7 as u64 { 2 } else { 0 },
                    ));
                }
                if key == input::F3 as u64
                    && state == input::PRESS as u64
                    && self.pending.is_some()
                    && self.health_kind == wire::PRESENT_COMMIT
                {
                    self.reject_commit = true;
                    time::storage_sample("commit-refusal-armed", &[]);
                }
                if state == input::PRESS as u64 && self.pending.is_none() {
                    self.probe = key == input::F5 as u64 || key == input::F3 as u64;
                    self.reject_commit = false;
                    self.armed = if key == input::F3 as u64 {
                        wire::PRESENT_COMMIT
                    } else if key == input::F8 as u64 {
                        wire::POLL_TAGGED
                    } else if key == input::F7 as u64 || key == input::F5 as u64 {
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
            if words[0] == wire::POLL || self.probe {
                self.health = Some((request.client.unwrap(), request.incarnation, false));
            }
            self.health_kind = words[0];
            self.polled = false;
            self.armed = 0;
            self.pending = Some((event, time::after(500)?));
            return Ok(None);
        }
        Ok(Some(event))
    }
}
