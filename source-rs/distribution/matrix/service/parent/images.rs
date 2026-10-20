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
        let mut bytes = [0; 64];
        if self.upstream {
            return cathedral_user_runtime::matrix::upstream::exchange(
                self.pair.unwrap(),
                words,
                Some(object),
            );
        }
        let (send, receive) = self.display.ok_or(Error(abi::DENIED as i64))?;
        let [_, _, key, x, y, w, h, stride] = words;
        if key == 0 || key > composition::MATRIX_TILES as u64 {
            return Ok(wire::error(abi::BAD_HANDLE));
        }
        send.send_memory(
            &display::encode([
                composition::MATRIX_IMAGE | ((key - 1) << 16),
                x,
                y,
                w,
                h,
                stride,
            ]),
            object,
        )?;
        let len =
            cathedral_user_runtime::matrix::upstream::receive(receive, &mut bytes, words[0], 100)?;
        let reply = display::decode(&bytes[..len]).ok_or(Error(abi::IO_ERROR as i64))?;
        Ok(wire::error(reply[0]))
    }
}
