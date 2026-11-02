//! While the parent holds scene work, ask LOSS then STOP through real control IPC.
//! The request stays held until its normal deadline; STOP must settle afterward.
use super::children::Children;
use super::*;
use cathedral_user_runtime::time;

#[derive(Clone, Copy)]
enum Stage {
    Loss,
    Stop,
    Recover,
}

pub(super) struct Cancellation {
    slot: usize,
    incarnation: u64,
    until: u64,
    stage: Stage,
    hops: usize,
    next_probe: u64,
    asked: bool,
}
impl Cancellation {
    pub fn new(slot: usize, incarnation: u64, held_until: u64, hops: usize) -> Self {
        Self {
            slot,
            incarnation,
            until: held_until.wrapping_add(300),
            stage: Stage::Loss,
            hops,
            next_probe: 0,
            asked: false,
        }
    }
    pub fn step(
        &mut self,
        server: &Server,
        children: &mut Children,
        now: u64,
        held: bool,
    ) -> Result<bool, Error> {
        assert_eq!(server.incarnation(self.slot), Some(self.incarnation));
        assert!(
            !time::reached(now, self.until),
            "owner cancellation timed out"
        );
        if matches!(self.stage, Stage::Recover) && (held || !time::reached(now, self.next_probe)) {
            return Ok(false);
        }
        let request = if self.asked {
            None
        } else {
            let mut words = match self.stage {
                Stage::Loss => {
                    assert!(held, "LOSS must start before scene release");
                    [wire::LOSS, 0, 0, 0, 0, 0, 0, 0]
                }
                Stage::Stop => {
                    assert!(held, "STOP must start before scene release");
                    time::storage_sample("selection-stop-requested", &[self.hops as u64]);
                    [wire::STOP, 1, 0, 0, 0, 0, 0, 0]
                }
                // A nonzero binding response cannot be confused with STOP's zero
                // acknowledgement. Notes still owns slot 2 and its storage scope.
                Stage::Recover => [wire::STORAGE_BINDING, 2, 0, 0, 0, 0, 0, 0],
            };
            for _ in 0..self.hops {
                words = wire::route(words, 1).unwrap();
            }
            Some(words)
        };
        let reply = children.poll_control_probe(self.slot, request, now, self.until)?;
        self.asked = true;
        let Some(reply) = reply else { return Ok(false) };
        match self.stage {
            Stage::Loss => {
                assert!(held, "LOSS must answer during held scene work");
                assert_eq!(reply, wire::error(abi::BUSY));
                time::storage_sample("selection-loss-busy", &[self.hops as u64]);
                self.stage = Stage::Stop;
                self.asked = false;
                Ok(false)
            }
            Stage::Stop if self.hops != 0 => {
                assert!(held, "forwarding deadline must expire during the hold");
                assert_eq!(reply, wire::error(abi::TIMED_OUT));
                time::storage_sample("selection-stop-timed-out", &[self.hops as u64]);
                // A deadline is not a cancellation. Never resubmit STOP: let the
                // accepted operation settle, then exercise the same control chain.
                self.stage = Stage::Recover;
                self.asked = false;
                self.next_probe = now;
                Ok(false)
            }
            Stage::Stop => {
                assert!(
                    !held,
                    "STOP acknowledged before retained scene work settled"
                );
                assert_eq!(reply, [0; 8]);
                time::storage_sample("selection-stop-acknowledged", &[]);
                Ok(true)
            }
            Stage::Recover if reply[0] == abi::BUSY || reply[0] == abi::TIMED_OUT => {
                self.asked = false;
                self.next_probe = now.wrapping_add(5);
                Ok(false)
            }
            Stage::Recover => {
                assert_eq!(reply[0], 0);
                assert!(reply[1] < 2 && reply[2] != 0);
                assert_eq!(reply[3..], [0; 5]);
                time::storage_sample("selection-control-recovered", &[self.hops as u64]);
                Ok(true)
            }
        }
    }
}
