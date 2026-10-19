// Subtree delegation: the owner selects a source and a scope object, the
// child sees only a local binding. A whole source belongs to one child at a
// time; a subtree may be held beside anything, since the provider confines
// it and the owner holds all of it anyway. The scope is what the matrix
// stamps on the child's requests; zero passes on everything the matrix holds.
use cathedral_kernel_contracts::user as abi;
#[derive(Clone, Copy)]
struct Binding {
    incarnation: u64,
    source: usize,
    scope: u64,
}
pub struct Scopes<const N: usize = 2> {
    bindings: [Option<Binding>; N],
    revisions: [u64; N],
}
impl<const N: usize> Scopes<N> {
    pub const fn empty() -> Self {
        Self {
            bindings: [None; N],
            revisions: [0; N],
        }
    }
    pub fn grant(
        &mut self,
        slot: usize,
        incarnation: u64,
        source: usize,
        scope: u64,
        available: usize,
    ) -> Result<(), u64> {
        if slot >= N || incarnation == 0 || source >= available {
            return Err(abi::DENIED);
        }
        if self.bindings[slot].is_some()
            || self
                .bindings
                .iter()
                .flatten()
                .any(|b| b.source == source && b.scope == 0 && scope == 0)
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
            scope,
        });
        Ok(())
    }
    /// The source and scope bound to this connection: source and scope.
    pub fn binding(&self, slot: usize, incarnation: u64) -> Result<(usize, u64), u64> {
        self.bindings
            .get(slot)
            .and_then(|b| *b)
            .filter(|b| b.incarnation == incarnation)
            .map(|b| (b.source, b.scope))
            .ok_or(abi::DENIED)
    }
    pub fn revision(&self, slot: usize, incarnation: u64) -> Result<u64, u64> {
        self.binding(slot, incarnation)?;
        Ok(self.revisions[slot])
    }
    pub fn authorize(
        &self,
        slot: usize,
        incarnation: u64,
        revision: u64,
    ) -> Result<(usize, u64), u64> {
        if self.revision(slot, incarnation)? != revision {
            return Err(abi::DENIED);
        }
        self.binding(slot, incarnation)
    }
    pub fn revoke(&mut self, slot: usize) -> Option<usize> {
        self.bindings.get_mut(slot)?.take().map(|b| b.source)
    }
    pub fn clear(&mut self) {
        self.bindings = [None; N];
    }
}

impl<const N: usize> Default for Scopes<N> {
    fn default() -> Self {
        Self::empty()
    }
}
