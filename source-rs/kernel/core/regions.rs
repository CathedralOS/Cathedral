//! Page-object authority and lifetime policy. Paging and frame custody live in user_tasks/session/memory.
//! A sealed accepted reader retains backing after producer death; no remote unmap of a live reader.
use cathedral_contracts::{
    memory::{Grant, MAX_PAGES, MAX_REGIONS},
    user as abi,
};
#[derive(Clone, Copy, Debug, Default)]
pub struct Region {
    pub owner: Option<usize>,
    pub peer: Option<usize>,
    pub accepted: bool,
    pub sealed: bool,
    pub pages: usize,
    pub epoch: u64,
}
impl Region {
    pub fn live(&self) -> bool {
        self.owner.is_some() || self.peer.is_some()
    }
    pub fn handle(&self, index: usize) -> u64 {
        (self.epoch << 8) | (index as u64 + 1)
    }
}
pub struct Regions {
    pub entries: [Region; MAX_REGIONS],
    private: [usize; abi::MAX_TASKS],
    shared: [usize; abi::MAX_TASKS],
}
impl Regions {
    pub fn new(tasks: usize, grants: &[Grant]) -> Result<Self, u64> {
        if tasks > abi::MAX_TASKS {
            return Err(abi::INVALID_ARGUMENT);
        }
        let mut model = Self {
            entries: [Region::default(); MAX_REGIONS],
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
    pub fn allocate(&mut self, owner: usize, pages: usize, epoch: u64) -> Result<usize, u64> {
        if owner >= abi::MAX_TASKS
            || pages == 0
            || pages > MAX_PAGES
            || epoch == 0
            || epoch > crate::ipc::MAX_EPOCH
        {
            return Err(abi::INVALID_ARGUMENT);
        }
        if self.private[owner] == 0 {
            return Err(abi::DENIED);
        }
        let used: usize = self
            .entries
            .iter()
            .filter(|r| r.owner == Some(owner))
            .map(|r| r.pages)
            .sum();
        if pages + used > self.private[owner] {
            return Err(abi::NO_MEMORY);
        }
        let index = self
            .entries
            .iter()
            .position(|r| !r.live())
            .ok_or(abi::NO_MEMORY)?;
        self.entries[index] = Region {
            owner: Some(owner),
            pages,
            epoch,
            ..Region::default()
        };
        Ok(index)
    }
    pub fn find(&self, handle: u64) -> Result<usize, u64> {
        let index = (handle & 255).checked_sub(1).ok_or(abi::BAD_HANDLE)? as usize;
        let region = self.entries.get(index).ok_or(abi::BAD_HANDLE)?;
        if !region.live() || region.handle(index) != handle {
            return Err(abi::BAD_HANDLE);
        }
        Ok(index)
    }
    pub fn access(&self, caller: usize, handle: u64) -> Result<usize, u64> {
        let index = self.find(handle)?;
        let region = self.entries[index];
        if region.owner != Some(caller) && !(region.peer == Some(caller) && region.accepted) {
            return Err(abi::DENIED);
        }
        Ok(index)
    }
    /// Caller has separately checked a live boot-approved producer -> consumer link.
    pub fn offer(&mut self, caller: usize, handle: u64, peer: usize) -> Result<usize, u64> {
        let index = self.find(handle)?;
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
    /// Authenticate the producer before a multiplexing service accepts an offer.
    pub fn check_owner(&self, handle: u64, owner: usize) -> Result<(), u64> {
        if self.entries[self.find(handle)?].owner != Some(owner) {
            return Err(abi::DENIED);
        }
        Ok(())
    }
    pub fn accept(&mut self, caller: usize, handle: u64) -> Result<usize, u64> {
        let index = self.find(handle)?;
        let region = self.entries[index];
        if region.peer != Some(caller) {
            return Err(abi::DENIED);
        }
        if region.accepted {
            return Err(abi::BUSY);
        }
        let used: usize = self
            .entries
            .iter()
            .filter(|r| r.peer == Some(caller) && r.accepted)
            .map(|r| r.pages)
            .sum();
        if used + region.pages > self.shared[caller] {
            return Err(abi::NO_MEMORY);
        }
        self.entries[index].accepted = true;
        Ok(index)
    }
    /// Returns the mapping to remove. A live accepted reader pins the allocation.
    pub fn release(&mut self, caller: usize, handle: u64) -> Result<usize, u64> {
        let index = self.find(handle)?;
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
