//! Tile and Settings selection from local pointer/keyboard input.
use super::*;
pub struct Selection {
    pub index: usize,
}
impl Selection {
    pub fn new() -> Self {
        Self { index: 1 }
    }
    /// Return repaint/activate; hovering alone does not repaint retained tiles.
    #[allow(clippy::too_many_arguments)]
    pub fn event(
        &mut self,
        event: wire::Packet,
        extent: wire::Packet,
        clicked: bool,
        count: usize,
        picker: bool,
        managed: bool,
        retry_saves: bool,
    ) -> (bool, bool) {
        let (selected, changed, activate) = policy::selection(
            self.index,
            event,
            extent,
            clicked,
            count,
            picker,
            managed,
            retry_saves,
        );
        self.index = selected;
        (changed, activate)
    }
}
