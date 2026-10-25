//! Encode one prepared visual command for the enclosing matrix or root display.
use super::*;
use cathedral_platform_contracts::display::{self, composition};
use cathedral_user_runtime::memory::Object;
impl Parent {
    pub(super) fn output_receiver(&self) -> Result<Handle, Error> {
        if self.upstream {
            return self.pair.map(|p| p.1).ok_or(Error(abi::DENIED as i64));
        }
        self.display
            .as_ref()
            .ok_or(Error(abi::DENIED as i64))?
            .receiver()
    }
    pub(in crate::service) fn display_wake(
        &self,
    ) -> Result<Option<cathedral_user_runtime::event::Interest>, Error> {
        self.display.as_ref().map_or(Ok(None), |d| d.wake())
    }
    pub(in crate::service) fn display_deadline(&self) -> Option<u64> {
        self.display.as_ref().and_then(|d| d.deadline())
    }
    pub(super) fn output_status(&self) -> Result<Option<u64>, Error> {
        self.display
            .as_ref()
            .ok_or(Error(abi::DENIED as i64))?
            .poll()
            .map(|r| r.map(|w| w[0]))
    }
    pub(super) fn send_output(
        &self,
        words: wire::Packet,
        object: Option<&Object>,
    ) -> Result<(), Error> {
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
        if self.upstream {
            let send = self.pair.ok_or(Error(abi::DENIED as i64))?.0;
            match object {
                Some(object) => send.send_memory(bytes, object),
                None => send.send(bytes),
            }
        } else {
            self.display
                .as_ref()
                .ok_or(Error(abi::DENIED as i64))?
                .begin(bytes, object)
        }
    }
}
