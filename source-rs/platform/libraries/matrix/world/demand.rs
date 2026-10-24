//! Render demand composes through owners independently of input or execution.
//! Current demand is display-only; future capture consumers must be counted
//! separately before removing demand. Hints do not enforce CPU use or publishing.
use super::*;
impl<const N: usize> World<N> {
    pub fn render_demand(&mut self, frames: bool) {
        if self.frames == frames {
            return;
        }
        self.frames = frames;
        for child in &mut self.children {
            child.demand(frames && child.visible);
        }
    }
    /// Optional owner policy, independent of hiding pixels. Retains the child's
    /// view and input grant but cancels current holds and forgets local routes.
    pub fn deactivate_input(&mut self, slot: usize) -> Result<(), u64> {
        let child = self.children.get_mut(slot).ok_or(abi::DENIED)?;
        child.cancel();
        self.input.forget(slot);
        Ok(())
    }
}
