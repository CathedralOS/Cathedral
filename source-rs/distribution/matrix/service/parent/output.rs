//! Encode one prepared visual command for the enclosing matrix or root display.
use super::*;
use cathedral_platform_contracts::display::{self, composition};
use cathedral_user_runtime::memory::Object;
impl Parent {
    pub(super) fn output(&self) -> Result<(Handle, Handle), Error> {
        (if self.upstream {
            self.pair
        } else {
            self.display
        })
        .ok_or(Error(abi::DENIED as i64))
    }
    pub(super) fn send_output(
        &self,
        words: wire::Packet,
        object: Option<&Object>,
    ) -> Result<(), Error> {
        let (send, _) = self.output()?;
        let mut bytes = wire::encode(words);
        if !self.upstream {
            let [op, _, key, x, y, w, h, value] = words;
            let values = if matches!(
                op,
                wire::PRESENT_BEGIN | wire::PRESENT_COMMIT | wire::PRESENT_ABORT
            ) {
                let command = match op {
                    wire::PRESENT_BEGIN => composition::MATRIX_BEGIN,
                    wire::PRESENT_COMMIT => composition::MATRIX_COMMIT,
                    _ => composition::MATRIX_ABORT,
                };
                [command, key, 0, 0, 0, 0]
            } else {
                if key == 0 || key > composition::MATRIX_TILES as u64 {
                    return Err(Error(abi::BAD_HANDLE as i64));
                }
                let command = if object.is_some() {
                    composition::MATRIX_IMAGE
                } else if op == wire::SURFACE_CURSOR {
                    composition::MATRIX_CURSOR
                } else {
                    composition::MATRIX_RECT
                };
                let mut values = [command | ((key - 1) << 16), x, y, w, h, value];
                if op == wire::ERASE {
                    values[1..].fill(0);
                }
                values
            };
            bytes[..48].copy_from_slice(&display::encode(values));
        }
        let bytes = &bytes[..if self.upstream { 64 } else { 48 }];
        match object {
            Some(object) => send.send_memory(bytes, object),
            None => send.send(bytes),
        }
    }
}
