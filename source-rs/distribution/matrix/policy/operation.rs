//! Process-local completion identity shared by stock matrix workflows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ticket(pub(super) u64);
impl Ticket {
    pub fn raw(self) -> u64 {
        self.0
    }
}
/// Process-local names are never recycled, including across launcher replacement.
#[derive(Default)]
pub struct Tickets {
    last: u64,
}
impl Tickets {
    pub fn issue(&mut self) -> Option<Ticket> {
        self.last = self.last.checked_add(1)?;
        Some(Ticket(self.last))
    }
}
#[cfg(test)]
mod ticket_tests {
    use super::*;
    #[test]
    fn exhausted_ticket_space_never_wraps() {
        let mut t = Tickets { last: u64::MAX - 1 };
        assert_eq!(t.issue().unwrap().raw(), u64::MAX);
        assert!(t.issue().is_none());
        assert!(t.issue().is_none());
    }
}
