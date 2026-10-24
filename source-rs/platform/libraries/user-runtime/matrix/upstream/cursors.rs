//! Forward only changed preferences on the supplied interaction connection.
//! Epoch/channel/cancellation cycle all participate in the cache identity.
use super::*;
pub struct Cursors([Option<[u64; 4]>; wire::INPUT_STREAMS]);
impl Cursors {
    pub const fn empty() -> Self {
        Self([None; wire::INPUT_STREAMS])
    }
    pub fn forget(&mut self, index: usize) {
        self.0[index] = None;
    }
    /// Binding is [view epoch, local cancellation cycle, supplied channel, mode].
    pub fn forward(&mut self, index: usize, pair: Pair, binding: [u64; 4]) -> Result<(), Error> {
        if self.0[index] == Some(binding) {
            return Ok(());
        }
        let [epoch, _, channel, mode] = binding;
        let reply = call(
            pair,
            [wire::CURSOR_PREFERENCE, epoch, channel, mode, 0, 0, 0, 0],
        )?;
        // A view/channel can retire before its notification arrives. Never
        // recreate authority; the next binding/cycle makes a fresh request.
        if !matches!(reply[0], 0 | abi::DENIED) {
            return Err(Error(reply[0] as i64));
        }
        self.0[index] = Some(binding);
        Ok(())
    }
}
impl Default for Cursors {
    fn default() -> Self {
        Self::empty()
    }
}
