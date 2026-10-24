//! Propagate the effective local preference; no global pointer identity crosses.
use super::*;
impl Output {
    pub fn cursors(&mut self, world: &World) -> Result<(), Error> {
        if !self.grant.valid() {
            return Ok(());
        }
        for index in 0..wire::INPUT_STREAMS {
            if let Some(c) = world.cursor(index).filter(|c| c.inherited) {
                self.cursors.forward(
                    index,
                    self.pair,
                    [self.grant.epoch, c.cycle, c.channel as u64, c.mode],
                )?;
            } else {
                self.cursors.forget(index);
            }
        }
        Ok(())
    }
}
