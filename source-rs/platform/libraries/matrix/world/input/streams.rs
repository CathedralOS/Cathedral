//! One independently routed stream. IDs are scoped to the supplying connection.
use super::Name;
#[derive(Clone, Copy)]
pub(super) struct Stream {
    pub id: Name,
    pub group: usize,
    pub retiring: bool,
    pub description: cathedral_platform_contracts::input::description::Description,
    pub description_locked: bool,
    pub capture: Option<usize>,
    pub point: Option<(u64, u64)>,
    pub cursor_cycle: u64,
    pub buttons: u64,
    pub blocked: bool,
    pub held: u64,
    pub suppressed: u64,
}
impl Stream {
    pub const fn empty() -> Self {
        Self {
            id: Name::EMPTY,
            group: 0,
            retiring: false,
            description: cathedral_platform_contracts::input::description::Description::UNKNOWN,
            description_locked: false,
            capture: None,
            point: None,
            cursor_cycle: 1,
            buttons: 0,
            blocked: false,
            held: 0,
            suppressed: 0,
        }
    }
    /// Interrupt delivery without erasing the nested world's remembered selection.
    pub fn cancel(&mut self) {
        self.point = None;
        if self.cursor_cycle != 0 {
            self.cursor_cycle = self.cursor_cycle.checked_add(1).unwrap_or(0);
        }
        self.capture = None;
        self.suppressed |= self.held;
        self.blocked |= self.buttons != 0;
        self.buttons = 0;
    }
    pub fn forget(&mut self, child: usize) {
        if self.capture == Some(child) {
            self.capture = None;
            self.blocked |= self.buttons != 0;
            self.buttons = 0;
        }
    }
}
