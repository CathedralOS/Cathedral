// Explicit force-stop confirmation for the task our owner lets us manage.
// The ticket, not the slot, is captured when opening the warning.
use super::*;
#[derive(Default)]
pub struct Controls {
    current: Option<u64>,
    confirming: Option<u64>,
    pub cancel: bool,
    pub status: &'static str,
}
impl Controls {
    pub fn available(&self) -> bool {
        self.current.is_some()
    }
    pub fn open(&self) -> bool {
        self.confirming.is_some()
    }
    pub fn refresh(&mut self, parent: Pair) -> bool {
        let reply = call(parent, [wire::MANAGED_TASK, 0, 0, 0, 0, 0, 0, 0]);
        let current = (reply[0] == 0 && reply[1] != 0).then_some(reply[1]);
        let changed = current != self.current;
        self.current = current;
        if changed {
            self.confirming = None;
        }
        changed
    }
    pub fn begin(&mut self) {
        self.confirming = self.current;
        self.cancel = true;
        self.status = "";
    }
    pub fn event(
        &mut self,
        event: wire::Packet,
        extent: wire::Packet,
        pressed: &mut bool,
        parent: Pair,
    ) -> bool {
        let activate = match event[2] {
            wire::POINTER => {
                let down = event[5] & 1 != 0;
                let hit = view::hit(extent, event[3], event[4]);
                let activate = down && !*pressed && hit.is_some();
                *pressed = down;
                if activate {
                    self.cancel = hit.unwrap();
                }
                activate
            }
            wire::CANCEL => {
                *pressed = false;
                false
            }
            wire::KEY if event[4] == input::PRESS as u64 => match event[3] as u8 {
                input::UP | input::DOWN | input::LEFT | input::RIGHT => {
                    self.cancel = !self.cancel;
                    return true;
                }
                input::ENTER => true,
                _ => false,
            },
            _ => false,
        };
        if !activate {
            return false;
        }
        if self.cancel {
            self.confirming = None;
            return true;
        }
        let reply = call(
            parent,
            [
                wire::TERMINATE_TASK,
                self.confirming.unwrap(),
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        );
        match reply[0] {
            0 => {
                self.confirming = None;
                self.current = None;
            }
            abi::BUSY => {
                self.status = "TRY AGAIN";
                self.cancel = true;
            }
            _ => {
                self.confirming = None;
                self.current = None;
            }
        }
        true
    }
}
