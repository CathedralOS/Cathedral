//! Owned setup/connection resources release in ordinary failure and Drop paths.
use super::*;
/// An installed endpoint held until the caller accepts custody explicitly.
pub struct Endpoint(Option<Handle>);
impl Endpoint {
    pub(crate) fn received(handle: Handle) -> Self {
        Self(Some(handle))
    }
    pub fn into_handle(mut self) -> Handle {
        self.0.take().unwrap()
    }
}
impl Drop for Endpoint {
    fn drop(&mut self) {
        if let Some(handle) = self.0.take() {
            let _ = handle.close();
        }
    }
}
pub(super) struct Lease(pub Pair);
impl Drop for Lease {
    fn drop(&mut self) {
        self.0.0.close().ok();
        self.0.1.close().ok();
    }
}
pub(super) struct Bell {
    pub wait: N,
    signal: Option<N>,
    control: N,
}
impl Bell {
    pub fn new() -> Result<Self, Error> {
        let [wait, signal, control] = N::create()?;
        Ok(Self {
            wait,
            signal: Some(signal),
            control,
        })
    }
    pub fn delegate(&mut self, endpoint: Handle) -> Result<(), Error> {
        let signal = self.signal.ok_or(Error(abi::INVALID_ARGUMENT as i64))?;
        signal.delegate(endpoint, b"")?;
        // The queued grant owns an independent right. Retain only our wait and
        // close authority; the setup-only signal must not consume a live slot.
        signal.release()?;
        self.signal = None;
        Ok(())
    }
}
// Teardown is a cold custody boundary, shared by client and accept failure paths.
impl Drop for Bell {
    #[inline(never)]
    fn drop(&mut self) {
        self.control.close_object().ok();
        self.wait.release().ok();
        self.control.release().ok();
        if let Some(signal) = self.signal {
            signal.release().ok();
        }
    }
}
pub(super) struct Signal(pub N);
impl Drop for Signal {
    fn drop(&mut self) {
        self.0.release().ok();
    }
}
