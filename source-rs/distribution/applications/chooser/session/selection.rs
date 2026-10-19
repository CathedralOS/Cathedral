//! Tile and Settings selection from local pointer/keyboard input.
use super::*;
pub struct Selection {
    pub index: usize,
    pub pressed: bool,
}
impl Selection {
    pub fn new() -> Self {
        Self {
            index: 1,
            pressed: false,
        }
    }
    /// Return repaint/activate; hovering alone does not repaint retained tiles.
    pub fn event(
        &mut self,
        event: wire::Packet,
        extent: wire::Packet,
        count: usize,
        picker: bool,
        managed: bool,
    ) -> (bool, bool) {
        match event[2] {
            wire::POINTER => {
                let down = event[5] & 1 != 0;
                let hit = if !picker && event[4] >= extent[3].saturating_sub(32) {
                    Some(SETTINGS)
                } else if managed
                    && (extent[3].saturating_sub(72)..extent[3].saturating_sub(48))
                        .contains(&event[4])
                {
                    Some(END_APP)
                } else {
                    layout::hit(extent, count, picker, event[3], event[4])
                };
                let choose = down && !self.pressed && hit.is_some();
                self.pressed = down;
                if choose {
                    self.index = hit.unwrap();
                }
                (choose, choose)
            }
            wire::CANCEL => {
                self.pressed = false;
                (false, false)
            }
            wire::KEY if event[4] == input::PRESS as u64 => match event[3] as u8 {
                input::UP | input::LEFT => {
                    self.index = layout::step(self.index, count, picker, false);
                    (true, false)
                }
                input::DOWN | input::RIGHT => {
                    self.index = layout::step(self.index, count, picker, true);
                    (true, false)
                }
                input::ENTER => (true, true),
                input::F2 if !picker => {
                    self.index = SETTINGS;
                    (true, true)
                }
                input::F3 if managed => {
                    self.index = END_APP;
                    (true, true)
                }
                _ => (false, false),
            },
            _ => (false, false),
        }
    }
}
