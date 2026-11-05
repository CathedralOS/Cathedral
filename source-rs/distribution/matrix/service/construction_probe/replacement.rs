//! Reinstall the standard launch requester through ordinary owner commands.
//! This is test orchestration, not automatic launcher recovery policy.
use super::super::*;
use super::Children;
use cathedral_user_runtime::time;

enum Step {
    Resolve,
    Pages,
    Run,
    Surfaces,
    Place,
    Done,
}

pub(super) struct Replacement {
    step: Step,
    asked: bool,
    object: u64,
    view: wire::Packet,
}
impl Replacement {
    pub fn new(view: wire::Packet) -> Self {
        assert_eq!(view[0], 0);
        assert!(view[2] >= 32 && view[3] >= 104);
        Self {
            step: Step::Resolve,
            asked: false,
            object: 0,
            view,
        }
    }
    pub fn step(&mut self, children: &mut Children, now: u64, until: u64) -> Result<bool, Error> {
        let words = match self.step {
            Step::Resolve => {
                let mut bytes = wire::encode([wire::RESOLVE_PATH, 0, 0, 0, 0, 0, 0, 0]);
                bytes[8..15].copy_from_slice(b"Desktop");
                wire::decode(&bytes).unwrap()
            }
            // Same 96-page launcher allowance installed by realm_programs, not
            // the older 48-page configuration value.
            Step::Pages => [wire::RUN_PAGES, 0, 96, 4, 0, 0, 0, 0],
            // Scope zero means the matrix's own realm, as in initial configuration.
            Step::Run => [
                wire::RUN_OBJECT,
                0,
                self.object,
                0,
                self.object << 8,
                3,
                0,
                0,
            ],
            Step::Surfaces => [wire::SURFACE_LIMIT, 0, 8, 0, 0, 0, 0, 0],
            Step::Place => [
                wire::PLACE,
                0,
                8,
                self.view[3] - 88,
                self.view[2] - 16,
                80,
                self.view[4],
                0,
            ],
            Step::Done => return Ok(true),
        };
        let reply = children.poll_control_probe(1, (!self.asked).then_some(words), now, until)?;
        self.asked = true;
        let Some(reply) = reply else { return Ok(false) };
        self.asked = false;
        // The former launch must settle before RUN_OBJECT may own another read.
        // Retry only a definite BUSY reply; never repeat accepted or timed-out work.
        if matches!(self.step, Step::Run) && reply == wire::error(abi::BUSY) {
            return Ok(false);
        }
        assert_eq!(
            reply[0], 0,
            "requester replacement command {words:?}: {reply:?}"
        );
        self.step = match self.step {
            Step::Resolve => {
                assert_ne!(reply[1], 0);
                self.object = reply[1];
                Step::Pages
            }
            Step::Pages => Step::Run,
            Step::Run => Step::Surfaces,
            Step::Surfaces => Step::Place,
            Step::Place => {
                time::storage_sample("admission-requester-replaced", &[]);
                Step::Done
            }
            Step::Done => unreachable!(),
        };
        Ok(matches!(self.step, Step::Done))
    }
}
