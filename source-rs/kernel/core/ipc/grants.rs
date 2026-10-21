//! Process-local names; endpoint arena indices and other tasks never enter a ticket.
//! Even names belong to boot grants, odd names to dynamic grants. Separate name
//! counters keep dynamic exhaustion from preventing a reserved boot reconnection.
use super::{Right, abi};
const BOOT: usize = abi::MAX_ENDPOINTS * 3;

#[derive(Clone, Copy)]
pub(super) struct Grant {
    pub owner: usize,
    pub ticket: u64,
    pub endpoint: usize,
    pub right: Right,
    pub visible: bool,
}
pub(super) struct Grants {
    pub entries: alloc::vec::Vec<Option<Grant>>,
    boot_next: [u64; abi::MAX_TASKS],
    dynamic_next: [u64; abi::MAX_TASKS],
}
impl Grants {
    /// Dynamic grants, including unpublished transfers, are installed in their
    /// recipient's reserved range. Moving a right vacates the source slot.
    pub fn dynamic_slots(task: usize) -> core::ops::Range<usize> {
        let first = BOOT + task * abi::MAX_DYNAMIC_HANDLES;
        first..first + abi::MAX_DYNAMIC_HANDLES
    }
    /// Only boot grants are globally packed. No task operation needs to inspect
    /// another task's dynamic range, regardless of the session's task count.
    pub fn task_slots(task: usize) -> impl Iterator<Item = usize> {
        (0..BOOT).chain(Self::dynamic_slots(task))
    }
    pub fn clear_boot(&mut self, endpoint: usize) {
        assert!(endpoint < abi::MAX_ENDPOINTS);
        self.entries[endpoint * 3..endpoint * 3 + 3].fill(None);
    }
    pub fn new(tasks: usize) -> Result<Self, u64> {
        let mut entries = alloc::vec::Vec::new();
        entries
            .try_reserve_exact(BOOT + tasks * abi::MAX_DYNAMIC_HANDLES)
            .map_err(|_| abi::NO_MEMORY)?;
        entries.resize(BOOT + tasks * abi::MAX_DYNAMIC_HANDLES, None);
        Ok(Self {
            entries,
            boot_next: [2; abi::MAX_TASKS],
            dynamic_next: [1; abi::MAX_TASKS],
        })
    }
    pub fn lookup(&self, task: usize, ticket: u64) -> Result<(usize, Grant), u64> {
        // Boot records are packed once globally, but lookup is always scoped by
        // caller. Dynamic slots have a reserved per-process budget. Neither
        // physical table placement nor endpoint indices appear in local names.
        Self::task_slots(task)
            .find_map(|i| {
                self.entries[i]
                    .filter(|g| g.owner == task && g.visible && g.ticket == ticket)
                    .map(|g| (i, g))
            })
            .ok_or(abi::BAD_HANDLE)
    }
    pub fn install_boot(&mut self, task: usize, endpoint: usize, right: Right, visible: bool) {
        let offset = match right {
            Right::Send => 0,
            Right::Receive => 1,
            Right::Revoke => 2,
        };
        // Boot rebindings are bounded by MAX_EPOCH for each of MAX_ENDPOINTS slots.
        let ticket = self.boot_next[task];
        self.boot_next[task] += 2;
        self.entries[endpoint * 3 + offset] = Some(Grant {
            owner: task,
            ticket,
            endpoint,
            right,
            visible,
        });
    }
    pub fn handle(&self, task: usize, index: u64) -> Result<u64, u64> {
        self.entries[..BOOT]
            .iter()
            .flatten()
            .filter(|g| g.owner == task && g.visible)
            .nth(usize::try_from(index).map_err(|_| abi::BAD_HANDLE)?)
            .map(|g| g.ticket)
            .ok_or(abi::BAD_HANDLE)
    }
    pub fn vacant(
        &self,
        task: usize,
        count: usize,
    ) -> Result<[usize; abi::MAX_INITIAL_ENDPOINTS], u64> {
        let mut slots = [0; abi::MAX_INITIAL_ENDPOINTS];
        if count > slots.len() {
            return Err(abi::INVALID_ARGUMENT);
        }
        if count == 0 {
            return Ok(slots);
        }
        let mut found = 0;
        for i in Self::dynamic_slots(task) {
            if self.entries[i].is_none() {
                slots[found] = i;
                found += 1;
                if found == count {
                    return Ok(slots);
                }
            }
        }
        Err(abi::NO_MEMORY)
    }
    pub fn can_mint(&self, task: usize, count: u64) -> Result<(), u64> {
        if self.dynamic_next[task] > i64::MAX as u64 - 2 * count {
            Err(abi::NO_MEMORY)
        } else {
            Ok(())
        }
    }
    pub fn install(
        &mut self,
        task: usize,
        slot: usize,
        endpoint: usize,
        right: Right,
        visible: bool,
    ) -> u64 {
        assert!(Self::dynamic_slots(task).contains(&slot));
        assert!(endpoint >= abi::MAX_ENDPOINTS && self.entries[slot].is_none());
        let ticket = self.dynamic_next[task];
        self.dynamic_next[task] += 2;
        self.entries[slot] = Some(Grant {
            owner: task,
            ticket,
            endpoint,
            right,
            visible,
        });
        ticket
    }
    pub fn publish(&mut self, task: usize, slot: usize) -> u64 {
        let grant = self.entries[slot].as_mut().unwrap();
        assert_eq!(grant.owner, task);
        assert!(!grant.visible);
        grant.visible = true;
        grant.ticket
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::{EndpointSpec, Ipc};
    #[test]
    fn name_exhaustion_does_not_wrap_or_consume_source_authority() {
        let mut ipc = Ipc::new(
            1,
            2,
            &[EndpointSpec {
                sender: 0,
                receiver: 1,
                revoker: None,
            }],
        )
        .unwrap();
        let [s, r] = ipc.create(0).unwrap();
        ipc.grants.dynamic_next[0] = i64::MAX as u64;
        assert_eq!(ipc.create(0), Err(abi::NO_MEMORY));
        ipc.grants.dynamic_next[1] = i64::MAX as u64;
        assert_eq!(
            ipc.send_cap(0, 2, b"no names", Some(s)),
            Err(abi::NO_MEMORY)
        );
        ipc.send(0, s, b"still owned").unwrap();
        assert_eq!(ipc.receive(0, r, 64).unwrap().unwrap().len, 11);
        assert_eq!(ipc.receive(1, 2, 64), Ok(None));
        // Reserved boot reconnects use their own counter even after exhaustion.
        ipc.close_task(1);
        assert_eq!(
            ipc.prepare_child_with(1, 0, 1, 2, 0, &[s]),
            Err(abi::NO_MEMORY)
        );
        ipc.check(0, s, Right::Send).unwrap();
        ipc.prepare_child_with(1, 0, 1, 2, 0, &[]).unwrap();
        ipc.accept_child(1, 0);
        assert!(ipc.handle(1, 0).is_ok());
    }
    #[test]
    fn unrelated_graph_activity_does_not_change_local_names() {
        let local = EndpointSpec {
            sender: 0,
            receiver: 1,
            revoker: None,
        };
        let other = EndpointSpec {
            sender: 2,
            receiver: 3,
            revoker: None,
        };
        let mut a = Ipc::new(1, 4, &[local]).unwrap();
        let mut b = Ipc::new(99, 4, &[other, local, other]).unwrap();
        for _ in 0..abi::DEFAULT_CHANNELS {
            b.create(2).unwrap();
        }
        assert_eq!(a.handle(0, 0), b.handle(0, 0));
        assert_eq!(a.handle(1, 0), b.handle(1, 0));
        assert_eq!(a.create(0), b.create(0));
    }
}
