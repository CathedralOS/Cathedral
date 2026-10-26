//! Disposable loss reply held past the owner's deadline while input stays live.
use cathedral_user_runtime::{server::OwnerControl, services::Wake, time};
#[derive(Default)]
pub(super) struct Probe {
    used: bool,
    held: Option<([u8; 64], u64)>,
}
impl Probe {
    pub fn hold(&mut self, reply: [u8; 64]) -> bool {
        if self.used {
            return false;
        }
        self.used = true;
        self.held = Some((reply, time::after(400).unwrap()));
        cathedral_user_runtime::write(b"Cathedral control probe: holding original loss answer\n")
            .unwrap();
        true
    }
    pub fn progress(&mut self, control: &mut OwnerControl, wake: Wake) -> bool {
        let Some((reply, until)) = self.held else {
            return false;
        };
        if time::reached(time::now().unwrap(), until) {
            control.reply(&reply).unwrap();
            self.held = None;
            cathedral_user_runtime::write(
                b"Cathedral control probe: released original loss answer\n",
            )
            .unwrap();
            return false;
        }
        match cathedral_user_runtime::event::wait(&[wake.interest()], until) {
            Ok(_) => {
                wake.take().unwrap();
            }
            Err(error) => assert_eq!(error.0, cathedral_kernel_contracts::user::TIMED_OUT as i64),
        }
        true
    }
}
