//! Executable capabilities use process-local non-reused names, never catalogue indices.
//! Code comes from a boot template, admitted any number of times, or from a
//! sealed memory object the kernel holds a reference on, admitted once.
use alloc::vec::Vec;
use cathedral_kernel_contracts::user as abi;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Code {
    /// A boot-supplied image by catalogue position.
    Template(usize),
    /// A sealed object by region, entered with `role` as its first argument.
    /// The kernel's read reference keeps the object's frames alive and unaliased
    /// by writers until the launch is admitted or released.
    Object { region: usize, role: u64 },
    /// An object image already admitted: no code authority remains, so the
    /// launch cannot be spawned again from frames that may have been reclaimed.
    Admitted,
}
#[derive(Clone, Copy)]
struct Grant {
    name: u64,
    code: Code,
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
    /// A template is granted once per holder; object grants are each distinct.
    pub fn find(&self, owner: usize, template: usize) -> Option<u64> {
        self.entries[owner]
            .iter()
            .flatten()
            .find(|g| g.code == Code::Template(template))
            .map(|g| g.name)
    }
    pub fn install(&mut self, owner: usize, code: Code, name: u64) -> Result<u64, u64> {
        if let Code::Template(template) = code
            && let Some(name) = self.find(owner, template)
        {
            return Ok(name);
        }
        let slot = self.entries[owner]
            .iter_mut()
            .find(|g| g.is_none())
            .ok_or(abi::NO_MEMORY)?;
        *slot = Some(Grant { name, code });
        Ok(name)
    }
    pub fn resolve(&self, owner: usize, name: u64) -> Result<Code, u64> {
        self.entries[owner]
            .iter()
            .flatten()
            .find(|g| g.name == name)
            .map(|g| g.code)
            .ok_or(abi::BAD_HANDLE)
    }
    /// Consume a single-use grant: the kernel reference it carries moves to the
    /// launch that reserved it.
    pub fn take(&mut self, owner: usize, name: u64) -> Result<Code, u64> {
        let slot = self.entries[owner]
            .iter_mut()
            .find(|g| g.is_some_and(|g| g.name == name))
            .ok_or(abi::BAD_HANDLE)?;
        Ok(slot.take().unwrap().code)
    }
    pub fn at(&self, owner: usize, index: usize) -> Result<u64, u64> {
        self.entries[owner]
            .iter()
            .flatten()
            .nth(index)
            .map(|g| g.name)
            .ok_or(abi::BAD_HANDLE)
    }
    /// Retire every grant. Returns the regions whose kernel references the
    /// caller must drop, one entry per unreserved object grant.
    pub fn close(&mut self, owner: usize) -> [Option<usize>; abi::MAX_LAUNCHES] {
        let mut held = [None; abi::MAX_LAUNCHES];
        for (slot, grant) in self.entries[owner].iter_mut().enumerate() {
            if let Some(Grant {
                code: Code::Object { region, .. },
                ..
            }) = grant.take()
            {
                held[slot] = Some(region);
            }
        }
        held
    }
}
