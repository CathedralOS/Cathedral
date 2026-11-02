//! F2 holds one real forwarded PUBLISH until the child answers HEALTH and polls
//! its interaction connection. The exact sealed input stays in the retained Event.
//! This is a Rust fixture parent, never a probe in the shipped Omega program.
use super::children::Children;
use super::*;
use cathedral_platform_contracts::{input, storage};
use cathedral_user_runtime::time;

#[derive(Default)]
pub(super) struct Gate {
    armed: bool,
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
        self.pending.is_some()
    }

    pub fn take(
        &mut self,
        server: &Server,
        children: &mut Children,
    ) -> Result<Option<Event>, Error> {
        if !self.active() {
            return Ok(None);
        }
        let now = time::now()?;
        assert_eq!(server.incarnation(self.child), Some(self.incarnation));
        assert!(
            !time::reached(now, self.until),
            "Omega did not service HEALTH and input while storage was held"
        );
        if !self.healthy {
            let question = (!self.asked).then_some([wire::HEALTH, 0, 0, 0, 0, 0, 0, 0]);
            if let Some(reply) =
                children.poll_control_probe(self.child, question, now, self.until)?
            {
                assert_eq!(reply, [0; 8]);
                self.healthy = true;
                time::storage_sample("storage-health-during-hold", &[]);
            }
            self.asked = true;
        }
        if self.healthy && self.polled {
            time::storage_sample("storage-delay-released", &[]);
            return Ok(self.pending.take());
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
                self.armed = true;
                time::storage_sample("storage-delay-armed", &[]);
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
        if self.armed
            && (words[0] & 0xffff) == wire::STORAGE_BASE + storage::PUBLISH
            && request.client.is_some_and(|c| parent.storage_stream(c))
        {
            assert!(
                matches!(&event, Event::Memory(_, _)),
                "PUBLISH carries a sealed input"
            );
            self.child = parent.storage_slot(request.client.unwrap());
            self.incarnation = server.incarnation(self.child).expect("live child");
            self.until = time::after(30)?;
            self.asked = false;
            self.healthy = false;
            self.polled = false;
            self.armed = false;
            self.pending = Some(event);
            world.notify(self.child);
            time::storage_sample("storage-delay-held", &[]);
            return Ok(None);
        }
        Ok(Some(event))
    }
}
