//! Owner-scoped, single-use admission receipts. Private construction remains in
//! lifecycle::spawn; retaining an outcome must not resume an unrelated syscall.
use super::{Supervisor, abi};

pub(super) struct Receipt {
    name: u64,
    pub(super) cancelled: bool,
    result: Option<Result<u64, u64>>,
}

impl Supervisor {
    pub fn request_async(
        &mut self,
        caller: usize,
        grant: u64,
        argument: u64,
        receipt: u64,
    ) -> Result<(), u64> {
        if receipt == 0 || receipt > i64::MAX as u64 {
            return Err(abi::INVALID_ARGUMENT);
        }
        self.request(caller, grant, argument)?;
        self.admission = Some(Receipt {
            name: receipt,
            cancelled: false,
            result: None,
        });
        Ok(())
    }

    pub fn has_receipt(&self, caller: usize, name: u64) -> bool {
        self.owner_alive
            && caller == self.owner
            && self
                .admission
                .as_ref()
                .is_some_and(|receipt| receipt.name == name)
    }

    /// True means completion was retained for polling, not a blocked syscall.
    /// Caller has already published the child or finished all private rollback.
    pub fn retain_admission(&mut self, result: Result<u64, u64>) -> bool {
        let Some(receipt) = &mut self.admission else {
            return false;
        };
        assert!(receipt.result.is_none());
        receipt.result = Some(result);
        true
    }

    /// Cancellation wins only before commit. A committed child is returned even
    /// when cancellation was requested here; its owner must stop/collect it.
    /// Pending cancellation still waits for rollback before returning REVOKED.
    pub fn poll_admission(
        &mut self,
        caller: usize,
        name: u64,
        cancel: bool,
    ) -> Result<Option<Result<u64, u64>>, u64> {
        if !self.has_receipt(caller, name) {
            return Err(abi::BAD_HANDLE);
        }
        let receipt = self.admission.as_mut().unwrap();
        if receipt.result.is_some() {
            return Ok(receipt.result);
        }
        receipt.cancelled |= cancel;
        Ok(None)
    }

    /// Only after a complete successful copyout; failed observation retains it.
    pub fn consume_admission(&mut self) {
        assert!(
            self.admission
                .as_ref()
                .is_some_and(|receipt| receipt.result.is_some())
        );
        self.admission = None;
    }
}
