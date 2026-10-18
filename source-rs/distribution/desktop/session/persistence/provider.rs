//! One provider incarnation owns its task and control stream together. Failed
//! readiness retires both before another attempt can reuse the launch grant.
use super::*;
use cathedral_user_runtime::task::{Child, Port};
pub(crate) struct Provider {
    pub child: Child,
    pub control: Pair,
    pub starting: Option<u64>,
}
impl Provider {
    pub fn begin(argument: u64) -> Result<Self, Error> {
        let deadline = time::after(policy::STARTUP_TIMEOUT)?;
        let child = Launch::at(3)?.spawn(argument)?;
        let control = match Port::at(3).and_then(|port| port.connect()) {
            Ok(pair) => pair,
            Err(error) => {
                retire_task(child);
                return Err(error);
            }
        };
        let provider = Self {
            child,
            control,
            starting: Some(deadline),
        };
        if let Err(error) = control.0.send(storage::HEALTH) {
            provider.retire();
            return Err(error);
        }
        Ok(provider)
    }
    /// Lab setup and runtime replacement retain their bounded synchronous wait.
    pub fn start() -> Result<Self, Error> {
        let mut provider = Self::begin(0)?;
        let deadline = provider.starting.unwrap();
        match provider.read_ready(deadline) {
            Ok(true) => Ok(provider),
            Ok(false) => unreachable!("readiness deadline reached"),
            Err(error) => {
                provider.retire();
                Err(error)
            }
        }
    }
    #[cfg(feature = "shell")]
    pub fn poll_ready(&mut self) -> Result<bool, Error> {
        self.read_ready(time::now()?)
    }
    fn read_ready(&mut self, until: u64) -> Result<bool, Error> {
        let Some(deadline) = self.starting else {
            return Ok(true);
        };
        let mut bytes = [0; 64];
        match self.control.1.receive_until(&mut bytes, until) {
            Ok(n) if &bytes[..n] == b"ready" => {
                self.starting = None;
                Ok(true)
            }
            Ok(_) => Err(Error(abi::IO_ERROR as i64)),
            Err(Error(code))
                if code == abi::TIMED_OUT as i64 && !time::reached(time::now()?, deadline) =>
            {
                Ok(false)
            }
            Err(error) => Err(error),
        }
    }
    pub fn retire(self) {
        retire_task(self.child);
        let _ = self.control.0.close();
        let _ = self.control.1.close();
    }
    #[cfg(feature = "notes-probes")]
    pub fn arm_notes(&self) {
        self.control
            .0
            .send(&[0xf5, 2, storage::PUBLISH as u8, 0])
            .unwrap();
        let mut bytes = [0; 64];
        let n = self
            .control
            .1
            .receive_until(&mut bytes, time::after(300).unwrap())
            .unwrap();
        assert_eq!(&bytes[..n], b"armed");
    }
}
fn retire_task(child: Child) {
    match child.wait_until(time::now().unwrap()) {
        Ok(_) => (),
        Err(Error(code)) if code == abi::BAD_HANDLE as i64 => (),
        Err(Error(code)) if code == abi::TIMED_OUT as i64 => {
            child.cancel().unwrap();
            child.wait().unwrap();
        }
        Err(error) => panic!("storage collection: {error:?}"),
    }
}
