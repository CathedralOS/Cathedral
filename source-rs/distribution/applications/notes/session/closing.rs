//! App-local close interaction. Only an explicit discard abandons a draft.
use super::wire;

#[derive(Default)]
pub struct Controls {
    pub open: bool,
    pointers: cathedral_matrix::reception::Pointers,
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
        self.pointers.reset();
    }
    pub fn action(&mut self, event: wire::Packet, width: u64, temporary: bool) -> Action {
        let clicked = self
            .pointers
            .update(event)
            .is_some_and(|b| b.left_pressed());
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
