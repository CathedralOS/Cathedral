//! Accept boot-approved peer connections. This grants no lifecycle or device rights.
use crate::{Error, abi, arch, ipc::Handle};
#[derive(Clone, Copy, Debug)]
pub struct Link(u64);
impl Link {
    pub fn at(index: u64) -> Result<Self, Error> {
        result(arch::call(abi::LINK_HANDLE, index, 0)).map(Self)
    }
    /// Client gets (send, receive); service gets (receive, send).
    pub fn connect(self) -> Result<(Handle, Handle), Error> {
        let index = result(arch::call(abi::LINK_CONNECT, self.0, 0))?;
        Ok((Handle::bootstrap(index)?, Handle::bootstrap(index + 1)?))
    }
}
fn result(value: u64) -> Result<u64, Error> {
    if (value as i64) < 0 {
        Err(Error(value as i64))
    } else {
        Ok(value)
    }
}
