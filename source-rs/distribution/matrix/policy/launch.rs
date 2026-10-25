//! Ordinary launch commands from Omega; all requests/resources remain host-owned.
use super::operation::Ticket;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum Command {
    Read = 1,
    RetirePicker,
    Admit,
    Bind,
    Place,
    Activate,
    Publish,
    Rollback,
    RestorePicker,
    Finish,
}
pub struct State {
    words: [u64; 6],
}
impl State {
    pub fn new(ticket: Ticket, selection: bool, child: bool, desktop: bool) -> Self {
        let mut s = Self { words: [0; 6] };
        assert_eq!(
            s.apply(
                0,
                [
                    ticket.raw(),
                    u64::from(selection) | (u64::from(child) << 1) | (u64::from(desktop) << 2),
                    0,
                    0
                ]
            ),
            1
        );
        s
    }
    pub fn command(&mut self) -> Option<Command> {
        Some(match self.apply(1, [0; 4]) {
            0 => return None,
            1 => Command::Read,
            2 => Command::RetirePicker,
            3 => Command::Admit,
            4 => Command::Bind,
            5 => Command::Place,
            6 => Command::Activate,
            7 => Command::Publish,
            8 => Command::Rollback,
            9 => Command::RestorePicker,
            10 => Command::Finish,
            _ => unreachable!(),
        })
    }
    pub fn complete(
        &mut self,
        ticket: Ticket,
        command: Command,
        status: u64,
        storage: bool,
    ) -> bool {
        self.apply(2, [ticket.raw(), command as u64, status, storage.into()]) == 1
    }
    pub fn cancel(&mut self, code: u64) {
        assert_ne!(code, 0);
        self.apply(3, [code, 0, 0, 0]);
    }
    pub fn accepts(&mut self, ticket: Ticket, command: Command) -> bool {
        self.apply(4, [ticket.raw(), command as u64, 0, 0]) == 1
    }
    pub fn status(&self) -> u64 {
        self.words[3]
    }
    pub fn picker_retired(&self) -> bool {
        self.words[5] != 0
    }
    #[inline(never)]
    fn apply(&mut self, event: u64, facts: [u64; 4]) -> u64 {
        let mut p = [0; 128];
        p[0] = 11;
        p[1] = event;
        p[2..6].copy_from_slice(&facts);
        p[16..22].copy_from_slice(&self.words);
        let result = super::native::decide(&mut p);
        self.words.copy_from_slice(&p[16..22]);
        result
    }
}
