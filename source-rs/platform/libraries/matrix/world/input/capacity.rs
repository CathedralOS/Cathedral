//! Calculate the inherited ceiling after reserving local sources. The service
//! must obtain the parent's acknowledgement before using this free capacity.
use super::*;
impl<const N: usize> World<N> {
    /// A device adapter has already consumed this sample and cannot retry it as
    /// an app command. Capacity pressure is a stream interruption, not adapter
    /// death; suppress the residual hold and keep receiving hardware events.
    pub fn source_sample(&mut self, channel: u32, event: [u64; 4]) -> Result<(), u64> {
        let [kind, x, y, value] = event;
        let result = match kind {
            wire::POINTER => self.pointer_stream(channel, x, y, value),
            wire::KEY if value == 0 => self.key_stream(channel, x, y),
            _ => Err(abi::INVALID_ARGUMENT),
        };
        if result == Err(abi::NO_MEMORY) {
            self.inherited_capacity_loss(Name::local(channel), [0, 0, kind, x, y, value, 0, 0]);
            Ok(())
        } else {
            result
        }
    }
    pub fn inherited_input_headroom(
        &mut self,
        additional_channels: usize,
        additional_groups: usize,
    ) -> Result<[usize; 2], u64> {
        self.reap_input();
        let channels = self
            .input
            .streams
            .iter()
            .filter(|s| s.id.valid() && !s.id.inherited)
            .count();
        let groups = self
            .input
            .groups
            .iter()
            .filter(|g| g.id.valid() && !g.id.inherited)
            .count();
        Ok([
            wire::INPUT_STREAMS
                .checked_sub(channels)
                .and_then(|n| n.checked_sub(additional_channels))
                .ok_or(abi::NO_MEMORY)?,
            wire::INPUT_GROUPS
                .checked_sub(groups)
                .and_then(|n| n.checked_sub(additional_groups))
                .ok_or(abi::NO_MEMORY)?,
        ])
    }
    /// An upstream event has already been consumed. A downstream capacity
    /// refusal cannot be reported as a parent grant violation. Interrupt this
    /// stream and suppress residual holds until release, as with queue overflow.
    pub(in crate::world) fn inherited_capacity_loss(&mut self, name: Name, event: wire::Packet) {
        let Some(index) = self.input.streams.iter().position(|s| s.id == name) else {
            return;
        };
        let stream = &mut self.input.streams[index];
        if event[2] == wire::POINTER {
            stream.buttons = event[5];
        } else if event[2] == wire::KEY {
            let bit = 1 << (event[3] - 1);
            if event[4] == cathedral_platform_contracts::input::RELEASE as u64 {
                stream.held &= !bit;
            } else {
                stream.held |= bit;
            }
        }
        self.cancel_stream_index(index);
    }
}
