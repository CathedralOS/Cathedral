//! One pending action, bound to an explicitly granted live connection.
pub struct Authority {
    grants: [Option<u64>; 2],
    pending: Option<(usize, u64)>,
}
impl Authority {
    pub const fn empty() -> Self {
        Self {
            grants: [None; 2],
            pending: None,
        }
    }
    pub fn grant(&mut self, slot: usize, incarnation: Option<u64>) {
        self.grants[slot] = incarnation;
        if self.pending.is_some_and(|(s, _)| s == slot) {
            self.pending = None;
        }
    }
    pub fn allowed(&self, slot: usize, incarnation: u64) -> bool {
        self.grants
            .get(slot)
            .is_some_and(|grant| *grant == Some(incarnation))
    }
    pub fn request(&mut self, slot: usize, incarnation: u64) -> bool {
        if !self.allowed(slot, incarnation) || self.pending.is_some() {
            return false;
        }
        self.pending = Some((slot, incarnation));
        true
    }
    pub fn take(&mut self, live: impl FnOnce(usize) -> Option<u64>) -> bool {
        self.pending.take().is_some_and(|(slot, incarnation)| {
            self.allowed(slot, incarnation) && live(slot) == Some(incarnation)
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn requests_cannot_cross_revocation_death_or_replacement() {
        let mut a = Authority::empty();
        assert!(!a.request(0, 7));
        a.grant(0, Some(7));
        assert!(!a.request(1, 7));
        assert!(!a.request(0, 8));
        assert!(a.request(0, 7));
        assert!(!a.request(0, 7));
        a.grant(0, None);
        assert!(!a.take(|_| Some(7)));
        a.grant(0, Some(7));
        assert!(a.request(0, 7));
        assert!(!a.take(|_| None));
        assert!(a.request(0, 7));
        assert!(!a.take(|_| Some(8)));
        assert!(a.request(0, 7));
        a.grant(0, Some(8));
        assert!(!a.take(|_| Some(8)));
        assert!(!a.request(0, 7));
        assert!(a.request(0, 8));
        assert!(a.take(|_| Some(8)));
        assert!(!a.take(|_| Some(8)));
    }
}
