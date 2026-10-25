//! Caller-local names and checked recipient escrow; source retains delegated rights.
use super::*;
impl Notifications {
    pub(super) fn range(task: usize) -> core::ops::Range<usize> {
        task * abi::NOTIFY_HANDLES..(task + 1) * abi::NOTIFY_HANDLES
    }
    pub(super) fn vacant(&self, task: usize, count: usize) -> Result<[usize; 3], u64> {
        let mut result = [0; 3];
        let mut found = 0;
        for index in Self::range(task) {
            if self.grants[index].is_none() {
                result[found] = index;
                found += 1;
                if found == count {
                    return Ok(result);
                }
            }
        }
        Err(abi::NO_MEMORY)
    }
    fn lookup(&self, task: usize, handle: u64) -> Result<(usize, Grant), u64> {
        Self::range(task)
            .find_map(|i| {
                self.grants[i]
                    .filter(|g| g.visible && g.name == handle)
                    .map(|g| (i, g))
            })
            .ok_or(abi::BAD_HANDLE)
    }
    pub(super) fn check(
        &self,
        task: usize,
        handle: u64,
        right: Right,
    ) -> Result<(usize, Grant), u64> {
        let (i, g) = self.lookup(task, handle)?;
        if g.right != right {
            return Err(abi::DENIED);
        }
        let object = self.objects[g.object].unwrap();
        if object.closed {
            return Err(abi::PEER_CLOSED);
        }
        if right == Right::Wait && object.withdrawn {
            return Err(abi::REVOKED);
        }
        Ok((i, g))
    }
    pub(crate) fn reserve(
        &mut self,
        task: usize,
        handle: u64,
        recipient: usize,
    ) -> Result<usize, u64> {
        let (_, g) = self.lookup(task, handle)?;
        self.check(task, handle, g.right)?;
        let index = self.vacant(recipient, 1)?[0];
        let name = self.names.mint(recipient)?;
        self.grants[index] = Some(Grant {
            name,
            visible: false,
            ..g
        });
        self.objects[g.object].as_mut().unwrap().references += 1;
        Ok(index)
    }
    pub(crate) fn publish(&mut self, recipient: usize, index: usize) -> u64 {
        assert!(Self::range(recipient).contains(&index));
        let g = self.grants[index].as_mut().unwrap();
        assert!(!g.visible);
        g.visible = true;
        g.name
    }
    pub fn drop_handle(&mut self, task: usize, handle: u64) -> Result<(), u64> {
        let (index, _) = self.lookup(task, handle)?;
        self.drop_grant(index);
        Ok(())
    }
}
