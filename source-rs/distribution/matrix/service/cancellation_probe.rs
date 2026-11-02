//! While the parent holds scene work, ask LOSS then STOP through real control IPC.
//! The request stays held until its normal deadline; STOP must settle afterward.
use super::children::Children;
use super::*;
use cathedral_user_runtime::time;

pub(super) struct Cancellation {
    slot: usize,
    incarnation: u64,
    until: u64,
    stopping: bool,
    asked: bool,
}
impl Cancellation {
    pub fn new(slot: usize, incarnation: u64, held_until: u64) -> Self {
        Self {
            slot,
            incarnation,
            until: held_until.wrapping_add(300),
            stopping: false,
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
        let request = if self.asked {
            None
        } else {
            assert!(held, "cancellation must start before scene release");
            if self.stopping {
                time::storage_sample("selection-stop-requested", &[]);
                Some([wire::STOP, 1, 0, 0, 0, 0, 0, 0])
            } else {
                Some([wire::LOSS, 0, 0, 0, 0, 0, 0, 0])
            }
        };
        let reply = children.poll_control_probe(self.slot, request, now, self.until)?;
        self.asked = true;
        let Some(reply) = reply else { return Ok(false) };
        if self.stopping {
            assert!(
                !held,
                "STOP acknowledged before retained scene work settled"
            );
            assert_eq!(reply, [0; 8]);
            time::storage_sample("selection-stop-acknowledged", &[]);
            Ok(true)
        } else {
            assert!(held, "LOSS must answer during held scene work");
            assert_eq!(reply, wire::error(abi::BUSY));
            time::storage_sample("selection-loss-busy", &[]);
            self.stopping = true;
            self.asked = false;
            Ok(false)
        }
    }
}
