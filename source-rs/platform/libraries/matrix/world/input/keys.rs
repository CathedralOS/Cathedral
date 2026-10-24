//! Per-stream key transitions and focus-switch suppression.
use super::*;
use cathedral_platform_contracts::input as keys;
impl<const N: usize> World<N> {
    pub fn key_stream(&mut self, channel: u32, key: u64, state: u64) -> Result<(), u64> {
        self.key_named(Name::local(channel), key, state)
    }
    pub(in crate::world) fn key_named(
        &mut self,
        channel: Name,
        key: u64,
        state: u64,
    ) -> Result<(), u64> {
        if !self.parent.valid() {
            return Err(abi::DENIED);
        }
        if !(1..=keys::LAST_KEY as u64).contains(&key)
            || key == keys::IDLE as u64
            || ![
                keys::PRESS as u64,
                keys::RELEASE as u64,
                keys::REPEAT as u64,
            ]
            .contains(&state)
        {
            return Err(abi::INVALID_ARGUMENT);
        }
        self.reap_input();
        let index = self.input.admit(channel)?;
        let route = &mut self.input.streams[index];
        let bit = 1 << (key - 1);
        if state != keys::PRESS as u64 && route.held & bit == 0 {
            return Ok(());
        }
        if route.suppressed & bit == 0
            && let Some(slot) = self.input.groups[route.group].focus
            && self.children[slot].grant.keys & bit != 0
        {
            self.children[slot].can_push(index, route.group, self.input.groups[route.group].id)?;
            route.description_locked = true;
        }
        if state == keys::RELEASE as u64 {
            route.held &= !bit;
            if route.suppressed & bit != 0 {
                route.suppressed &= !bit;
                return Ok(());
            }
        } else {
            route.held |= bit;
            if route.suppressed & bit != 0 {
                return Ok(());
            }
        }
        if let Some(slot) = self.input.groups[route.group].focus {
            let child = &mut self.children[slot];
            if child.grant.keys & bit != 0
                && !child.push(
                    index,
                    channel,
                    route.group,
                    self.input.groups[route.group].id,
                    route.description,
                    [wire::KEY, key, state, 0],
                )?
            {
                self.cancel_stream_index(index);
            }
        }
        Ok(())
    }
}
