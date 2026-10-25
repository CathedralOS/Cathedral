//! Control changes current waits; independent grants retain the original charge.
use super::*;
impl Notifications {
    pub fn control(&mut self, task: usize, handle: u64, action: u64) -> Result<(), u64> {
        if action > abi::NOTIFY_CANCEL_WAITS {
            return Err(abi::INVALID_ARGUMENT);
        }
        let (_, grant) = self.check(task, handle, Right::Control)?;
        let object = self.objects[grant.object].as_mut().unwrap();
        let outcome = match action {
            abi::NOTIFY_CLOSE => {
                object.closed = true;
                object.pending = false;
                abi::NOTIFY_CLOSED
            }
            abi::NOTIFY_WITHDRAW_WAITS => {
                object.withdrawn = true;
                object.pending = false;
                abi::NOTIFY_REVOKED
            }
            _ => abi::NOTIFY_CANCELLED,
        };
        let waiters = object.waiters;
        self.finish_set(grant.object, waiters, outcome);
        Ok(())
    }
    pub(crate) fn drop_grant(&mut self, index: usize) {
        let grant = self.grants[index]
            .take()
            .expect("notification grant custody");
        let task = index / abi::NOTIFY_HANDLES;
        if self.waits[task].is_some_and(|w| w.grant == index && w.result.is_none()) {
            self.finish_set(grant.object, 1 << task, abi::NOTIFY_REVOKED);
        }
        self.unref(grant.object);
    }
    pub(super) fn unref(&mut self, index: usize) {
        let object = self.objects[index].as_mut().unwrap();
        object.references -= 1;
        if object.references == 0 {
            assert_eq!(object.waiters, 0);
            self.charges[object.account] -= 1;
            self.objects[index] = None;
        }
    }
    pub(crate) fn close_task(&mut self, task: usize) {
        if let Some(wait) = self.waits[task].take() {
            self.objects[wait.object].as_mut().unwrap().waiters &= !(1 << task);
            self.unref(wait.object);
        }
        self.changed &= !(1 << task);
        for index in Self::range(task) {
            if self.grants[index].is_some() {
                self.drop_grant(index);
            }
        }
    }
}
