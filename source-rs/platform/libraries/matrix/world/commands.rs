//! Local protocol dispatch after the program authenticates its owner/input endpoint.
//! These operations are never accepted by the child-facing `request` dispatcher.
use super::*;

impl<const N: usize> World<N> {
    /// Placement and revocation within already supplied authority. Binding,
    /// stacking allocation and launch/recovery policy remain the owner's choices.
    pub fn owner_command(&mut self, words: wire::Packet) -> wire::Packet {
        if (wire::INPUT_GROUP_CREATE..=wire::INPUT_SELECT).contains(&words[0]) {
            return self.input_owner_command(words);
        }
        let result = match words {
            [
                wire::INPUT_PRODUCER_GRANT,
                slot,
                count,
                rights,
                channels,
                groups,
                0,
                0,
            ] if slot < N as u64
                && count <= wire::INPUT_STREAMS as u64
                && channels <= wire::INPUT_STREAMS as u64
                && groups <= wire::INPUT_GROUPS as u64 =>
            {
                self.grant_input_producer(
                    slot as usize,
                    count as usize,
                    rights,
                    channels as usize,
                    groups as usize,
                )
            }
            [wire::INPUT_PRODUCER_REVOKE, slot, 0, 0, 0, 0, 0, 0] if slot < N as u64 => {
                self.revoke_input_producer(slot as usize);
                Ok(())
            }
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
        let result = match words {
            [wire::POINTER_SOURCE, x, y, buttons @ 0..=7, 0, 0, 0, 0] => {
                self.source_sample(wire::SINGLE_INPUT_STREAM, [wire::POINTER, x, y, buttons])
            }
            [wire::KEY_SOURCE, key, state, 0, 0, 0, 0, 0] => {
                self.source_sample(wire::SINGLE_INPUT_STREAM, [wire::KEY, key, state, 0])
            }
            _ => return wire::error(abi::DENIED),
        };
        wire::error(result.err().unwrap_or(0))
    }
}
