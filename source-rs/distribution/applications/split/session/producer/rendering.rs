//! The local producer's cursor obeys its own pointer target, independently of
//! inherited pointers. Preference changes can repaint without another motion.
use super::*;
impl Producer {
    pub(super) fn render_cursor(
        &mut self,
        output: &mut Output,
        world: &World,
    ) -> Result<(), Error> {
        let Some((ordinal, x, y)) = self.point else {
            return Ok(());
        };
        let Some((epoch, name)) = self.cursor else {
            return Ok(());
        };
        let hidden = world
            .producer_cursor(SLOT, ordinal)
            .is_none_or(|c| c.mode == wire::CURSOR_HIDDEN);
        let state = (x, y, hidden);
        if self.painted == Some(state) {
            return Ok(());
        }
        let (width, height) = if hidden { (0, 0) } else { (7, 12) };
        let reply = output.services.request_interaction(
            [
                wire::SURFACE_CURSOR,
                epoch,
                name,
                x,
                y,
                width,
                height,
                0xffd166,
            ],
            None,
        )?;
        if reply[0] != 0 {
            return Err(Error(reply[0] as i64));
        }
        self.painted = Some(state);
        Ok(())
    }
}
