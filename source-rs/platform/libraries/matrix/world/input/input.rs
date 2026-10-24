// Admit upstream stream names, then route or cancel only that stream.
// Caller authentication belongs to the service; no child-facing injection exists.
use super::*;

pub(super) struct Input {
    streams: [Stream; wire::INPUT_STREAMS],
}
impl Input {
    pub const fn empty() -> Self {
        Self {
            streams: [Stream::empty(); wire::INPUT_STREAMS],
        }
    }
    fn admit(&mut self, id: u32) -> Result<usize, u64> {
        if id == 0 {
            return Err(abi::INVALID_ARGUMENT);
        }
        if let Some(slot) = self.streams.iter().position(|s| s.id == id) {
            return Ok(slot);
        }
        let slot = self
            .streams
            .iter()
            .position(|s| s.id == 0)
            .ok_or(abi::NO_MEMORY)?;
        self.streams[slot].id = id;
        Ok(slot)
    }
    pub fn forget(&mut self, child: usize) {
        for stream in &mut self.streams {
            stream.forget(child);
        }
    }
}

impl<const N: usize> World<N> {
    /// Entire upstream binding is interrupted; preserve local selections.
    pub fn cancel(&mut self) {
        for index in 0..wire::INPUT_STREAMS {
            if self.input.streams[index].id != 0 {
                self.cancel_stream_index(index);
            }
        }
    }
    fn cancel_stream_index(&mut self, index: usize) {
        let id = self.input.streams[index].id;
        for child in &mut self.children {
            child.cancel_stream(index, id);
        }
        self.input.streams[index].cancel();
    }
    /// Authenticated upstream lifecycle event, not a child-selected stream ID.
    pub fn cancel_stream(&mut self, id: u32) -> Result<(), u64> {
        if !self.parent.valid() {
            return Err(abi::DENIED);
        }
        let index = self.input.admit(id)?;
        self.cancel_stream_index(index);
        Ok(())
    }
    /// A delivered CANCEL already terminates the upstream stream's held state.
    /// Its sender suppresses residual hardware holds until release. Waiting for
    /// those releases here would suppress the first fresh press after recovery.
    pub(super) fn inherit_stream_cancel(&mut self, id: u32) -> Result<(), u64> {
        if !self.parent.valid() {
            return Err(abi::DENIED);
        }
        let index = self.input.admit(id)?;
        self.cancel_stream_index(index);
        let focus = self.input.streams[index].focus;
        self.input.streams[index] = Stream {
            id,
            focus,
            ..Stream::empty()
        };
        Ok(())
    }
    /// Existing single-source adapter; this does not grant a child injection rights.
    pub fn pointer(&mut self, x: u64, y: u64, buttons: u64) {
        let _ = self.pointer_stream(wire::SINGLE_INPUT_STREAM, x, y, buttons);
    }
    pub fn key(&mut self, key: u64, state: u64) {
        let _ = self.key_stream(wire::SINGLE_INPUT_STREAM, key, state);
    }
}
