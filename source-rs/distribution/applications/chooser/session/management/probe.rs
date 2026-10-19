//! Fixture-only delayed requests verify the provider, independently of the UI.
use super::*;
#[derive(Default)]
pub struct Probe(Option<u64>);
impl Probe {
    pub fn event(&mut self, parent: Pair, event: wire::Packet) {
        if event[2] != wire::KEY || event[4] != input::PRESS as u64 {
            return;
        }
        match event[3] as u8 {
            input::F6 => {
                let reply = call(parent, [wire::MANAGED_TASK, 0, 0, 0, 0, 0, 0, 0]);
                assert_eq!(reply[0], 0);
                assert_ne!(reply[1], 0);
                self.0 = Some(reply[1]);
                cathedral_user_runtime::write(b"Cathedral chooser: management ticket retained\n")
                    .unwrap();
            }
            input::F5 => {
                assert_eq!(
                    call(
                        parent,
                        [wire::TERMINATE_TASK, self.0.unwrap(), 0, 0, 0, 0, 0, 0]
                    ),
                    wire::error(abi::DENIED)
                );
                cathedral_user_runtime::write(
                    b"Cathedral chooser: stale management ticket denied\n",
                )
                .unwrap();
            }
            _ => (),
        }
    }
}
