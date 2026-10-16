//! Monotonic caller-local names. Global slots and epochs never enter a name.
//! Counters survive task-slot reuse; allocation failure cannot wrap into old authority.
use cathedral_kernel_contracts::user as abi;
#[derive(Clone)]
pub struct Names {
    next: [u64; abi::MAX_TASKS],
}
impl Default for Names {
    fn default() -> Self {
        Self {
            next: [1; abi::MAX_TASKS],
        }
    }
}
impl Names {
    pub fn mint(&mut self, task: usize) -> Result<u64, u64> {
        let next = self.next.get_mut(task).ok_or(abi::INVALID_ARGUMENT)?;
        if *next > i64::MAX as u64 {
            return Err(abi::NO_MEMORY);
        }
        let name = *next;
        *next += 1;
        Ok(name)
    }
    pub fn pair(&mut self, owner: usize, peer: usize) -> Result<[u64; 2], u64> {
        let mut staged = self.clone();
        let pair = [staged.mint(owner)?, staged.mint(peer)?];
        *self = staged;
        Ok(pair)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unrelated_activity_is_invisible_and_exhaustion_is_atomic() {
        let mut a = Names::default();
        let mut b = Names::default();
        for _ in 0..30 {
            b.mint(3).unwrap();
        }
        assert_eq!(a.pair(0, 1), b.pair(0, 1));
        b.next[1] = i64::MAX as u64 + 1;
        let before = b.next[0];
        assert_eq!(b.pair(0, 1), Err(abi::NO_MEMORY));
        assert_eq!(b.next[0], before);
        assert_eq!(b.mint(1), Err(abi::NO_MEMORY));
    }
    #[test]
    fn task_clock_and_link_names_do_not_reveal_graph_positions() {
        use crate::{
            ipc::link::{Link, LinkSpec},
            user_tasks::lifecycle::supervision::Supervisor,
            user_tasks::time::Clock,
        };
        fn trace(owner: usize, child: usize, noisy: bool) -> alloc::vec::Vec<u64> {
            let mut names = Names::default();
            if noisy {
                // A disjoint principal holds and cycles unrelated rights.
                let mut other = Supervisor::new(names.pair(0, 0).unwrap(), 0, 0, 1).unwrap();
                for _ in 0..20 {
                    other.request(0, other.launch(0).unwrap(), 0).unwrap();
                    other.started(names.mint(0).unwrap());
                    other.reaped([0; abi::EXIT_BYTES]);
                    other.consume();
                }
            }
            let mut clock = Clock::new(abi::MAX_TASKS, &[owner]).unwrap();
            clock.install(owner, names.mint(owner).unwrap());
            let mut task =
                Supervisor::new(names.pair(owner, owner).unwrap(), owner, owner, child).unwrap();
            let mut link = Link::new(
                LinkSpec {
                    client: owner,
                    service: child,
                },
                abi::MAX_TASKS,
                0,
            )
            .unwrap();
            link.admit(owner, names.mint(owner).unwrap());
            let mut visible = alloc::vec![
                clock.handle(owner).unwrap(),
                task.launch(owner).unwrap(),
                task.port(owner).unwrap(),
                link.port(owner).unwrap()
            ];
            for _ in 0..3 {
                task.request(owner, task.launch(owner).unwrap(), 0).unwrap();
                let ticket = task.started(names.mint(owner).unwrap());
                visible.push(ticket);
                task.reaped([0; abi::EXIT_BYTES]);
                task.consume();
                assert_eq!(task.wait(owner, ticket), Err(abi::BAD_HANDLE));
            }
            visible
        }
        assert_eq!(trace(0, 2, false), trace(3, 7, true));
    }
}
