//! Shared keyboard selection; pointer capture stays with each participating stream.
use super::*;

#[derive(Clone, Copy)]
pub(super) struct Group {
    pub id: u32,
    pub focus: Option<usize>,
}
impl Group {
    pub const fn empty() -> Self {
        Self { id: 0, focus: None }
    }
}

impl<const N: usize> World<N> {
    /// The calling service authenticates control authority, including allowed
    /// targets. This operation is not exposed by the child request dispatcher.
    pub fn select_input_group(&mut self, id: u32, child: Option<usize>) -> Result<(), u64> {
        if !self.parent.valid()
            || child.is_some_and(|s| !self.children.get(s).is_some_and(|c| c.grant.valid()))
        {
            return Err(abi::DENIED);
        }
        let group = self
            .input
            .groups
            .iter()
            .position(|g| g.id != 0 && g.id == id)
            .ok_or(abi::DENIED)?;
        self.select_input_group_index(group, child);
        Ok(())
    }
    pub(super) fn select_input_group_index(&mut self, group: usize, child: Option<usize>) {
        let selection = &mut self.input.groups[group];
        if selection.focus == child {
            return;
        }
        for (index, stream) in self.input.streams.iter_mut().enumerate() {
            if stream.id == 0 || stream.group != group {
                continue;
            }
            if let Some(old) = selection.focus {
                self.children[old].cancel_keys(index, stream.id);
            }
            stream.suppressed |= stream.held;
        }
        selection.focus = child;
    }
}
