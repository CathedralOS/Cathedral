//! The funding-delay fixture uses F2 to cycle a held FUND through grant, refusal and launcher-scope regrant.
//! The real Omega child must keep control, input and rendering alive throughout.
use super::children::Children;
use super::*;
use cathedral_platform_contracts::input;
use cathedral_user_runtime::time;

#[derive(Default)]
pub(super) struct Gate {
    armed: u8,
    mode: u8,
    pending: Option<Event>,
    child: usize,
    incarnation: u64,
    until: u64,
    asked: bool,
    healthy: bool,
    polled: bool,
    rendered: bool,
    regrant: u8,
    binding: [u64; 2],
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
        if self.regrant == 3 {
            assert!(
                !time::reached(now, self.until),
                "funding regrant did not settle"
            );
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
                time::storage_sample("funding-grant-restored", &[]);
            } else {
                self.asked = true;
            }
            return Ok(None);
        }
        if !self.healthy {
            assert!(
                !time::reached(now, self.until),
                "child did not answer HEALTH during FUND"
            );
            let question = (!self.asked).then_some([wire::HEALTH, 0, 0, 0, 0, 0, 0, 0]);
            if let Some(reply) =
                children.poll_control_probe(self.child, question, now, self.until)?
            {
                assert_eq!(reply, [0; 8]);
                self.healthy = true;
                self.asked = false;
                time::storage_sample("funding-health-during-hold", &[]);
            } else {
                self.asked = true;
            }
        }
        if self.healthy && self.polled && matches!(self.regrant, 1 | 2) {
            let op = if self.regrant == 1 {
                wire::STORAGE_BINDING
            } else {
                wire::STORAGE_REVOKE
            };
            let question = (!self.asked).then_some([op, 0, 0, 0, 0, 0, 0, 0]);
            if let Some(reply) =
                children.poll_control_probe(self.child, question, now, self.until)?
            {
                assert_eq!(reply[0], 0);
                self.asked = false;
                if self.regrant == 1 {
                    self.binding = [reply[1], reply[2]];
                    self.regrant = 2;
                } else {
                    self.regrant = 4;
                    time::storage_sample("funding-grant-revoked", &[]);
                }
            } else {
                self.asked = true;
            }
        }
        if !time::reached(now, self.until) {
            return Ok(None);
        }
        assert!(
            self.healthy && self.polled && self.rendered,
            "FUND blocked independent work"
        );
        assert!(!matches!(self.regrant, 1 | 2), "FUND blocked revocation");
        let event = self.pending.take().unwrap();
        if self.mode == 2 {
            let Event::Request(request) = &event else {
                panic!("FUND has no attachment")
            };
            match server.reply(
                request,
                &wire::encode([abi::NO_MEMORY, 0, 0, 0, 0, 0, 0, 0]),
            ) {
                Ok(()) => {
                    time::storage_sample("funding-refused", &[]);
                    return Ok(None);
                }
                Err(error) if error.0 == abi::WOULD_BLOCK as i64 => {
                    self.pending = Some(event);
                    return Ok(None);
                }
                Err(error) => return Err(error),
            }
        }
        if self.regrant == 4 {
            self.regrant = 3;
            self.until = time::after(2 * wire::STORAGE_DEADLINE_TICKS)?;
        }
        time::storage_sample("funding-released", &[]);
        Ok(Some(event))
    }
    pub fn intercept(&mut self, event: Event, server: &mut Server) -> Result<Option<Event>, Error> {
        if !cfg!(feature = "funding-delay") {
            return Ok(Some(event));
        }
        let request = match &event {
            Event::Request(r) | Event::Memory(r, _) => r,
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
                self.armed = self.armed % 3 + 1;
                time::storage_sample(
                    match self.armed {
                        1 => "funding-hold-armed",
                        2 => "funding-refusal-armed",
                        _ => "funding-regrant-armed",
                    },
                    &[],
                );
            }
            server.reply(request, &wire::encode([0; 8]))?;
            return Ok(None);
        }
        if self.pending.is_some() && request.client == Some(self.child) {
            if !self.polled && matches!(words[0], wire::POLL | wire::POLL_TAGGED) {
                self.polled = true;
                time::storage_sample("funding-poll-during-hold", &[]);
            }
            if !self.rendered && matches!(words[0], wire::SURFACE_IMAGE | wire::PRESENT_COMMIT) {
                self.rendered = true;
                time::storage_sample("funding-render-during-hold", &[]);
            }
        }
        if self.armed != 0 && words[0] == wire::FUND && request.client.is_some_and(|c| c < CHILDREN)
        {
            self.mode = self.armed;
            self.armed = 0;
            self.child = request.client.unwrap();
            self.incarnation = request.incarnation;
            self.until = time::after(500)?;
            self.asked = false;
            self.healthy = false;
            self.polled = false;
            self.rendered = false;
            self.regrant = if self.mode == 3 { 1 } else { 0 };
            self.pending = Some(event);
            time::storage_sample("funding-held", &[]);
            return Ok(None);
        }
        Ok(Some(event))
    }
}
