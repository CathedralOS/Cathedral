//! Root-only display connection. Interior mutation keeps transport sequencing
//! separate from borrowed retained images; the process has one service thread.
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::display;
use cathedral_user_runtime::{
    Error,
    connection::{Client, Response},
    event::Interest,
    ipc::Handle,
    memory::Object,
    time,
};
use core::cell::RefCell;
pub(super) struct Display(RefCell<Client>);
impl Display {
    pub fn connect(pair: (Handle, Handle)) -> Result<Self, Error> {
        let client = Client::connect(pair, time::after(300)?)?;
        cathedral_user_runtime::write(b"Cathedral matrix: shared display connection ready\n")?;
        Ok(Self(RefCell::new(client)))
    }
    pub fn begin(&self, bytes: &[u8], object: Option<&Object>) -> Result<(), Error> {
        self.0.borrow_mut().begin(bytes, object).map_err(|error| {
            // The shared client retires after a failed submission. The enclosing
            // presentation adapter's WOULD_BLOCK retry must not replay it.
            if error.0 == abi::WOULD_BLOCK as i64 {
                Error(abi::PEER_CLOSED as i64)
            } else {
                error
            }
        })
    }
    pub fn call(&self, bytes: &[u8], object: Option<&Object>) -> Result<[u64; 6], Error> {
        let response = self.0.borrow_mut().call_retained_memory(bytes, object)?;
        self.decode(response)
    }
    pub fn poll(&self) -> Result<Option<[u64; 6]>, Error> {
        let response = self.0.borrow_mut().poll()?;
        response.map(|r| self.decode(r)).transpose()
    }
    fn decode(&self, (bytes, n, object): Response) -> Result<[u64; 6], Error> {
        if object.is_none()
            && let Some(words) = display::decode(&bytes[..n])
        {
            return Ok(words);
        }
        self.0.borrow_mut().retire();
        Err(Error(abi::IO_ERROR as i64))
    }
    pub fn receiver(&self) -> Result<Handle, Error> {
        match self.0.borrow().interests()?[0] {
            Interest::Ipc(h) => Ok(h),
            _ => unreachable!(),
        }
    }
    pub fn wake(&self) -> Result<Option<Interest>, Error> {
        let client = self.0.borrow();
        if client.pending() {
            Ok(Some(client.interests()?[1]))
        } else {
            Ok(None)
        }
    }
    pub fn deadline(&self) -> Option<u64> {
        self.0.borrow().transfer_deadline()
    }
}
