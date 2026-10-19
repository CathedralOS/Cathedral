//! Owner-only lab probes wait asynchronously while children call their matrix.
//! One FIFO probe at a time; a failed/expired probe ends this proof program.
use super::children::Children;
use cathedral_kernel_contracts::user as abi;
use cathedral_platform_contracts::matrix as wire;
use cathedral_user_runtime::{
    Error,
    ipc::Handle,
    server::{Request, Server},
    time,
};

pub struct Pending {
    request: Request,
    receive: Handle,
    until: u64,
}
impl Pending {
    /// Lab control envelope: high word is child slot + 1, low word its opcode.
    pub fn start(
        request: Request,
        mut words: wire::Packet,
        children: &Children,
    ) -> Result<Self, Error> {
        let slot = (words[0] >> 32)
            .checked_sub(1)
            .ok_or(Error(abi::DENIED as i64))?;
        let child = children
            .0
            .get(slot as usize)
            .and_then(Option::as_ref)
            .ok_or(Error(abi::PEER_CLOSED as i64))?;
        words[0] &= u32::MAX as u64;
        let bytes = wire::encode(words);
        child.control.0.send(if words[0] == u32::MAX as u64 {
            &bytes[..7]
        } else {
            &bytes
        })?;
        Ok(Self {
            request,
            receive: child.control.1,
            until: time::after(300)?,
        })
    }
    pub fn wait(&self) -> (Handle, u64) {
        (self.receive, self.until)
    }
    pub fn progress(&self, server: &mut Server) -> Result<bool, Error> {
        let mut bytes = [0; 64];
        match self.receive.receive_until(&mut bytes, time::now()?) {
            Ok(len) => {
                server.reply(&self.request, &bytes[..len])?;
                Ok(true)
            }
            Err(Error(code))
                if code == abi::TIMED_OUT as i64 && !time::reached(time::now()?, self.until) =>
            {
                Ok(false)
            }
            Err(error) => Err(error),
        }
    }
}
