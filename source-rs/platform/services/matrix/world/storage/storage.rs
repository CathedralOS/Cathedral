// Whole-root delegation: owner selects a source, child sees only a local binding.
// Exclusive sibling assignments keep transactions and root generations private.
use cathedral_kernel_contracts::user as abi;
#[derive(Clone, Copy)]
struct Binding {
    incarnation: u64,
    source: usize,
}
#[derive(Default)]
pub struct Scopes {
    bindings: [Option<Binding>; 2],
    revisions: [u64; 2],
}
impl Scopes {
    pub const fn empty() -> Self {
        Self {
            bindings: [None; 2],
            revisions: [0; 2],
        }
    }
    pub fn grant(
        &mut self,
        slot: usize,
        incarnation: u64,
        source: usize,
        available: usize,
    ) -> Result<(), u64> {
        if slot >= 2 || incarnation == 0 || source >= available {
            return Err(abi::DENIED);
        }
        if self.bindings[slot].is_some()
            || self.bindings.iter().flatten().any(|b| b.source == source)
        {
            return Err(abi::BUSY);
        }
        self.revisions[slot] = self.revisions[slot]
            .checked_add(1)
            .filter(|n| *n <= u64::MAX >> cathedral_platform_contracts::matrix::STORAGE_SCOPE_BITS)
            .ok_or(abi::NO_MEMORY)?;
        self.bindings[slot] = Some(Binding {
            incarnation,
            source,
        });
        Ok(())
    }
    pub fn source(&self, slot: usize, incarnation: u64) -> Result<usize, u64> {
        self.bindings
            .get(slot)
            .and_then(|b| *b)
            .filter(|b| b.incarnation == incarnation)
            .map(|b| b.source)
            .ok_or(abi::DENIED)
    }
    pub fn revision(&self, slot: usize, incarnation: u64) -> Result<u64, u64> {
        self.source(slot, incarnation)?;
        Ok(self.revisions[slot])
    }
    pub fn authorize(&self, slot: usize, incarnation: u64, revision: u64) -> Result<usize, u64> {
        if self.revision(slot, incarnation)? != revision {
            return Err(abi::DENIED);
        }
        self.source(slot, incarnation)
    }
    pub fn revoke(&mut self, slot: usize) -> Option<usize> {
        self.bindings.get_mut(slot)?.take().map(|b| b.source)
    }
    pub fn clear(&mut self) {
        self.bindings = [None; 2];
    }
}
