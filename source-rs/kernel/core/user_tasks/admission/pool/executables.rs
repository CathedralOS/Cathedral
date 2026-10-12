//! Executable capabilities use process-local non-reused names, never catalogue indices.
use alloc::vec::Vec;
use cathedral_contracts::user as abi;
#[derive(Clone, Copy)]
struct Grant {
    name: u64,
    template: usize,
}
pub struct Executables {
    entries: Vec<[Option<Grant>; abi::MAX_LAUNCHES]>,
}
impl Executables {
    pub fn new(count: usize) -> Result<Self, u64> {
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(count)
            .map_err(|_| abi::NO_MEMORY)?;
        entries.resize(count, [None; abi::MAX_LAUNCHES]);
        Ok(Self { entries })
    }
    pub fn find(&self, owner: usize, template: usize) -> Option<u64> {
        self.entries[owner]
            .iter()
            .flatten()
            .find(|g| g.template == template)
            .map(|g| g.name)
    }
    pub fn install(&mut self, owner: usize, template: usize, name: u64) -> Result<u64, u64> {
        if let Some(name) = self.find(owner, template) {
            return Ok(name);
        }
        let slot = self.entries[owner]
            .iter_mut()
            .find(|g| g.is_none())
            .ok_or(abi::NO_MEMORY)?;
        *slot = Some(Grant { name, template });
        Ok(name)
    }
    pub fn resolve(&self, owner: usize, name: u64) -> Result<usize, u64> {
        self.entries[owner]
            .iter()
            .flatten()
            .find(|g| g.name == name)
            .map(|g| g.template)
            .ok_or(abi::BAD_HANDLE)
    }
    pub fn at(&self, owner: usize, index: usize) -> Result<u64, u64> {
        self.entries[owner]
            .iter()
            .flatten()
            .nth(index)
            .map(|g| g.name)
            .ok_or(abi::BAD_HANDLE)
    }
    pub fn close(&mut self, owner: usize) {
        self.entries[owner].fill(None);
    }
}
