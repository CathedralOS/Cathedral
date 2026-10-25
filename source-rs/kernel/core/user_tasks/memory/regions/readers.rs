//! Read capabilities pin objects independently of mappings. Queue escrow is
//! charged before send, inaccessible until receive, and cancelled on discard.
use super::*;
#[derive(Clone, Copy, Debug)]
pub struct ReadGrant {
    pub task: usize,
    pub object: usize,
    pub name: u64,
    pub published: bool,
    pub mapping: Option<usize>,
}
impl Regions {
    pub fn read_slot(&self, task: usize, name: u64) -> Option<usize> {
        self.reads
            .iter()
            .position(|r| r.is_some_and(|r| r.task == task && r.name == name && r.published))
    }
    pub(super) fn read_count(&self, task: usize) -> usize {
        self.reads
            .iter()
            .flatten()
            .filter(|r| r.task == task)
            .count()
    }
    pub fn freeze(&mut self, task: usize, name: u64) -> Result<usize, u64> {
        let index = self.access(task, name)?;
        let object = &mut self.entries[index];
        if object.owner != Some(task) || object.owner_name != name {
            return Err(abi::DENIED);
        }
        if object.sealed || object.shared_mutable {
            return Err(abi::BUSY);
        }
        object.sealed = true;
        Ok(index)
    }
    pub fn reserve_read(&mut self, task: usize, name: u64, receiver: usize) -> Result<usize, u64> {
        self.reserve_view(task, name, receiver, false)
    }
    pub fn share_mutable(&mut self, task: usize, name: u64) -> Result<usize, u64> {
        let index = self.access(task, name)?;
        let object = &mut self.entries[index];
        if object.owner != Some(task) || object.owner_name != name {
            return Err(abi::DENIED);
        }
        if object.sealed || object.shared_mutable {
            return Err(abi::BUSY);
        }
        object.shared_mutable = true;
        Ok(index)
    }
    pub fn reserve_view(
        &mut self,
        task: usize,
        name: u64,
        receiver: usize,
        mutable: bool,
    ) -> Result<usize, u64> {
        let object = self.access(task, name)?;
        if !(if mutable {
            self.entries[object].shared_mutable
        } else {
            self.entries[object].sealed
        }) {
            return Err(abi::DENIED);
        }
        if receiver >= abi::MAX_TASKS || self.shared[receiver] == 0 {
            return Err(abi::DENIED);
        }
        let legacy = self
            .entries
            .iter()
            .filter(|r| r.peer == Some(receiver) && r.accepted)
            .count();
        if self.read_count(receiver) + legacy >= MAX_SHARED_REGIONS
            || self.charged(receiver)[1] + self.entries[object].pages > self.shared[receiver]
        {
            return Err(abi::NO_MEMORY);
        }
        let slot = self
            .reads
            .iter()
            .position(Option::is_none)
            .ok_or(abi::NO_MEMORY)?;
        let name = self.names.mint(receiver)?;
        self.reads[slot] = Some(ReadGrant {
            task: receiver,
            object,
            name,
            published: false,
            mapping: None,
        });
        self.entries[object].read_refs += 1;
        Ok(slot)
    }
    pub fn publish_read(&mut self, task: usize, slot: usize) -> u64 {
        let grant = self.reads[slot].as_mut().unwrap();
        assert!(grant.task == task && !grant.published);
        grant.published = true;
        grant.name
    }
    pub fn drop_read(&mut self, slot: usize) -> Option<usize> {
        let grant = self.reads[slot].take()?;
        assert!(grant.mapping.is_none());
        self.entries[grant.object].read_refs -= 1;
        Some(grant.object)
    }
    pub(super) fn vacant_mapping(&self, task: usize) -> Result<usize, u64> {
        (MAX_PRIVATE_REGIONS..MAX_PRIVATE_REGIONS + MAX_SHARED_REGIONS)
            .find(|&slot| {
                !self
                    .entries
                    .iter()
                    .any(|r| r.peer == Some(task) && r.accepted && r.reader_slot == slot)
                    && !self
                        .reads
                        .iter()
                        .flatten()
                        .any(|r| r.task == task && r.mapping == Some(slot))
            })
            .ok_or(abi::NO_MEMORY)
    }
    pub fn mapped(&self, task: usize, name: u64) -> Result<Option<usize>, u64> {
        if let Some(slot) = self.read_slot(task, name) {
            return Ok(self.reads[slot].unwrap().mapping);
        }
        let index = self.access(task, name)?;
        let r = self.entries[index];
        Ok(if r.owner == Some(task) {
            r.owner_mapped.then_some(r.owner_slot)
        } else {
            Some(r.reader_slot)
        })
    }
    pub fn prepare_mapping(&self, task: usize, name: u64) -> Result<(usize, usize), u64> {
        let object = self.access(task, name)?;
        if !self.entries[object].sealed && !self.entries[object].shared_mutable {
            return Err(abi::DENIED);
        }
        if self.mapped(task, name)?.is_some() {
            return Err(abi::BUSY);
        }
        let slot = if self.read_slot(task, name).is_some() {
            self.vacant_mapping(task)?
        } else {
            self.entries[object].owner_slot
        };
        Ok((object, slot))
    }
    pub fn set_mapping(&mut self, task: usize, name: u64, mapping: Option<usize>) {
        if let Some(slot) = self.read_slot(task, name) {
            self.reads[slot].as_mut().unwrap().mapping = mapping;
        } else {
            let object = self.access(task, name).unwrap();
            assert_eq!(self.entries[object].owner, Some(task));
            self.entries[object].owner_mapped = mapping.is_some();
        }
    }
}
