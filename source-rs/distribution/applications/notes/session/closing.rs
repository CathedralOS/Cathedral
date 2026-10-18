//! App-local close interaction. Only an explicit discard abandons a draft.
use super::wire;

#[derive(Default)]
pub struct Controls {
    pub open: bool,
    pressed: bool,
}
pub enum Action {
    None,
    Request,
    Save,
    Discard,
    Cancel,
}
impl Controls {
    pub fn rebind(&mut self) {
        self.pressed = false;
    }
    pub fn action(&mut self, event: wire::Packet, width: u64, temporary: bool) -> Action {
        if event[2] == wire::CANCEL {
            self.pressed = false;
        }
        if event[2] != wire::POINTER {
            return Action::None;
        }
        let down = event[5] & 1 != 0;
        let clicked = down && !self.pressed;
        self.pressed = down;
        if !clicked {
            return Action::None;
        }
        let (x, y) = (event[3], event[4]);
        if (width.saturating_sub(24)..width.saturating_sub(4)).contains(&x) && (2..18).contains(&y)
        {
            return Action::Request;
        }
        if self.open && (10..190).contains(&x) {
            return match y {
                36..52 if !temporary => Action::Save,
                52..68 => Action::Discard,
                68..84 => Action::Cancel,
                _ => Action::None,
            };
        }
        Action::None
    }
}
