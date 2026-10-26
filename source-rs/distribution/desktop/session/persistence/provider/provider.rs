// One provider incarnation owns its task and control stream together. Failed
// readiness retires both before another attempt can reuse the launch grant.
use super::*;
use cathedral_user_runtime::task::{Child, Port};
use control::Control;
pub(crate) struct Provider {
    pub child: Child,
    control: Control,
    pub starting: Option<u64>,
}
impl Provider {
    pub fn begin(argument: u64) -> Result<Self, Error> {
        let deadline = time::after(policy::STARTUP_TIMEOUT)?;
        let child = Launch::at(3)?.spawn(argument).inspect_err(|error| {
            time::storage_sample("storage-spawn-failed", &[error.0 as u64]);
        })?;
        let control = match Port::at(3).and_then(|port| port.connect()) {
            Ok(pair) => pair,
            Err(error) => {
                retire_task(child);
                report_retired();
                return Err(error);
            }
        };
        match Control::open(control, deadline) {
            Ok(control) => Ok(Self {
                child,
                control,
                starting: Some(deadline),
            }),
            Err(error) => {
                if let Ok(budget) = cathedral_user_runtime::resources::budget() {
                    time::storage_sample(
                        "storage-control-failed",
                        &[
                            error.0 as u64,
                            budget.used.private_pages,
                            budget.limits.private_pages,
                            budget.used.channels,
                            budget.limits.channels,
                        ],
                    );
                }
                retire_task(child);
                report_retired();
                Err(error)
            }
        }
    }
    /// Lab setup retains its bounded synchronous wait; F5 polls readiness.
    #[cfg(not(feature = "shell"))]
    pub fn start() -> Result<Self, Error> {
        let mut provider = Self::begin(0)?;
        let deadline = provider.starting.unwrap();
        let result = (|| {
            while !provider.read_ready()? {
                provider.control.wait(deadline)?;
            }
            Ok(())
        })();
        match result {
            Ok(()) => Ok(provider),
            Err(error) => {
                provider.retire();
                Err(error)
            }
        }
    }
    #[cfg(feature = "shell")]
    pub fn poll_ready(&mut self) -> Result<bool, Error> {
        self.read_ready()
    }
    fn read_ready(&mut self) -> Result<bool, Error> {
        let Some(deadline) = self.starting else {
            return Ok(true);
        };
        match self.control.poll()? {
            Some((bytes, n)) if &bytes[..n] == b"ready" => {
                self.starting = None;
                Ok(true)
            }
            Some(_) => Err(Error(abi::IO_ERROR as i64)),
            None if !time::reached(time::now()?, deadline) => Ok(false),
            None => Err(Error(abi::TIMED_OUT as i64)),
        }
    }
    pub fn begin_health(&mut self) -> Result<(), Error> {
        self.control.health()
    }
    pub fn poll_health(&mut self) -> Result<bool, Error> {
        match self.control.poll()? {
            Some((bytes, n)) if &bytes[..n] == b"ready" => Ok(true),
            Some(_) => Err(Error(abi::IO_ERROR as i64)),
            None => Ok(false),
        }
    }
    #[cfg(any(
        not(feature = "shell"),
        feature = "notes-probes",
        feature = "storage-live-probes"
    ))]
    pub fn command(&mut self, bytes: &[u8]) -> ([u8; 64], usize) {
        self.control.command(bytes).unwrap()
    }
    pub fn retire(self) {
        retire_task(self.child);
        drop(self.control);
        report_retired();
    }
    #[cfg(feature = "notes-probes")]
    pub fn arm_notes(&mut self) {
        let (bytes, n) = self.command(&[0xf2, 2, storage::PUBLISH as u8, 0]);
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

fn report_retired() {
    #[cfg(feature = "shell")]
    {
        let budget = cathedral_user_runtime::resources::budget().unwrap();
        let notifications = cathedral_user_runtime::notification::budget().unwrap();
        time::storage_sample(
            "storage-control-retired",
            &[
                budget.used.private_pages,
                budget.used.reader_pages,
                budget.used.channels,
                notifications[1],
                notifications[3],
            ],
        );
    }
}
