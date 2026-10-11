//! Upstream matrices receive clipped commands; only the root speaks compositor IPC.
use super::*;
use cathedral_contracts::{composition, display};
impl Parent {
    pub(in super::super) fn surface(&self, words: wire::Packet) -> Result<wire::Packet, Error> {
        if self.upstream {
            return self.call(words);
        }
        if self.display.is_none() {
            return Ok(wire::error(abi::DENIED));
        }
        let [op, _, key, x, y, w, h, color] = words;
        if !composition::MATRIX_KEYS.contains(&key) {
            return Ok(wire::error(abi::NO_MEMORY));
        }
        if op == wire::ERASE {
            let bits = if key < 4 { 2 } else { 4 };
            for index in composition::MATRIX_KEYS {
                if index & ((1 << bits) - 1) == key {
                    self.display_call([composition::MATRIX_RECT | (index << 16), 0, 0, 0, 0, 0])?;
                }
            }
        } else {
            let reply =
                self.display_call([composition::MATRIX_RECT | (key << 16), x, y, w, h, color])?;
            return Ok(wire::error(reply[0]));
        }
        Ok([0; 8])
    }
    pub(in super::super) fn erase(&self, child: usize) -> Result<(), Error> {
        if self.display.is_some() || self.upstream {
            self.surface([
                wire::ERASE,
                self.grant.epoch,
                child as u64 + 1,
                0,
                0,
                0,
                0,
                0,
            ])?;
        }
        Ok(())
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
    fn display_call(&self, words: [u64; 6]) -> Result<[u64; 6], Error> {
        let (send, recv) = self.display.unwrap();
        send.send(&display::encode(words))?;
        let mut bytes = [0; 64];
        let len = recv.receive_until(&mut bytes, time::after(100)?)?;
        display::decode(&bytes[..len]).ok_or(Error(abi::IO_ERROR as i64))
    }
}
