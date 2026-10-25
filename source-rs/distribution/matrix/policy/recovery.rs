//! Omega commands/completions; Rust owns tickets and all IPC/image/task custody.
//! Workflow entrance: source/distribution/matrix/recovery/main.omg.
pub use super::operation::{Ticket, Tickets};
pub struct State {
    words: [u64; 2],
}
#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Wait,
    Cancel,
    RetireFallback,
    Read,
    Admit(Ticket),
    Fallback,
}
impl State {
    pub fn configured() -> Self {
        let mut state = Self { words: [0; 2] };
        assert_eq!(state.apply(0, [0; 3]).0, 1);
        state
    }
    pub fn retired(&mut self, failed: bool) -> bool {
        self.fact(1, failed)
    }
    pub fn ready(&self, blocked: bool) -> bool {
        decide(self.words, 2, [blocked.into(), 0, 0]).0 == 1
    }
    pub fn action(&mut self, authority_matches: bool, vacant: bool) -> Action {
        let (action, ticket) = self.apply(3, [authority_matches.into(), vacant.into(), 0]);
        match action {
            0 => Action::Wait,
            1 => Action::Cancel,
            2 => Action::RetireFallback,
            3 => Action::Read,
            4 => {
                assert_ne!(ticket, 0);
                Action::Admit(Ticket(ticket))
            }
            5 => Action::Fallback,
            _ => unreachable!("launcher recovery command"),
        }
    }
    pub fn read_started(&mut self, ticket: Ticket) {
        assert_eq!(self.apply(4, [0, 0, ticket.0]).0, 1);
    }
    pub fn read_unavailable(&mut self) {
        assert!(self.fact(10, false));
    }
    /// False means stale/duplicate: the host must release the supplied attachment.
    pub fn read_completed(&mut self, ticket: Ticket, success: bool) -> bool {
        self.apply(5, [success.into(), 0, ticket.0]).0 == 1
    }
    pub fn admission_finished(&mut self, ticket: Ticket, success: bool) -> bool {
        self.apply(11, [success.into(), 0, ticket.0]).0 == 1
    }
    pub fn fallback_finished(&mut self, success: bool) -> bool {
        self.fact(6, success)
    }
    pub fn request(&mut self, authenticated: bool) -> bool {
        self.fact(7, authenticated)
    }
    pub fn fallback_changed(&mut self, initial: bool) -> bool {
        self.fact(8, initial)
    }
    pub fn explicit_retired(&mut self) {
        assert!(self.fact(9, false));
    }
    fn fact(&mut self, event: u64, fact: bool) -> bool {
        self.apply(event, [fact.into(), 0, 0]).0 == 1
    }
    fn apply(&mut self, event: u64, facts: [u64; 3]) -> (u64, u64) {
        let (result, words, ticket) = decide(self.words, event, facts);
        self.words = words;
        (result, ticket)
    }
}
#[inline(never)]
fn decide(state: [u64; 2], event: u64, facts: [u64; 3]) -> (u64, [u64; 2], u64) {
    let mut p = [0; 128];
    p[0] = 9;
    p[1] = event;
    p[16..18].copy_from_slice(&state);
    p[2..5].copy_from_slice(&facts);
    let action = super::native::decide(&mut p);
    (action, [p[16], p[17]], p[24])
}
