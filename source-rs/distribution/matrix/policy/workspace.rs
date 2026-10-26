//! Private native adapter to distribution/matrix/workspace/main.omg.
use super::operation::Ticket;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum Command {
    Begin = 1,
    Wait,
    Activate,
    Restore,
    WaitRestore,
    Finish,
    Failed,
}
pub struct State {
    words: [u64; 3],
}
impl State {
    pub fn new(ticket: Ticket) -> Self {
        let mut s = Self { words: [0; 3] };
        assert_eq!(s.apply(0, [ticket.raw(), 0, 0, 0]), 1);
        s
    }
    pub fn command(&mut self) -> Option<Command> {
        Some(match self.apply(1, [0; 4]) {
            0 => return None,
            1 => Command::Begin,
            2 => Command::Wait,
            3 => Command::Activate,
            4 => Command::Restore,
            5 => Command::WaitRestore,
            6 => Command::Finish,
            7 => Command::Failed,
            _ => unreachable!(),
        })
    }
    pub fn complete(
        &mut self,
        ticket: Ticket,
        command: Command,
        status: u64,
        committed: bool,
    ) -> bool {
        self.apply(2, [ticket.raw(), command as u64, status, committed.into()]) == 1
    }
    pub fn cancel(&mut self, status: u64) -> bool {
        self.apply(3, [status, 0, 0, 0]) == 1
    }
    pub fn status(&self) -> u64 {
        self.words[2]
    }
    #[inline(never)]
    fn apply(&mut self, event: u64, facts: [u64; 4]) -> u64 {
        let mut p = [0; 128];
        p[0] = 12;
        p[1] = event;
        p[2..6].copy_from_slice(&facts);
        p[16..19].copy_from_slice(&self.words);
        let result = super::native::decide(&mut p);
        self.words.copy_from_slice(&p[16..19]);
        result
    }
}
