//! Local protocol dispatch after the program authenticates its owner/input endpoint.
//! These operations are never accepted by the child-facing `request` dispatcher.
use super::*;

impl<const N: usize> World<N> {
    /// Placement and revocation within already supplied authority. Binding,
    /// stacking allocation and launch/recovery policy remain the owner's choices.
    pub fn owner_command(&mut self, words: wire::Packet) -> wire::Packet {
        let result = match words {
            [wire::PLACE, child, x, y, width, height, keys, 0] if child < N as u64 => self.place(
                child as usize,
                Rect {
                    x,
                    y,
                    width,
                    height,
                },
                keys,
            ),
            [wire::REVOKE, child, 0, 0, 0, 0, 0, 0] if child < N as u64 => {
                self.revoke(child as usize)
            }
            [wire::HEALTH, 0, 0, 0, 0, 0, 0, 0] => Ok(()),
            _ => Err(abi::DENIED),
        };
        wire::error(result.err().unwrap_or(0))
    }

    /// Input injected by a trusted upstream source, not necessarily hardware.
    /// Coordinates are clamped by this world's input rules; no child may call this.
    pub fn source_command(&mut self, words: wire::Packet) -> wire::Packet {
        match words {
            [wire::POINTER_SOURCE, x, y, buttons @ 0..=7, 0, 0, 0, 0] => {
                self.pointer(x, y, buttons)
            }
            [wire::KEY_SOURCE, key, state, 0, 0, 0, 0, 0] => self.key(key, state),
            _ => return wire::error(abi::DENIED),
        }
        [0; 8]
    }
}
