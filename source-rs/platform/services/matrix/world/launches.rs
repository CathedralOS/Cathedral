//! Bounded launch observations. Receipt sequences and results are local to the
//! requesting endpoint, and follow an exact child handle rather than its slot.
use super::{abi, wire};

#[derive(Clone, Copy)]
struct Observation {
    incarnation: u64,
    receipt: u64,
    child: u64,
    state: u64,
}
#[derive(Default)]
pub struct Launches {
    next: [u64; 2],
    entries: [Option<Observation>; 2],
}
impl Launches {
    pub fn available(&self, slot: usize) -> bool {
        slot < 2 && self.next[slot] != u64::MAX
    }
    pub fn record(&mut self, slot: usize, incarnation: u64, child: u64) -> Result<u64, u64> {
        if !self.available(slot) {
            return Err(abi::NO_MEMORY);
        }
        self.next[slot] += 1;
        self.entries[slot] = Some(Observation {
            incarnation,
            receipt: self.next[slot],
            child,
            state: wire::LAUNCH_RUNNING,
        });
        Ok(self.next[slot])
    }
    pub fn query(&self, slot: usize, incarnation: u64, receipt: u64) -> wire::Packet {
        match self.entries.get(slot).and_then(|entry| *entry) {
            Some(entry) if entry.incarnation == incarnation && entry.receipt == receipt => {
                [0, entry.state, 0, 0, 0, 0, 0, 0]
            }
            _ => wire::error(abi::DENIED),
        }
    }
    pub fn finished(&mut self, child: u64, state: u64) -> Option<usize> {
        assert!(matches!(
            state,
            wire::LAUNCH_EXITED | wire::LAUNCH_FAILED | wire::LAUNCH_STOPPED
        ));
        for (slot, entry) in self.entries.iter_mut().enumerate() {
            if let Some(entry) = entry
                && entry.child == child
                && entry.state == wire::LAUNCH_RUNNING
            {
                entry.state = state;
                return Some(slot);
            }
        }
        None
    }
    pub fn disconnected(&mut self, slot: usize) {
        self.entries[slot] = None;
        self.next[slot] = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn observations_follow_exact_children_and_never_cross_endpoint_lifetimes() {
        let mut launches = Launches::default();
        assert_eq!(launches.query(0, 7, 0), wire::error(abi::DENIED));
        let a = launches.record(0, 7, 100).unwrap();
        assert_eq!(launches.query(1, 8, a), wire::error(abi::DENIED));
        assert_eq!(launches.query(0, 8, a), wire::error(abi::DENIED));
        assert_eq!(launches.query(0, 7, a)[1], wire::LAUNCH_RUNNING);
        let b = launches.record(1, 8, 200).unwrap();
        assert_eq!(
            a, b,
            "one observer's counter cannot expose another's launches"
        );
        assert_eq!(launches.finished(100, wire::LAUNCH_FAILED), Some(0));
        assert_eq!(launches.query(0, 7, a)[1], wire::LAUNCH_FAILED);
        assert_eq!(launches.query(1, 8, b)[1], wire::LAUNCH_RUNNING);
        let replacement = launches.record(0, 7, 101).unwrap();
        assert_eq!(launches.query(0, 7, a), wire::error(abi::DENIED));
        assert_eq!(launches.finished(100, wire::LAUNCH_STOPPED), None);
        assert_eq!(launches.query(0, 7, replacement)[1], wire::LAUNCH_RUNNING);
        launches.disconnected(0);
        let fresh = launches.record(0, 9, 102).unwrap();
        assert_eq!(fresh, 1);
        assert_eq!(launches.query(0, 7, fresh), wire::error(abi::DENIED));
        assert_eq!(launches.finished(101, wire::LAUNCH_FAILED), None);
        assert_eq!(launches.finished(102, wire::LAUNCH_EXITED), Some(0));
        assert_eq!(launches.query(0, 9, fresh)[1], wire::LAUNCH_EXITED);
        launches.next[0] = u64::MAX;
        assert!(!launches.available(0));
        assert_eq!(launches.record(0, 9, 103), Err(abi::NO_MEMORY));
        assert_eq!(launches.query(0, 9, fresh)[1], wire::LAUNCH_EXITED);
    }
}
