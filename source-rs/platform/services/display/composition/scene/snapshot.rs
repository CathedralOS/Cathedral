//! Validate a bounded snapshot in scratch storage before the client publishes it.
use super::{Draw, Rect, Scene};
use cathedral_contracts::{composition as wire, user as abi};

impl Scene {
    pub fn decode(&mut self, bytes: &[u8]) -> Result<(), u64> {
        if bytes.len() < wire::SNAPSHOT_BYTES {
            return Err(abi::INVALID_ARGUMENT);
        }
        let word = |at| u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap());
        if word(0) != wire::MAGIC || word(8) > wire::MAX_NODES as u64 {
            return Err(abi::INVALID_ARGUMENT);
        }
        let scene = self;
        scene.count = word(8) as usize;
        for index in 0..scene.count {
            let base = 16 + index * 56;
            let [parent, kind, x, y, w, h, value] = core::array::from_fn(|i| word(base + i * 8));
            let (ox, oy, clip) = if parent == wire::ROOT {
                (
                    0,
                    0,
                    Rect {
                        x: 0,
                        y: 0,
                        w: 256,
                        h: 256,
                    },
                )
            } else {
                let ancestor = scene
                    .draws
                    .get(parent as usize)
                    .filter(|_| parent < index as u64)
                    .ok_or(abi::INVALID_ARGUMENT)?;
                if ancestor.kind != wire::GROUP {
                    return Err(abi::INVALID_ARGUMENT);
                }
                (ancestor.rect.x, ancestor.rect.y, ancestor.clip)
            };
            let rect = Rect {
                x: ox.checked_add(x).ok_or(abi::INVALID_ARGUMENT)?,
                y: oy.checked_add(y).ok_or(abi::INVALID_ARGUMENT)?,
                w,
                h,
            };
            if !rect.valid() {
                return Err(abi::INVALID_ARGUMENT);
            }
            match kind {
                wire::GROUP | wire::INPUT if value == 0 => (),
                wire::RECT if value <= 0xffffff => (),
                wire::TEXT => {
                    let text = value.to_le_bytes();
                    let len = text.iter().position(|&b| b == 0).unwrap_or(8);
                    if len == 0
                        || text[..len].iter().any(|b| !(32..=126).contains(b))
                        || text[len..].iter().any(|&b| b != 0)
                        || !(7..=28).contains(&h)
                        || h % 7 != 0
                        || w != len as u64 * 6 * (h / 7)
                    {
                        return Err(abi::INVALID_ARGUMENT);
                    }
                }
                wire::IMAGE if w == 16 && h == 16 && value == 0 => (),
                _ => return Err(abi::INVALID_ARGUMENT),
            }
            scene.draws[index] = Draw {
                kind,
                rect,
                clip: rect.intersect(clip),
                value,
            };
        }
        // Canonical unused nodes prevent hidden trailing commands.
        if bytes[16 + scene.count * 56..16 + wire::MAX_NODES * 56]
            .iter()
            .any(|&b| b != 0)
        {
            return Err(abi::INVALID_ARGUMENT);
        }
        for (pixel, chunk) in scene
            .pixels
            .iter_mut()
            .zip(bytes[16 + wire::MAX_NODES * 56..wire::SNAPSHOT_BYTES].chunks_exact(4))
        {
            *pixel = u32::from_le_bytes(chunk.try_into().unwrap());
            if *pixel > 0xffffff {
                return Err(abi::INVALID_ARGUMENT);
            }
        }
        Ok(())
    }
}
