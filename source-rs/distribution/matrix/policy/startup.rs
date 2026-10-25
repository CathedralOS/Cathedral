//! Local startup recovery decisions in source/distribution/matrix/startup/main.omg.
#[derive(Default)]
pub struct State {
    words: [u64; 5],
}
#[derive(Clone, Copy)]
pub enum Role {
    Picker,
    User,
    Background,
    Other,
}
pub struct Context {
    pub login: bool,
    pub entered: bool,
    pub desktop: bool,
    pub owner_recovers: bool,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Picker {
    Wait,
    Restart,
    Exhausted,
}
impl State {
    /// Returns whether a failed selected user must relinquish the entered state.
    pub fn retired(&mut self, role: Role, failed: bool, context: Context) -> bool {
        self.apply(
            0,
            [
                role as u64,
                failed.into(),
                context.login.into(),
                context.entered.into(),
                context.desktop.into(),
                context.owner_recovers.into(),
            ],
        ) == 1
    }
    /// Count an admission attempt and return the stock picker's display mode.
    pub fn picker_started(&mut self) -> u64 {
        self.apply(1, [0; 6])
    }
    /// The host observed failed admission/placement with the picker already gone.
    pub fn selection_failed(&mut self, owner_recovers: bool) {
        self.apply(2, [owner_recovers.into(), 0, 0, 0, 0, 0]);
    }
    /// An explicit workspace configuration failed its initial background admission.
    pub fn background_failed(&mut self) {
        self.apply(3, [0; 6]);
    }
    pub fn take_picker(&mut self, blocked: bool) -> Picker {
        match self.apply(4, [blocked.into(), 0, 0, 0, 0, 0]) {
            0 => Picker::Wait,
            1 => Picker::Restart,
            2 => Picker::Exhausted,
            _ => unreachable!("startup picker action"),
        }
    }
    pub fn take_background(&mut self, blocked: bool) -> bool {
        self.apply(5, [blocked.into(), 0, 0, 0, 0, 0]) == 1
    }
    #[inline(never)]
    fn apply(&mut self, event: u64, facts: [u64; 6]) -> u64 {
        let mut p = [0; 128];
        p[0] = 10;
        p[1] = event;
        p[2..8].copy_from_slice(&facts);
        p[16..21].copy_from_slice(&self.words);
        let result = super::native::decide(&mut p);
        self.words.copy_from_slice(&p[16..21]);
        result
    }
}
