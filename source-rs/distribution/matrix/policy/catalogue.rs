//! Opaque state and value translation for Omega's workspace catalogue.
#[derive(Default)]
pub struct Catalogue {
    words: [u64; 8],
}
#[derive(Debug, PartialEq, Eq)]
pub enum Selection {
    Missing,
    Busy,
    Current,
    Switch(usize),
}
impl Catalogue {
    pub fn publish(&mut self, slot: usize, object: u64) -> bool {
        exchange(&mut self.words, 0, [slot as u64, object]).0 == 1
    }
    pub fn retire(&mut self, slot: usize) {
        exchange(&mut self.words, 1, [slot as u64, 0]);
    }
    pub fn activate(&mut self, slot: usize) -> bool {
        exchange(&mut self.words, 2, [slot as u64, 0]).0 == 1
    }
    pub fn entry(&self, index: u64) -> Option<[u64; 8]> {
        let (kind, row) = self.query(3, [index, 0]);
        match kind {
            0 => None,
            1 => Some([0; 8]),
            2 => Some([0, row[0], row[1], row[2], 0, 0, 0, 0]),
            _ => unreachable!(),
        }
    }
    pub fn select(&self, ticket: u64, busy: bool) -> Selection {
        let (kind, row) = self.query(4, [ticket, busy.into()]);
        match kind {
            0 => Selection::Missing,
            1 => Selection::Busy,
            2 => Selection::Current,
            3 => Selection::Switch(row[0] as usize),
            _ => unreachable!(),
        }
    }
    pub fn fallback(&self) -> Option<usize> {
        let (kind, row) = self.query(5, [0; 2]);
        (kind == 1).then_some(row[0] as usize)
    }
    pub fn active(&self) -> Option<usize> {
        let (_, row) = self.query(6, [0; 2]);
        (row[0] != 0).then_some(row[0] as usize)
    }
    pub fn has_ticket_capacity(&self) -> bool {
        self.query(7, [0; 2]).0 == 1
    }
    fn query(&self, event: u64, facts: [u64; 2]) -> (u64, [u64; 3]) {
        let mut snapshot = self.words;
        exchange(&mut snapshot, event, facts)
    }
}
#[inline(never)]
fn exchange(words: &mut [u64; 8], event: u64, facts: [u64; 2]) -> (u64, [u64; 3]) {
    let mut p = [0; 128];
    p[0] = 13;
    p[1] = event;
    p[2..4].copy_from_slice(&facts);
    p[16..24].copy_from_slice(words);
    let result = super::native::decide(&mut p);
    words.copy_from_slice(&p[16..24]);
    (result, [p[8], p[9], p[10]])
}
