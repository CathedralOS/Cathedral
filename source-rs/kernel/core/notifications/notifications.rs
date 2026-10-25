// Remember signals and serialize registration/completion. No payload, PID lookup,
// scheduling handoff or memory-publication semantics belong to this object.
// Grants own local names/escrow; lifetime owns revocation and final funding release.
use alloc::vec::Vec;
use cathedral_kernel_contracts::user as abi;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Right {
    Wait,
    Signal,
    Control,
}
#[derive(Clone, Copy)]
struct Object {
    account: usize,
    references: usize,
    pending: bool,
    closed: bool,
    withdrawn: bool,
    waiters: u32,
    observers: u32,
}
#[derive(Clone, Copy)]
struct Grant {
    name: u64,
    object: usize,
    right: Right,
    visible: bool,
}
#[derive(Clone, Copy)]
struct Wait {
    grant: usize,
    object: usize,
    deadline: Option<u64>,
    result: Option<u64>,
}
pub struct Notifications {
    objects: Vec<Option<Object>>,
    grants: Vec<Option<Grant>>,
    names: crate::names::Names,
    waits: [Option<Wait>; abi::MAX_TASKS],
    charges: [usize; abi::MAX_TASKS],
    changed: u32,
}
impl Notifications {
    pub(crate) fn new(tasks: usize) -> Result<Self, u64> {
        assert!(tasks <= abi::MAX_TASKS);
        let mut objects = Vec::new();
        objects
            .try_reserve_exact(abi::MAX_DYNAMIC_CHANNELS)
            .map_err(|_| abi::NO_MEMORY)?;
        objects.resize(abi::MAX_DYNAMIC_CHANNELS, None);
        let mut grants = Vec::new();
        grants
            .try_reserve_exact(tasks * abi::NOTIFY_HANDLES)
            .map_err(|_| abi::NO_MEMORY)?;
        grants.resize(tasks * abi::NOTIFY_HANDLES, None);
        Ok(Self {
            objects,
            grants,
            names: Default::default(),
            waits: [None; abi::MAX_TASKS],
            charges: [0; abi::MAX_TASKS],
            changed: 0,
        })
    }
    pub(crate) fn create(&mut self, task: usize) -> Result<[u64; 3], u64> {
        let vacant = self.vacant(task, 3)?;
        let object = self
            .objects
            .iter()
            .position(Option::is_none)
            .ok_or(abi::NO_MEMORY)?;
        let mut names = self.names.clone();
        let handles = [names.mint(task)?, names.mint(task)?, names.mint(task)?];
        self.names = names;
        self.objects[object] = Some(Object {
            account: task,
            references: 3,
            pending: false,
            closed: false,
            withdrawn: false,
            waiters: 0,
            observers: 0,
        });
        self.charges[task] += 1;
        for (i, right) in [Right::Wait, Right::Signal, Right::Control]
            .into_iter()
            .enumerate()
        {
            self.grants[vacant[i]] = Some(Grant {
                name: handles[i],
                object,
                right,
                visible: true,
            });
        }
        Ok(handles)
    }
    pub fn charged(&self, account: usize) -> usize {
        self.charges[account]
    }
    pub fn handles(&self, task: usize) -> usize {
        self.grants[Self::range(task)].iter().flatten().count()
    }
    pub fn signal(&mut self, task: usize, handle: u64, all: bool) -> Result<(), u64> {
        let (_, grant) = self.check(task, handle, Right::Signal)?;
        self.signal_object(grant.object, all);
        Ok(())
    }
    fn signal_object(&mut self, object: usize, all: bool) {
        let obj = self.objects[object].as_mut().unwrap();
        self.changed |= obj.observers;
        let mut waiters = obj.waiters;
        if waiters == 0 {
            obj.pending = true;
            return;
        }
        if !all {
            waiters &= waiters.wrapping_neg();
        }
        self.finish_set(object, waiters, abi::NOTIFY_SIGNALED);
    }
    pub fn wait(
        &mut self,
        task: usize,
        handle: u64,
        deadline: Option<u64>,
        now: u64,
    ) -> Result<Option<u64>, u64> {
        if self.waits[task].is_some() {
            return Err(abi::BUSY);
        }
        let (index, grant) = self.check(task, handle, Right::Wait)?;
        Ok(self.register(task, index, grant.object, deadline, now))
    }
    pub fn signal_and_wait(
        &mut self,
        task: usize,
        signal: u64,
        wait: u64,
        deadline: Option<u64>,
        now: u64,
    ) -> Result<Option<u64>, u64> {
        if self.waits[task].is_some() {
            return Err(abi::BUSY);
        }
        let (_, peer) = self.check(task, signal, Right::Signal)?;
        let (index, own) = self.check(task, wait, Right::Wait)?;
        // No fallible operation follows this signal commit. A nonnegative wait
        // outcome (including timeout/closure) must never imply it wasn't sent.
        self.signal_object(peer.object, false);
        Ok(self.register(task, index, own.object, deadline, now))
    }
    fn register(
        &mut self,
        task: usize,
        grant: usize,
        object: usize,
        deadline: Option<u64>,
        now: u64,
    ) -> Option<u64> {
        let obj = self.objects[object].as_mut().unwrap();
        if core::mem::take(&mut obj.pending) {
            return Some(abi::NOTIFY_SIGNALED);
        }
        if expired(now, deadline) {
            return Some(abi::NOTIFY_TIMED_OUT);
        }
        obj.waiters |= 1 << task;
        obj.references += 1; // Completion custody pins even a withdrawn grant.
        self.waits[task] = Some(Wait {
            grant,
            object,
            deadline,
            result: None,
        });
        None
    }
    fn finish_set(&mut self, object: usize, mut tasks: u32, outcome: u64) {
        self.objects[object].as_mut().unwrap().waiters &= !tasks;
        self.changed |= tasks;
        while tasks != 0 {
            let slot = tasks.trailing_zeros() as usize;
            tasks &= tasks - 1;
            let wait = self.waits[slot].as_mut().unwrap();
            assert_eq!(wait.object, object);
            assert!(wait.result.is_none());
            wait.result = Some(outcome);
        }
    }
    pub(crate) fn take_changed(&mut self) -> u32 {
        core::mem::take(&mut self.changed)
    }
    #[cfg(feature = "memory-lab")]
    pub(crate) fn deadline(&self, task: usize) -> Option<u64> {
        self.waits[task].unwrap().deadline
    }
    pub(crate) fn complete(&mut self, task: usize, now: u64) -> Option<u64> {
        let wait = self.waits[task].expect("registered notification wait");
        let result = wait
            .result
            .or_else(|| expired(now, wait.deadline).then_some(abi::NOTIFY_TIMED_OUT))?;
        self.objects[wait.object].as_mut().unwrap().waiters &= !(1 << task);
        self.waits[task] = None;
        self.unref(wait.object);
        Some(result)
    }
}
fn expired(now: u64, deadline: Option<u64>) -> bool {
    deadline.is_some_and(|d| now.wrapping_sub(d) < (1 << 63))
}
