//! Descriptions are fixed before delivery, scoped with the channel and retained
//! through cancellation. A source cannot rename another source or a live binding.
use super::*;
use cathedral_platform_contracts::input::description::Description;
impl<const N: usize> World<N> {
    pub fn describe_input_stream(
        &mut self,
        channel: u32,
        description: Description,
    ) -> Result<(), u64> {
        self.describe_input(Name::local(channel), description)
    }
    pub(in crate::world) fn describe_input(
        &mut self,
        channel: Name,
        description: Description,
    ) -> Result<(), u64> {
        if !self.parent.valid() || description == Description::UNKNOWN {
            return Err(abi::DENIED);
        }
        let index = self
            .input
            .streams
            .iter()
            .position(|s| s.id == channel && !s.retiring)
            .ok_or(abi::DENIED)?;
        let current = self.input.streams[index].description;
        if current == description {
            return Ok(());
        }
        if current != Description::UNKNOWN || self.input.streams[index].description_locked {
            return Err(abi::BUSY);
        }
        self.input.streams[index].description = description;
        Ok(())
    }
}
