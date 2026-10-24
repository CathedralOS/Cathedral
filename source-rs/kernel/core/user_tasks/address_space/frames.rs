//! Capture every allocation, including partial table construction; retirement refunds in reverse order.
use cathedral_arch::FrameSource;
pub(super) struct Tracked<'a, F> {
    pub frames: &'a mut F,
    pub addresses: &'a mut alloc::vec::Vec<u64>,
    pub len: &'a mut usize,
}
// SAFETY: The underlying source supplies exclusive physical custody. Every allocation
// remains in this ledger; callers serialize it and release only after retiring the root.
unsafe impl<F: FrameSource> FrameSource for Tracked<'_, F> {
    fn allocate(&mut self) -> Option<u64> {
        // The supplied source enforces admission and conserved physical funding.
        // Extra translation tables still belong in the retirement ledger.
        self.addresses.try_reserve(1).ok()?;
        let address = self.frames.allocate()?;
        self.addresses.push(address);
        *self.len += 1;
        Some(address)
    }
    unsafe fn release(&mut self, _address: u64) {
        unreachable!("mapping backend cannot refund owned frames");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{boxed::Box, vec, vec::Vec};
    #[derive(Clone)]
    #[repr(align(4096))]
    struct Page([u8; 4096]);
    struct Funded {
        pages: Box<[Page]>,
        used: usize,
    }
    // SAFETY: The fixed allocation owns disjoint aligned pages until teardown.
    unsafe impl FrameSource for Funded {
        fn allocate(&mut self) -> Option<u64> {
            if self.used == self.pages.len() {
                return None;
            }
            let address = self.pages[self.used].0.as_mut_ptr() as u64;
            self.used += 1;
            Some(address)
        }
        unsafe fn release(&mut self, _: u64) {
            panic!("test source retains all pages until teardown");
        }
    }
    #[test]
    fn ledger_records_extra_tables_but_never_allocates_past_its_funded_source() {
        let limit = cathedral_kernel_contracts::user::MAX_ADMISSION_FRAMES + 1;
        let mut source = Funded {
            pages: vec![Page([0; 4096]); limit].into_boxed_slice(),
            used: 0,
        };
        let mut addresses = Vec::new();
        let mut length = 0;
        {
            let mut tracked = Tracked {
                frames: &mut source,
                addresses: &mut addresses,
                len: &mut length,
            };
            for _ in 0..limit {
                assert!(tracked.allocate().is_some());
            }
            assert!(tracked.allocate().is_none());
        }
        assert_eq!(length, limit);
        assert_eq!(addresses.len(), limit);
        assert_eq!(source.pages.len(), limit);
        assert_eq!(source.used, limit);
    }
}
