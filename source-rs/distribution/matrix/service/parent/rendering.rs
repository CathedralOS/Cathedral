//! Upstream matrices receive clipped commands; only the root speaks compositor IPC.
use super::*;
use cathedral_platform_contracts::display;
use cathedral_platform_contracts::display::composition;
impl Parent {
    pub(in super::super) fn surface(&mut self, words: wire::Packet) -> Result<wire::Packet, Error> {
        if words[0] == wire::SURFACE_IMAGE {
            let object = self.images.get(words[2])?;
            return self.image_call(words, object);
        }
        let reply = self.rectangle(words)?;
        if reply[0] == 0
            || (words[0] == wire::ERASE && matches!(reply[0], abi::DENIED | abi::BAD_HANDLE))
        {
            self.images.commit(words[2], None);
        }
        Ok(reply)
    }
    pub(super) fn rectangle(&self, words: wire::Packet) -> Result<wire::Packet, Error> {
        if self.upstream {
            return self.call(words);
        }
        if self.display.is_none() {
            return Ok(wire::error(abi::DENIED));
        }
        let [op, _, key, x, y, w, h, color] = words;
        if key == 0 || key > composition::MATRIX_TILES as u64 {
            return Ok(wire::error(abi::BAD_HANDLE));
        }
        let reply = if op == wire::ERASE {
            self.display_call([composition::MATRIX_RECT | ((key - 1) << 16), 0, 0, 0, 0, 0])?
        } else {
            self.display_call([
                (if op == wire::SURFACE_CURSOR {
                    composition::MATRIX_CURSOR
                } else {
                    composition::MATRIX_RECT
                }) | ((key - 1) << 16),
                x,
                y,
                w,
                h,
                color,
            ])?
        };
        Ok(wire::error(reply[0]))
    }
    pub(in super::super) fn create_surface(&self, rank: u64) -> Result<wire::Packet, Error> {
        if self.upstream {
            return self.call([wire::SURFACE_CREATE, self.grant.epoch, rank, 0, 0, 0, 0, 0]);
        }
        if self.display.is_none() || rank >= composition::MATRIX_TILES as u64 {
            return Ok(wire::error(abi::NO_MEMORY));
        }
        // Root slots are private implementation details; children receive fresh
        // matrix-local names, never these tile indices.
        Ok([0, rank + 1, 0, 0, 0, 0, 0, 0])
    }
    pub(in super::super) fn pointer(&self, x: u64, y: u64) -> Result<(), Error> {
        if self.display.is_some() {
            self.display_call([composition::MATRIX_POINTER, x, y, 0, 0, 0])?;
        }
        Ok(())
    }
    pub(in super::super) fn pixel(&self, x: u64, y: u64) -> Result<wire::Packet, Error> {
        if self.display.is_none() {
            return Ok(wire::error(abi::DENIED));
        }
        let reply = self.display_call([composition::PIXEL, x, y, 0, 0, 0])?;
        Ok([reply[0], reply[1], 0, 0, 0, 0, 0, 0])
    }
    pub(super) fn display_call(&self, words: [u64; 6]) -> Result<[u64; 6], Error> {
        let (send, recv) = self.display.unwrap();
        send.send(&display::encode(words))?;
        let mut bytes = [0; 64];
        let len =
            cathedral_user_runtime::matrix::upstream::receive(recv, &mut bytes, words[0], 100)?;
        display::decode(&bytes[..len]).ok_or(Error(abi::IO_ERROR as i64))
    }
}
