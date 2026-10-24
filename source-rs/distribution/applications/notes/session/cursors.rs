//! F6 toggles only pointers in the triggering keyboard's local input group.
//! Aliases and preferences expire with the view or a stream cancellation.
use super::*;
#[derive(Default)]
pub struct Cursors {
    epoch: u64,
    pointers: [(u64, u64, bool); wire::INPUT_STREAMS],
}
impl Cursors {
    pub fn observe(&mut self, parent: (Handle, Handle), event: wire::Packet) {
        if self.epoch != event[1] {
            self.epoch = event[1];
            self.pointers.fill((0, 0, false));
        }
        let channel = event[6];
        match event[2] {
            wire::POINTER if channel != 0 => {
                if !self.pointers.iter().any(|p| p.0 == channel)
                    && let Some(p) = self.pointers.iter_mut().find(|p| p.0 == 0)
                {
                    *p = (channel, event[7], false);
                }
            }
            wire::CANCEL | wire::RETIRED => {
                for p in &mut self.pointers {
                    if p.0 == channel {
                        *p = (0, 0, false);
                    }
                }
            }
            wire::KEY if event[3] == input::F6 as u64 && event[4] == input::PRESS as u64 => {
                for p in &mut self.pointers {
                    if p.0 == 0 || p.1 != event[7] {
                        continue;
                    }
                    let mode = if p.2 {
                        wire::CURSOR_DEFAULT
                    } else {
                        wire::CURSOR_HIDDEN
                    };
                    let reply = call(
                        parent,
                        [wire::CURSOR_PREFERENCE, self.epoch, p.0, mode, 0, 0, 0, 0],
                    );
                    if reply[0] == 0 {
                        p.2 = !p.2;
                        time::storage_sample("note-cursor", &[p.0, mode]);
                    }
                }
            }
            _ => (),
        }
    }
}
