//! Retain read authority for replay, but never map pixels in a matrix process.
use super::*;
use cathedral_contracts::{composition, display};
use cathedral_user_runtime::memory::Object;
impl Parent {
    pub(in super::super) fn image(
        &mut self,
        words: wire::Packet,
        object: Object,
    ) -> Result<wire::Packet, Error> {
        let slot = self
            .images
            .iter()
            .position(|i| i.as_ref().is_some_and(|(key, _)| *key == words[2]))
            .or_else(|| self.images.iter().position(Option::is_none))
            .ok_or(Error(abi::NO_MEMORY as i64))?;
        let reply = self.image_call(words, &object)?;
        if reply[0] == 0 {
            self.images[slot] = Some((words[2], object));
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
            let (send, receive) = self.pair.unwrap();
            send.send_memory(&wire::encode(words), object)?;
            let len = receive.receive_until(&mut bytes, time::after(100)?)?;
            return wire::decode(&bytes[..len]).ok_or(Error(abi::IO_ERROR as i64));
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
        let len = receive.receive_until(&mut bytes, time::after(100)?)?;
        let reply = display::decode(&bytes[..len]).ok_or(Error(abi::IO_ERROR as i64))?;
        Ok(wire::error(reply[0]))
    }
}
