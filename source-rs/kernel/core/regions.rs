//! Page-object authority and lifetime policy. Paging and frame custody live in user_tasks/session/memory.
//! A sealed accepted reader retains backing after producer death; no remote unmap of a live reader.
use cathedral_contracts::{
    memory::{Grant, MAX_PAGES, MAX_PRIVATE_REGIONS, MAX_REGIONS, MAX_SHARED_REGIONS},
    user as abi,
};
#[derive(Clone, Copy, Debug, Default)]
pub struct Region {
    pub owner: Option<usize>,
    pub peer: Option<usize>,
    pub accepted: bool,
    pub sealed: bool,
    pub pages: usize,
    pub owner_name: u64,
    pub reader_name: u64,
    pub owner_slot: usize,
    pub reader_slot: usize,
}
impl Region {
    pub fn live(&self) -> bool {
        self.owner.is_some() || self.peer.is_some()
    }
}
pub struct Regions {
    names: crate::names::Names,
    pub entries: alloc::vec::Vec<Region>,
    private: [usize; abi::MAX_TASKS],
    shared: [usize; abi::MAX_TASKS],
}
impl Regions {
    pub fn new(tasks: usize, grants: &[Grant]) -> Result<Self, u64> {
        if tasks > abi::MAX_TASKS {
            return Err(abi::INVALID_ARGUMENT);
        }
        // Construct the bounded metadata arena on the heap. Returning several
        // debug-build copies of this array otherwise exhausts the kernel boot stack.
        let mut entries = alloc::vec::Vec::new();
        entries
            .try_reserve_exact(MAX_REGIONS)
            .map_err(|_| abi::NO_MEMORY)?;
        entries.resize(MAX_REGIONS, Region::default());
        let mut model = Self {
            names: crate::names::Names::default(),
            entries,
            private: [0; abi::MAX_TASKS],
            shared: [0; abi::MAX_TASKS],
        };
        let mut seen = 0u8;
        for grant in grants {
            if grant.task >= tasks
                || grant.private_pages > MAX_PAGES
                || grant.shared_pages > MAX_PAGES
                || seen & (1 << grant.task) != 0
            {
                return Err(abi::INVALID_ARGUMENT);
            }
            seen |= 1 << grant.task;
            model.private[grant.task] = grant.private_pages;
            model.shared[grant.task] = grant.shared_pages;
        }
        Ok(model)
    }
    pub fn enabled(&self, task: usize) -> bool {
        self.private[task] + self.shared[task] != 0
    }
    pub fn allocate(&mut self, owner: usize, pages: usize) -> Result<usize, u64> {
        if owner >= abi::MAX_TASKS || pages == 0 || pages > MAX_PAGES {
            return Err(abi::INVALID_ARGUMENT);
        }
        if self.private[owner] == 0 {
            return Err(abi::DENIED);
        }
        let owned = self.entries.iter().filter(|r| r.owner == Some(owner));
        let used: usize = owned.clone().map(|r| r.pages).sum();
        if pages + used > self.private[owner] || owned.count() >= MAX_PRIVATE_REGIONS {
            return Err(abi::NO_MEMORY);
        }
        let index = self
            .entries
            .iter()
            .position(|r| !r.live())
            .ok_or(abi::NO_MEMORY)?;
        let owner_slot = (0..MAX_PRIVATE_REGIONS)
            .find(|&slot| {
                !self
                    .entries
                    .iter()
                    .any(|r| r.owner == Some(owner) && r.owner_slot == slot)
            })
            .ok_or(abi::NO_MEMORY)?;
        let owner_name = self.names.mint(owner)?;
        self.entries[index] = Region {
            owner: Some(owner),
            pages,
            owner_name,
            owner_slot,
            ..Region::default()
        };
        Ok(index)
    }
    /// Resolve only the caller's installed mappings. Unknown and foreign are identical.
    pub fn access(&self, caller: usize, handle: u64) -> Result<usize, u64> {
        self.entries
            .iter()
            .position(|r| {
                (r.owner == Some(caller) && r.owner_name == handle)
                    || (r.peer == Some(caller) && r.accepted && r.reader_name == handle)
            })
            .ok_or(abi::BAD_HANDLE)
    }
    pub fn mapping(&self, caller: usize, index: usize) -> usize {
        let region = self.entries[index];
        if region.owner == Some(caller) {
            region.owner_slot
        } else {
            assert!(region.peer == Some(caller) && region.accepted);
            region.reader_slot
        }
    }
    pub fn handle(&self, caller: usize, index: usize) -> u64 {
        let region = self.entries[index];
        if region.owner == Some(caller) {
            region.owner_name
        } else {
            assert!(region.peer == Some(caller) && region.accepted);
            region.reader_name
        }
    }
    /// Caller has separately checked a live boot-approved producer -> consumer link.
    pub fn offer(&mut self, caller: usize, handle: u64, peer: usize) -> Result<usize, u64> {
        let index = self.access(caller, handle)?;
        if self.entries[index].owner != Some(caller)
            || peer >= abi::MAX_TASKS
            || peer == caller
            || self.shared[peer] == 0
        {
            return Err(abi::DENIED);
        }
        if self.entries[index].sealed {
            return Err(abi::BUSY);
        }
        self.entries[index].sealed = true;
        self.entries[index].peer = Some(peer);
        Ok(index)
    }
    /// The offer is a producer-local name scoped by an authenticated service link.
    /// It never acts as a receiver mapping handle; acceptance installs a fresh one.
    pub fn accept(&mut self, caller: usize, producer: usize, offer: u64) -> Result<usize, u64> {
        let index = self
            .entries
            .iter()
            .position(|r| {
                r.owner == Some(producer)
                    && r.owner_name == offer
                    && r.peer == Some(caller)
                    && r.sealed
            })
            .ok_or(abi::BAD_HANDLE)?;
        let region = self.entries[index];
        if region.accepted {
            return Err(abi::BUSY);
        }
        let readers = self
            .entries
            .iter()
            .filter(|r| r.peer == Some(caller) && r.accepted);
        let used: usize = readers.clone().map(|r| r.pages).sum();
        if used + region.pages > self.shared[caller] || readers.count() >= MAX_SHARED_REGIONS {
            return Err(abi::NO_MEMORY);
        }
        let reader_slot = (MAX_PRIVATE_REGIONS..MAX_PRIVATE_REGIONS + MAX_SHARED_REGIONS)
            .find(|&slot| {
                !self
                    .entries
                    .iter()
                    .any(|r| r.peer == Some(caller) && r.accepted && r.reader_slot == slot)
            })
            .ok_or(abi::NO_MEMORY)?;
        let reader_name = self.names.mint(caller)?;
        self.entries[index].reader_name = reader_name;
        self.entries[index].reader_slot = reader_slot;
        self.entries[index].accepted = true;
        Ok(index)
    }
    /// Returns the backing index; caller resolves its local mapping before release.
    /// A live accepted reader pins the allocation.
    pub fn release(&mut self, caller: usize, handle: u64) -> Result<usize, u64> {
        let index = self.access(caller, handle)?;
        let region = &mut self.entries[index];
        if region.owner == Some(caller) {
            if region.peer.is_some() {
                return Err(abi::BUSY);
            }
            region.owner = None;
        } else if region.peer == Some(caller) && region.accepted {
            region.peer = None;
            region.accepted = false;
        } else {
            return Err(abi::DENIED);
        }
        Ok(index)
    }
    pub fn close(&mut self, index: usize, task: usize) {
        let region = &mut self.entries[index];
        if region.owner == Some(task) {
            region.owner = None;
            if !region.accepted {
                region.peer = None;
            }
        }
        if region.peer == Some(task) {
            region.peer = None;
            region.accepted = false;
        }
    }
}
#[cfg(test)]
mod tests;
