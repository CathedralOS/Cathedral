//! Owned setup/connection resources release in ordinary failure and Drop paths.
use super::*;
pub(super) struct Lease(pub Pair);
impl Drop for Lease {
    fn drop(&mut self) {
        self.0.0.close().ok();
        self.0.1.close().ok();
    }
}
pub(super) struct Bell(pub [N; 3]);
impl Bell {
    pub fn new() -> Result<Self, Error> {
        N::create().map(Self)
    }
}
impl Drop for Bell {
    fn drop(&mut self) {
        self.0[2].close_object().ok();
        for h in self.0 {
            h.release().ok();
        }
    }
}
pub(super) struct Signal(pub N);
impl Drop for Signal {
    fn drop(&mut self) {
        self.0.release().ok();
    }
}
