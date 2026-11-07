//! Stop a pending layout's controller, then reinstall a fresh requester through
//! ordinary owner IPC. Neither the old reply nor its layout grant may transfer.
use super::cancellation_probe::Cancellation;
use super::children::Children;
use super::requester_probe::Replacement;
use super::*;
use cathedral_user_runtime::time;

pub(super) struct Controller {
    cancellation: Option<Cancellation>,
    replacement: Replacement,
    until: u64,
}
impl Controller {
    pub fn new(slot: usize, incarnation: u64, held_until: u64, view: wire::Packet) -> Self {
        // The shared launcher fixture addresses this root's user in slot 1.
        assert_eq!(slot, 1);
        Self {
            cancellation: Some(Cancellation::controller(slot, incarnation, held_until)),
            replacement: Replacement::new(view),
            until: 0,
        }
    }
    pub fn step(
        &mut self,
        server: &Server,
        children: &mut Children,
        now: u64,
        held: bool,
    ) -> Result<bool, Error> {
        if let Some(cancellation) = &mut self.cancellation {
            if !cancellation.step(server, children, now, held)? {
                return Ok(false);
            }
            self.cancellation = None;
            // Cancellation has settled. Reinstallation uses its normal deadline.
            self.until = now.wrapping_add(wire::RUN_OBJECT_DEADLINE_TICKS);
            time::storage_sample("layout-controller-stopped", &[]);
        }
        assert!(
            !time::reached(now, self.until),
            "controller replacement timed out"
        );
        if self.replacement.step(children, now, self.until)? {
            time::storage_sample("layout-controller-replaced", &[]);
            return Ok(true);
        }
        Ok(false)
    }
}
