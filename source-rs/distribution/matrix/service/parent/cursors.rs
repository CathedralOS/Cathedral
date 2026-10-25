//! Compose descendant preferences on their inherited pointer aliases. Only the
//! root adapter translates its explicitly installed pointer into display control.
use super::*;
impl Parent {
    pub(in crate::service) fn cursors(&mut self, world: &World) -> Result<(), Error> {
        if self.upstream {
            let Some(link) = self.interaction.as_ref().filter(|_| self.grant.valid()) else {
                return Ok(());
            };
            for index in 0..wire::INPUT_STREAMS {
                if let Some(c) = world.cursor(index).filter(|c| c.inherited) {
                    self.cursors.forward_with(
                        index,
                        [self.grant.epoch, c.cycle, c.channel as u64, c.mode],
                        |words| link.call(words, None),
                    )?;
                } else {
                    self.cursors.forget(index);
                }
            }
        } else if self.display.is_some() {
            let mode = (0..wire::INPUT_STREAMS)
                .filter_map(|i| world.cursor(i))
                .find(|c| !c.inherited && c.channel == super::super::requests::source::POINTER)
                .map_or(wire::CURSOR_DEFAULT, |c| c.mode);
            if self.cursor_mode != Some(mode) {
                let reply = self.display_call([
                    cathedral_platform_contracts::display::composition::MATRIX_POINTER_STYLE,
                    mode,
                    0,
                    0,
                    0,
                    0,
                ])?;
                if reply[0] != 0 {
                    return Err(Error(reply[0] as i64));
                }
                self.cursor_mode = Some(mode);
            }
        }
        Ok(())
    }
}
