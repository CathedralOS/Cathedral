//! Stock retry state in Omega; Rust keeps all task, endpoint, placement and image custody.
//! Policy entrance: source/distribution/matrix/recovery/main.omg.
pub struct State {
    word: u64,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Wait,
    Cancel,
    RetireFallback,
    Read,
    Admit,
    Fallback,
}
impl State {
    pub fn configured() -> Self {
        let mut state = Self { word: 0 };
        assert!(state.apply(0, false));
        state
    }
    pub fn retired(&mut self, failed: bool) -> bool {
        self.apply(1, failed)
    }
    pub fn ready(&self, blocked: bool) -> bool {
        decide(self.word, 2, [blocked, false, false]).0 == 1
    }
    pub fn action(&self, authority_matches: bool, vacant: bool, result_ready: bool) -> Action {
        match decide(self.word, 3, [authority_matches, vacant, result_ready]).0 {
            0 => Action::Wait,
            1 => Action::Cancel,
            2 => Action::RetireFallback,
            3 => Action::Read,
            4 => Action::Admit,
            5 => Action::Fallback,
            _ => unreachable!("launcher recovery policy action"),
        }
    }
    pub fn read_started(&mut self) {
        assert!(self.apply(4, false));
    }
    pub fn read_finished(&mut self, success: bool) {
        assert!(self.apply(5, success));
    }
    pub fn fallback_finished(&mut self, success: bool) -> bool {
        self.apply(6, success)
    }
    pub fn request(&mut self, authenticated: bool) -> bool {
        self.apply(7, authenticated)
    }
    pub fn fallback_changed(&mut self, initial: bool) -> bool {
        self.apply(8, initial)
    }
    pub fn explicit_retired(&mut self) {
        assert!(self.apply(9, false));
    }
    fn apply(&mut self, event: u64, fact: bool) -> bool {
        let (result, state) = decide(self.word, event, [fact, false, false]);
        self.word = state;
        result == 1
    }
}
#[inline(never)]
fn decide(state: u64, event: u64, facts: [bool; 3]) -> (u64, u64) {
    let mut p = [0; 128];
    p[0] = 9;
    p[1] = event;
    p[16] = state;
    for (word, fact) in p[2..5].iter_mut().zip(facts) {
        *word = u64::from(fact);
    }
    let action = super::native::decide(&mut p);
    (action, p[16])
}
