//! Owner-delegated management of one sibling slot. A query mints a local ticket
//! for the exact task; revocation/regrant and task replacement invalidate it.
use super::{abi, wire};
#[derive(Clone, Copy)]
struct Grant {
    incarnation: u64,
    target: usize,
    observed: Option<(u64, u64)>, // ticket, private child handle
}
#[derive(Default)]
pub struct Management {
    grants: [Option<Grant>; 2],
    next: [u64; 2],
}
impl Management {
    pub fn grant(&mut self, manager: usize, incarnation: u64, target: usize) -> Result<(), u64> {
        if manager >= 2 || target >= 2 || manager == target || incarnation == 0 {
            return Err(abi::DENIED);
        }
        self.grants[manager] = Some(Grant {
            incarnation,
            target,
            observed: None,
        });
        Ok(())
    }
    pub fn revoke(&mut self, manager: usize) {
        self.grants[manager] = None;
    }
    pub fn disconnected(&mut self, slot: usize) {
        self.revoke(slot);
        // A new connection must not learn how many tasks its predecessor saw.
        self.next[slot] = 0;
        for grant in self.grants.iter_mut().flatten() {
            if grant.target == slot {
                grant.observed = None;
            }
        }
    }
    pub fn target(&self, manager: usize, incarnation: u64) -> Result<usize, u64> {
        self.grants
            .get(manager)
            .and_then(|g| *g)
            .filter(|g| g.incarnation == incarnation)
            .map(|g| g.target)
            .ok_or(abi::DENIED)
    }
    pub fn query(&mut self, manager: usize, incarnation: u64, task: Option<u64>) -> wire::Packet {
        if let Err(code) = self.target(manager, incarnation) {
            return wire::error(code);
        }
        let grant = self.grants[manager].as_mut().unwrap();
        let Some(task) = task else {
            grant.observed = None;
            return [0; 8];
        };
        if grant.observed.is_none_or(|(_, previous)| previous != task) {
            let Some(next) = self.next[manager].checked_add(1) else {
                return wire::error(abi::NO_MEMORY);
            };
            self.next[manager] = next;
            grant.observed = Some((next, task));
        }
        [0, grant.observed.unwrap().0, 0, 0, 0, 0, 0, 0]
    }
    pub fn authorize(
        &self,
        manager: usize,
        incarnation: u64,
        ticket: u64,
        current: Option<u64>,
    ) -> Result<usize, u64> {
        let target = self.target(manager, incarnation)?;
        match self.grants[manager].unwrap().observed {
            Some((expected, task)) if ticket == expected && current == Some(task) => Ok(target),
            _ => Err(abi::DENIED),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grants_and_confirmations_cannot_cross_task_or_endpoint_lifetimes() {
        let mut m = Management::default();
        assert_eq!(m.target(0, 4), Err(abi::DENIED));
        assert_eq!(m.grant(0, 4, 0), Err(abi::DENIED));
        m.grant(0, 4, 1).unwrap();
        assert_eq!(m.query(0, 4, None), [0; 8]);
        let a = m.query(0, 4, Some(90))[1];
        assert_eq!(m.authorize(0, 4, a, Some(90)), Ok(1));
        assert_eq!(m.authorize(1, 4, a, Some(90)), Err(abi::DENIED));
        assert_eq!(m.authorize(0, 5, a, Some(90)), Err(abi::DENIED));
        assert_eq!(m.authorize(0, 4, a, Some(91)), Err(abi::DENIED));
        assert_eq!(m.authorize(0, 4, a, None), Err(abi::DENIED));
        let b = m.query(0, 4, Some(91))[1];
        assert_ne!(a, b);
        m.revoke(0);
        assert_eq!(m.query(0, 4, Some(91)), wire::error(abi::DENIED));
        m.grant(0, 4, 1).unwrap();
        let c = m.query(0, 4, Some(91))[1];
        assert_ne!(b, c);
        assert_eq!(m.authorize(0, 4, b, Some(91)), Err(abi::DENIED));
        m.disconnected(0);
        assert_eq!(m.target(0, 4), Err(abi::DENIED));
        m.grant(0, 5, 1).unwrap();
        let d = m.query(0, 5, Some(91))[1];
        assert_eq!(d, 1);
        assert_eq!(m.authorize(0, 4, d, Some(91)), Err(abi::DENIED));
        assert_eq!(m.authorize(0, 5, c, Some(91)), Err(abi::DENIED));
        assert_eq!(m.authorize(0, 5, d, Some(91)), Ok(1));
        m.disconnected(1);
        assert_eq!(m.authorize(0, 5, d, Some(91)), Err(abi::DENIED));
        m.next[0] = u64::MAX;
        assert_eq!(m.query(0, 5, Some(92)), wire::error(abi::NO_MEMORY));
    }
}
