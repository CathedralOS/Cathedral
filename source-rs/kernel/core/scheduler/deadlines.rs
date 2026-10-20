// Indexed deadline heap: one entry per account, removed on cancellation/reuse.
// All storage is reserved at construction; updates never allocate. Callers
// supply extended timestamps so wrapping tick clocks do not reorder the heap.
use alloc::{vec, vec::Vec};

pub(super) struct Deadlines {
    heap: Vec<usize>,
    positions: Vec<Option<usize>>,
    times: Vec<u128>,
    #[cfg(test)]
    pub(super) inspections: usize,
}

impl Deadlines {
    pub(super) fn new(count: usize) -> Self {
        Self {
            heap: Vec::with_capacity(count),
            positions: vec![None; count],
            times: vec![0; count],
            #[cfg(test)]
            inspections: 0,
        }
    }

    pub(super) fn set(&mut self, slot: usize, time: u128) {
        self.remove(slot);
        self.times[slot] = time;
        self.positions[slot] = Some(self.heap.len());
        self.heap.push(slot);
        self.up(self.heap.len() - 1);
    }

    pub(super) fn remove(&mut self, slot: usize) {
        let Some(index) = self.positions[slot].take() else {
            return;
        };
        self.heap.swap_remove(index);
        if index < self.heap.len() {
            self.positions[self.heap[index]] = Some(index);
            let index = self.up(index);
            self.down(index);
        }
    }

    pub(super) fn pop_due(&mut self, now: u128) -> Option<usize> {
        #[cfg(test)]
        {
            self.inspections += 1;
        }
        let slot = *self.heap.first()?;
        if self.times[slot] > now {
            return None;
        }
        self.remove(slot);
        Some(slot)
    }

    fn earlier(&self, left: usize, right: usize) -> bool {
        (self.times[self.heap[left]], self.heap[left])
            < (self.times[self.heap[right]], self.heap[right])
    }

    fn swap(&mut self, left: usize, right: usize) {
        self.heap.swap(left, right);
        self.positions[self.heap[left]] = Some(left);
        self.positions[self.heap[right]] = Some(right);
    }

    fn up(&mut self, mut index: usize) -> usize {
        while index > 0 {
            let parent = (index - 1) / 2;
            if !self.earlier(index, parent) {
                break;
            }
            self.swap(index, parent);
            index = parent;
        }
        index
    }

    fn down(&mut self, mut index: usize) {
        loop {
            let left = index * 2 + 1;
            if left >= self.heap.len() {
                return;
            }
            let right = left + 1;
            let next = if right < self.heap.len() && self.earlier(right, left) {
                right
            } else {
                left
            };
            if !self.earlier(next, index) {
                return;
            }
            self.swap(index, next);
            index = next;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replacement_cancellation_and_expiry_match_an_ordered_model() {
        let mut deadlines = Deadlines::new(32);
        let mut model = [None; 32];
        let mut now = u128::from(u64::MAX) - 100;
        let mut random = 11u64;
        for step in 0..20_000 {
            random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
            let slot = (random >> 32) as usize % 32;
            if step % 4 == 0 {
                deadlines.remove(slot);
                model[slot] = None;
            } else {
                let time = now + u128::from(random % 200);
                deadlines.set(slot, time);
                model[slot] = Some(time);
            }
            now += 3;
            loop {
                let expected = model
                    .iter()
                    .enumerate()
                    .filter_map(|(slot, time)| time.map(|time| (time, slot)))
                    .filter(|&(time, _)| time <= now)
                    .min()
                    .map(|(_, slot)| slot);
                assert_eq!(deadlines.pop_due(now), expected);
                let Some(slot) = expected else { break };
                model[slot] = None;
            }
            assert_eq!(deadlines.heap.len(), model.iter().flatten().count());
            assert_eq!(deadlines.heap.capacity(), 32);
        }
    }
}
