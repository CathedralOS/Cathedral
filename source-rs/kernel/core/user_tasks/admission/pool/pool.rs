// Backed process capacity and caller-local executable grants, separate from running tasks.
use cathedral_kernel_contracts::user as abi;
#[derive(Clone)]
pub struct Pool {
    owners: [Option<usize>; abi::MAX_TASKS],
    occupied: [bool; abi::MAX_TASKS],
}
impl Pool {
    pub fn new(first: usize, count: usize) -> Self {
        assert!(first > 0 && count <= abi::MAX_TASKS && first <= count);
        let mut owners = [None; abi::MAX_TASKS];
        owners[first..count].fill(Some(0));
        Self {
            owners,
            occupied: [false; abi::MAX_TASKS],
        }
    }
    pub fn available(&self, owner: usize) -> usize {
        (0..abi::MAX_TASKS)
            .filter(|&i| self.owners[i] == Some(owner) && !self.occupied[i])
            .count()
    }
    pub fn prepare(&self, owner: usize, descendants: usize) -> Result<(usize, u32), u64> {
        let mut free = (owner + 1..abi::MAX_TASKS)
            .filter(|&i| self.owners[i] == Some(owner) && !self.occupied[i]);
        let child = free.next().ok_or(abi::NO_MEMORY)?;
        let mut mask = 0;
        for _ in 0..descendants {
            mask |= 1 << free.next().ok_or(abi::NO_MEMORY)?;
        }
        Ok((child, mask))
    }
    pub fn commit(&mut self, owner: usize, child: usize, descendants: u32) {
        assert_eq!(self.owners[child], Some(owner));
        assert!(!self.occupied[child]);
        self.occupied[child] = true;
        for slot in child + 1..abi::MAX_TASKS {
            if descendants & (1 << slot) != 0 {
                assert_eq!(self.owners[slot], Some(owner));
                assert!(!self.occupied[slot]);
                self.owners[slot] = Some(child);
            }
        }
    }
    pub fn assign(&mut self, owner: usize, child: usize, count: usize) -> Result<(), u64> {
        let old = self.available(child);
        if count > abi::MAX_TASKS {
            return Err(abi::INVALID_ARGUMENT);
        }
        let (from, to, needed) = if count > old {
            (owner, child, count - old)
        } else {
            (child, owner, old - count)
        };
        let candidates = (child + 1..abi::MAX_TASKS)
            .filter(|&i| self.owners[i] == Some(from) && !self.occupied[i]);
        if candidates.clone().count() < needed {
            return Err(abi::NO_MEMORY);
        }
        let mask = candidates
            .take(needed)
            .fold(0u32, |bits, i| bits | (1 << i));
        for i in 0..abi::MAX_TASKS {
            if mask & (1 << i) != 0 {
                self.owners[i] = Some(to);
            }
        }
        Ok(())
    }
    /// Structured owner death returns capacity to its nearest surviving ancestor.
    /// Occupied accounts remain unavailable until their authority/backing is retired.
    pub fn reclaim(&mut self, dead: usize, survivor: usize) {
        for owner in &mut self.owners {
            if *owner == Some(dead) {
                *owner = Some(survivor);
            }
        }
    }
    pub fn reattach(&mut self, child: usize, owner: usize) {
        if self.occupied[child] {
            self.owners[child] = Some(owner);
        }
    }
    pub fn release(&mut self, owner: usize, child: usize) {
        assert_eq!(self.owners[child], Some(owner));
        assert!(self.occupied[child]);
        self.reclaim(child, owner);
        self.occupied[child] = false;
    }
}
