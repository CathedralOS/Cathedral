//! Retain read authority for replay, but never map pixels in a matrix process.
use super::*;
use cathedral_platform_contracts::display;
use cathedral_platform_contracts::display::composition;
use cathedral_user_runtime::memory::Object;
impl Parent {
    pub(in super::super) fn image(
        &mut self,
        words: wire::Packet,
        object: Object,
    ) -> Result<wire::Packet, Error> {
        self.images.room(words[2])?;
        let reply = self.image_call(words, &object)?;
        if reply[0] == 0 {
            self.images.commit(words[2], Some(object));
        }
        Ok(reply)
    }
    pub(super) fn image_call(
        &self,
        words: wire::Packet,
        object: &Object,
    ) -> Result<wire::Packet, Error> {
        if self.upstream {
            return cathedral_user_runtime::matrix::upstream::exchange(
                self.pair.unwrap(),
                words,
                Some(object),
            );
        }
        let display = self.display.as_ref().ok_or(Error(abi::DENIED as i64))?;
        let [_, _, key, x, y, w, h, stride] = words;
        if key == 0 || key > composition::MATRIX_TILES as u64 {
            return Ok(wire::error(abi::BAD_HANDLE));
        }
        let reply = display.call(
            &display::encode([
                composition::MATRIX_IMAGE | ((key - 1) << 16),
                x,
                y,
                w,
                h,
                stride,
            ]),
            Some(object),
        )?;
        Ok(wire::error(reply[0]))
    }
}
