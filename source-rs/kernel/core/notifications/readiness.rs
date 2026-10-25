//! Mixed waits observe pending state, never consume it. Subscriptions carry no
//! authority and disappear before task-slot reuse. Held grants pin the objects.
use super::*;
impl Notifications {
    pub(crate) fn ready(&self, task: usize, handle: u64) -> Result<bool, u64> {
        let (_, grant) = self.check(task, handle, Right::Wait)?;
        Ok(self.objects[grant.object].unwrap().pending)
    }
    pub(crate) fn observe(&mut self, task: usize, handle: u64) -> Result<(), u64> {
        let (_, grant) = self.check(task, handle, Right::Wait)?;
        self.objects[grant.object].as_mut().unwrap().observers |= 1 << task;
        Ok(())
    }
    pub(crate) fn unobserve(&mut self, task: usize) {
        // Fixed preallocated arena, not a scan of processes or queued requests.
        for object in self.objects.iter_mut().flatten() {
            object.observers &= !(1 << task);
        }
    }
}
