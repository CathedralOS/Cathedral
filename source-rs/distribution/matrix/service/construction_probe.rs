//! Owner-side probes for the Omega construction observation window. Only active
//! with construction-probe: F8 asks HEALTH, F7 additionally stops pending slot 1,
//! and F6 revokes then restores slot 1's imports. No kernel test hook is used.
use super::children::Children;
use super::*;
use cathedral_platform_contracts::{imports, input};
use cathedral_user_runtime::time;

#[derive(Default)]
pub(super) struct Gate {
    pending: Option<Probe>,
}
struct Probe {
    incarnation: u64,
    mode: u64,
    stage: u64,
    asked: bool,
    polls: u8,
    until: u64,
}
impl Gate {
    pub fn active(&self) -> bool {
        self.pending.is_some()
    }
    pub fn intercept(&mut self, event: Event, server: &mut Server) -> Result<Option<Event>, Error> {
        if !cfg!(feature = "construction-probe") {
            return Ok(Some(event));
        }
        let request = match &event {
            Event::Request(request) | Event::Memory(request, _) => request,
            _ => return Ok(Some(event)),
        };
        let Some(words) = wire::decode(&request.bytes[..request.len]) else {
            return Ok(Some(event));
        };
        if let Some(probe) = &mut self.pending
            && request.client == Some(1)
            && request.incarnation == probe.incarnation
            && matches!(words[0], wire::POLL | wire::POLL_TAGGED)
            && probe.polls < 2
        {
            // One poll could already be queued when the host sees the hold.
            // The matrix has one parent poll outstanding; a second requires
            // consuming its predecessor's response and issuing new work.
            probe.polls += 1;
            time::storage_sample(
                if probe.polls == 1 {
                    "admission-input-first-poll"
                } else {
                    "admission-input-polled"
                },
                &[],
            );
        }
        if request.client == Some(SOURCE)
            && let [wire::KEY_SOURCE, key, state, 0, 0, 0, 0, 0] = words
            && [input::F8 as u64, input::F7 as u64, input::F6 as u64].contains(&key)
        {
            if state == input::PRESS as u64 {
                assert!(self.pending.is_none(), "overlapping construction probes");
                let mode = if key == input::F8 as u64 {
                    0
                } else if key == input::F7 as u64 {
                    1
                } else {
                    2
                };
                self.pending = Some(Probe {
                    incarnation: server.incarnation(1).expect("user matrix present"),
                    mode,
                    stage: 0,
                    asked: false,
                    polls: 0,
                    until: time::after(200)?,
                });
                time::storage_sample("admission-owner-probe", &[mode]);
            }
            server.reply(request, &wire::encode([0; 8]))?;
            return Ok(None);
        }
        Ok(Some(event))
    }
    pub fn step(&mut self, server: &Server, children: &mut Children) -> Result<(), Error> {
        let Some(probe) = &mut self.pending else {
            return Ok(());
        };
        assert_eq!(server.incarnation(1), Some(probe.incarnation));
        let now = time::now()?;
        assert!(
            !time::reached(now, probe.until),
            "construction owner probe timed out"
        );
        if probe.stage == 3 {
            if probe.polls == 2 {
                time::storage_sample("admission-probe-complete", &[]);
                self.pending = None;
            }
            return Ok(());
        }
        // Prove independent input progress before cancelling the held launch.
        if probe.stage == 1 && probe.polls < 2 {
            return Ok(());
        }
        let words = match probe.stage {
            0 => [wire::HEALTH, 0, 0, 0, 0, 0, 0, 0],
            1 if probe.mode == 1 => [wire::STOP, 1, 0, 0, 0, 0, 0, 0],
            1 => [wire::LAUNCH_IMPORTS, 1, 0, 0, 0, 0, 0, 0],
            _ => [wire::LAUNCH_IMPORTS, 1, imports::ALL, 0, 0, 0, 0, 0],
        };
        let request = (!probe.asked).then_some(words);
        let reply = children.poll_control_probe(1, request, now, probe.until)?;
        probe.asked = true;
        let Some(reply) = reply else { return Ok(()) };
        assert_eq!(reply, [0; 8], "construction owner command refused");
        time::storage_sample(
            match probe.stage {
                0 => "admission-health",
                1 if probe.mode == 1 => "admission-stopped",
                1 => "admission-policy-revoked",
                _ => "admission-policy-restored",
            },
            &[],
        );
        probe.stage = match (probe.stage, probe.mode) {
            (0, 0) | (1, 1) | (2, _) => 3,
            _ => probe.stage + 1,
        };
        probe.asked = false;
        Ok(())
    }
}
