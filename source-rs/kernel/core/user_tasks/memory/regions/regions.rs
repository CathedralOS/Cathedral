// Page-object authority and lifetime policy. Paging and frame custody live in user_tasks/memory/memory.rs.
// A sealed accepted reader retains backing after producer death; no remote unmap of a live reader.
use cathedral_kernel_contracts::memory::{
    Grant, MAX_PAGES, MAX_PRIVATE_PAGES, MAX_PRIVATE_REGIONS, MAX_READER_PAGES, MAX_REGIONS,
    MAX_SHARED_REGIONS,
};
use cathedral_kernel_contracts::user as abi;
#[derive(Clone, Copy, Debug, Default)]
pub struct Region {
    /// Persistent backing sponsor; accepted readers can outlive its process.
    pub account: Option<usize>,
    pub read_refs: usize,
    pub owner_mapped: bool,
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
        self.owner.is_some() || self.peer.is_some() || self.read_refs != 0
    }
}
pub struct Regions {
    names: crate::names::Names,
    pub reads: alloc::vec::Vec<Option<readers::ReadGrant>>,
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
        let mut reads = alloc::vec::Vec::new();
        reads
            .try_reserve_exact(cathedral_kernel_contracts::memory::MAX_READ_HANDLES)
            .map_err(|_| abi::NO_MEMORY)?;
        reads.resize(cathedral_kernel_contracts::memory::MAX_READ_HANDLES, None);
        let mut model = Self {
            reads,
            names: crate::names::Names::default(),
            entries,
            private: [0; abi::MAX_TASKS],
            shared: [0; abi::MAX_TASKS],
        };
        let mut seen = 0u32;
        for grant in grants {
            if grant.task >= tasks
                || grant.private_pages > MAX_PRIVATE_PAGES
                || grant.shared_pages > MAX_READER_PAGES
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
        let used = self.charged(owner)[0];
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
            account: Some(owner),
            owner: Some(owner),
            owner_mapped: true,
            pages,
            owner_name,
            owner_slot,
            ..Region::default()
        };
        Ok(index)
    }
    /// Resolve only the caller's published rights, mapped or not. Unknown and foreign are identical.
    pub fn access(&self, caller: usize, handle: u64) -> Result<usize, u64> {
        self.entries
            .iter()
            .position(|r| {
                (r.owner == Some(caller) && r.owner_name == handle)
                    || (r.peer == Some(caller) && r.accepted && r.reader_name == handle)
            })
            .or_else(|| {
                self.read_slot(caller, handle)
                    .map(|i| self.reads[i].unwrap().object)
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
        let used = self.charged(caller)[1];
        if used + region.pages > self.shared[caller]
            || readers.count() + self.read_count(caller) >= MAX_SHARED_REGIONS
        {
            return Err(abi::NO_MEMORY);
        }
        let reader_slot = self.vacant_mapping(caller)?;
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
    pub(crate) fn limits(&self, task: usize) -> [usize; 2] {
        [self.private[task], self.shared[task]]
    }
    pub fn charged(&self, task: usize) -> [usize; 2] {
        [
            self.entries
                .iter()
                .filter(|r| r.live() && r.account == Some(task))
                .map(|r| r.pages)
                .sum(),
            self.entries
                .iter()
                .filter(|r| r.accepted && r.peer == Some(task))
                .map(|r| r.pages)
                .sum::<usize>()
                + self
                    .reads
                    .iter()
                    .flatten()
                    .filter(|r| r.task == task)
                    .map(|r| self.entries[r.object].pages)
                    .sum::<usize>(),
        ]
    }
    /// Only the validated resource transaction may alter backed allowances.
    pub(crate) fn install_limits(&mut self, task: usize, limits: [usize; 2]) {
        [self.private[task], self.shared[task]] = limits;
    }
}
