//! Optional owner-link retirement. A lost connection says nothing about task
//! lifetime; independent clients, caller interests and accepted work stay local.
use super::*;
impl Server {
    /// Finish an explicitly selected owner-control handshake without consuming
    /// application work. The owner must initiate setup independently. Copied
    /// requests during setup are not a substitute for the selected protocol.
    pub fn establish_control(&mut self, deadline: u64) -> Result<(), Error> {
        while !self.shared_ready(0) {
            self.expire_shared()?;
            if self.poll()?.is_some() {
                // Copied application payloads are not the selected setup protocol.
                // Dropping the event also releases any unexpected attachment.
                return Err(Error(abi::IO_ERROR as i64));
            }
            if self.shared_ready(0) {
                break;
            }
            if crate::time::reached(crate::time::now()?, deadline) {
                return Err(Error(abi::TIMED_OUT as i64));
            }
            self.wait(&[], Some(deadline))?;
        }
        Ok(())
    }
    /// Retire failed owner control while continuing independent service work.
    /// Requires a clock grant for the multiplexed wait path. Typed-reply errors
    /// still preserve and report attachment custody; only plain abandoned replies
    /// are discarded. Caller-owned wait errors are never swallowed.
    pub fn isolate_control_failures(&mut self) {
        self.control_isolated = true;
    }
    /// Whether control is admitted and has not failed locally. This includes
    /// incomplete setup; it proves neither shared readiness nor owner liveness.
    pub fn control_connected(&self) -> bool {
        self.control_live
    }
    pub(super) fn control_failed(&mut self, error: Error) -> Result<(), Error> {
        self.reset_shared(0);
        if !self.control_isolated {
            return Err(error);
        }
        if self.control_live {
            self.control_live = false;
            // Shared teardown can already have closed these exact local names.
            self.control.0.close().ok();
            self.control.1.close().ok();
        }
        Ok(())
    }
    pub(super) fn abandoned_control(&self, request: &Request) -> bool {
        request.control
            && self.control_isolated
            && (!self.control_live || request.incarnation != self.control.0.raw())
    }
}
