//! F2 arms PUBLISH; pressing it twice arms BULK, three times returns a bad
//! executable after BULK, four times revokes and regrants the launcher scope.
//! Each hold waits until the child answers HEALTH and polls
//! its interaction connection. The exact sealed input stays in the retained Event.
//! This is a Rust fixture parent, never a probe in the shipped Omega program.
use super::children::Children;
use super::*;
use cathedral_platform_contracts::{input, storage};
use cathedral_user_runtime::time;

#[derive(Default)]
pub(super) struct Gate {
    mode: u8,
    invalid_image: bool,
    regrant: u8,
    binding: [u64; 2],
    pending: Option<Event>,
    child: usize,
    incarnation: u64,
    until: u64,
    asked: bool,
    healthy: bool,
    polled: bool,
}

impl Gate {
    pub fn active(&self) -> bool {
        self.pending.is_some() || self.regrant == 3
    }

    pub fn take(
        &mut self,
        server: &mut Server,
        children: &mut Children,
    ) -> Result<Option<Event>, Error> {
        if !self.active() {
            return Ok(None);
        }
        let now = time::now()?;
        assert_eq!(server.incarnation(self.child), Some(self.incarnation));
        assert!(
            !time::reached(now, self.until),
            "Omega storage probe did not settle (regrant phase {})",
            self.regrant
        );
        if self.regrant == 3 {
            let question = (!self.asked).then_some([
                wire::STORAGE_GRANT,
                0,
                self.binding[0],
                self.binding[1],
                0,
                0,
                0,
                0,
            ]);
            if let Some(reply) =
                children.poll_control_probe(self.child, question, now, self.until)?
            {
                assert_eq!(reply, [0; 8]);
                self.regrant = 0;
                self.asked = false;
                time::storage_sample("storage-grant-restored", &[]);
            } else {
                self.asked = true;
            }
            return Ok(None);
        }
        if !self.healthy {
            let question = (!self.asked).then_some([wire::HEALTH, 0, 0, 0, 0, 0, 0, 0]);
            if let Some(reply) =
                children.poll_control_probe(self.child, question, now, self.until)?
            {
                assert_eq!(reply, [0; 8]);
                self.healthy = true;
                self.asked = false;
                time::storage_sample("storage-health-during-hold", &[]);
            } else {
                self.asked = true;
            }
        }
        if self.healthy && self.polled {
            if self.regrant != 0 {
                let opcode = if self.regrant == 1 {
                    wire::STORAGE_BINDING
                } else {
                    wire::STORAGE_REVOKE
                };
                let question = (!self.asked).then_some([opcode, 0, 0, 0, 0, 0, 0, 0]);
                let Some(reply) =
                    children.poll_control_probe(self.child, question, now, self.until)?
                else {
                    self.asked = true;
                    return Ok(None);
                };
                assert_eq!(reply[0], 0);
                self.asked = false;
                if self.regrant == 1 {
                    self.binding = [reply[1], reply[2]];
                    self.regrant = 2;
                    return Ok(None);
                }
                self.regrant = 3;
                // The hold's liveness check has passed. Regrant now waits for
                // the real disk read to drain, then the normal binding query;
                // allow both existing storage deadlines, without extending either.
                self.until = time::after(2 * wire::STORAGE_DEADLINE_TICKS)?;
                time::storage_sample("storage-grant-revoked", &[]);
            }
            time::storage_sample("storage-delay-released", &[]);
            let event = self.pending.take().expect("held storage request");
            if self.invalid_image {
                let Event::Request(request) = event else {
                    panic!("BULK has no input attachment");
                };
                let words = wire::decode(&request.bytes[..request.len]).unwrap();
                let mut page = cathedral_user_runtime::memory::Private::allocate(1)?;
                page.bytes_mut()[..4].copy_from_slice(b"bad!");
                let image = page.freeze()?;
                server.reply_memory(
                    &request,
                    &wire::encode([0, words[1], 1, 0, 4, 0, 0, 0]),
                    &image,
                )?;
                time::storage_sample("storage-invalid-image", &[]);
                return Ok(None);
            }
            return Ok(Some(event));
        }
        Ok(None)
    }

    pub fn intercept(
        &mut self,
        event: Event,
        server: &mut Server,
        parent: &Parent,
        world: &mut World,
    ) -> Result<Option<Event>, Error> {
        let request = match &event {
            Event::Request(request) | Event::Memory(request, _) => request,
            _ => return Ok(Some(event)),
        };
        let Some(words) = wire::decode(&request.bytes[..request.len]) else {
            return Ok(Some(event));
        };
        if request.client == Some(SOURCE)
            && let [wire::KEY_SOURCE, key, state, 0, 0, 0, 0, 0] = words
            && key == input::F2 as u64
        {
            if state == input::PRESS as u64 && !self.active() {
                self.mode = self.mode % 4 + 1;
                time::storage_sample(
                    match self.mode {
                        1 => "storage-delay-armed",
                        2 => "storage-read-armed",
                        3 => "storage-invalid-armed",
                        _ => "storage-regrant-armed",
                    },
                    &[],
                );
            }
            server.reply(request, &wire::encode([0; 8]))?;
            return Ok(None);
        }
        if self.active()
            && !self.polled
            && request.client == Some(self.child)
            && matches!(words[0], wire::POLL | wire::POLL_TAGGED)
        {
            self.polled = true;
            time::storage_sample("storage-poll-during-hold", &[]);
        }
        if self.mode != 0
            && (words[0] & 0xffff)
                == wire::STORAGE_BASE
                    + if self.mode == 1 {
                        storage::PUBLISH
                    } else {
                        storage::BULK
                    }
            && request.client.is_some_and(|c| parent.storage_stream(c))
        {
            if self.mode == 1 {
                assert!(
                    matches!(&event, Event::Memory(_, _)),
                    "PUBLISH carries a sealed input"
                );
            } else {
                assert!(
                    matches!(&event, Event::Request(_)),
                    "BULK has no input attachment"
                );
                time::storage_sample("storage-read-held", &[words[1]]);
            }
            self.child = parent.storage_slot(request.client.unwrap());
            self.incarnation = server.incarnation(self.child).expect("live child");
            self.until = time::after(30)?;
            self.asked = false;
            self.healthy = false;
            self.polled = false;
            self.invalid_image = self.mode == 3;
            self.regrant = if self.mode == 4 { 1 } else { 0 };
            self.mode = 0;
            self.pending = Some(event);
            world.notify(self.child);
            time::storage_sample("storage-delay-held", &[]);
            return Ok(None);
        }
        Ok(Some(event))
    }
}
