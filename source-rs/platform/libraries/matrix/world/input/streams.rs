//! One independently routed stream. IDs are scoped to the supplying connection.
#[derive(Clone, Copy)]
pub(super) struct Stream {
    pub id: u32,
    pub focus: Option<usize>,
    pub capture: Option<usize>,
    pub buttons: u64,
    pub blocked: bool,
    pub held: u64,
    pub suppressed: u64,
}
impl Stream {
    pub const fn empty() -> Self {
        Self {
            id: 0,
            focus: None,
            capture: None,
            buttons: 0,
            blocked: false,
            held: 0,
            suppressed: 0,
        }
    }
    /// Interrupt delivery without erasing the nested world's remembered selection.
    pub fn cancel(&mut self) {
        self.capture = None;
        self.suppressed |= self.held;
        self.blocked |= self.buttons != 0;
        self.buttons = 0;
    }
    pub fn forget(&mut self, child: usize) {
        if self.focus == Some(child) {
            self.focus = None;
            self.suppressed |= self.held;
        }
        if self.capture == Some(child) {
            self.capture = None;
            self.blocked |= self.buttons != 0;
            self.buttons = 0;
        }
    }
}
