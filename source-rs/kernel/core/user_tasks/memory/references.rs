//! Coalesce surrendered executable references and retire one rotating object per visit.
use super::*;

impl Memory {
    /// Take a kernel reference: the region stays live whatever its holders do.
    pub fn hold(&mut self, region: usize) {
        self.model.entries[region].read_refs += 1;
    }
    /// Give a kernel reference back; reclamation waits for the boot stack.
    pub fn unref_later(&mut self, region: usize) {
        self.pending[region] += 1;
        self.pending_regions[region / 64] |= 1 << (region % 64);
        self.retirement_pending = true;
    }
    pub fn retirement_pending(&self) -> bool {
        self.retirement_pending || self.escrow_active
    }
    /// # Safety
    /// Boot stack with frame custody; no user root is active.
    pub(super) unsafe fn settle(&mut self, frames: &mut Frames<'_>) {
        if !self.retirement_pending {
            return;
        }
        let start = self.reference_cursor;
        let next = self
            .pending_regions
            .iter()
            .enumerate()
            .find_map(|(word, &bits)| {
                let bits = if word < start / 64 {
                    0
                } else if word == start / 64 {
                    bits & (u64::MAX << (start % 64))
                } else {
                    bits
                };
                (bits != 0).then(|| word * 64 + bits.trailing_zeros() as usize)
            })
            .or_else(|| {
                self.pending_regions
                    .iter()
                    .enumerate()
                    .find_map(|(word, &bits)| {
                        (bits != 0).then(|| word * 64 + bits.trailing_zeros() as usize)
                    })
            })
            .unwrap();
        self.reference_cursor = (next + 1) % wire::MAX_REGIONS;
        self.model.entries[next].read_refs -= usize::from(core::mem::take(&mut self.pending[next]));
        // SAFETY: References are surrendered once; the bit retains custody of
        // any partly reclaimed backing, and its metadata slot cannot be reused.
        if unsafe { self.reclaim_step(next, frames) } {
            self.pending_regions[next / 64] &= !(1 << (next % 64));
        }
        self.retirement_pending = self.pending_regions.iter().any(|&bits| bits != 0);
    }
}
