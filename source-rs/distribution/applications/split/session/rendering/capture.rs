//! Pin the accepted local scene during this dispatch; map only sealed readers.
//! Masked destination remains private until the capture provider freezes it.
use super::*;
use cathedral_matrix::capture::{Layer, Paint, Plan, Rect, Source};
impl Output {
    pub fn capture(
        &mut self,
        world: &World,
        source: Source,
        plan: &Plan,
        target: &mut Private,
    ) -> Result<(), Error> {
        let mut draws = [None; 4];
        for (rank, draw) in draws.iter_mut().enumerate() {
            *draw = world
                .capture_surface(
                    source.participant.slot,
                    source.participant.incarnation,
                    source.epoch,
                    rank,
                )
                .map_err(|c| Error(c as i64))?;
        }
        let result = (|| {
            for draw in draws.iter().flatten() {
                if draw[0] == wire::SURFACE_IMAGE {
                    self.images.map(draw[2])?;
                }
            }
            let mut layers = [Layer {
                rect: Rect {
                    x: 0,
                    y: 0,
                    width: 1,
                    height: 1,
                },
                paint: Paint::Solid(0),
            }; 4];
            let mut count = 0;
            for draw in draws.iter().flatten() {
                let paint = match draw[0] {
                    wire::PAINT => Paint::Solid(draw[7] as u32),
                    wire::SURFACE_CURSOR => Paint::Cursor(draw[7] as u32),
                    wire::SURFACE_IMAGE => Paint::Image {
                        bytes: self
                            .images
                            .get(draw[2])?
                            .bytes()
                            .ok_or(Error(abi::IO_ERROR as i64))?,
                        stride: draw[7],
                    },
                    _ => return Err(Error(abi::DENIED as i64)),
                };
                layers[count] = Layer {
                    rect: Rect {
                        x: draw[3],
                        y: draw[4],
                        width: draw[5],
                        height: draw[6],
                    },
                    paint,
                };
                count += 1;
            }
            plan.render(&layers[..count], target.bytes_mut())
                .map_err(|_| Error(abi::INVALID_ARGUMENT as i64))
        })();
        // Unmap on every path, including partial mapping failure. Read authority
        // remains in the normal scene ledger; no unmasked attachment leaves here.
        for draw in draws.iter().flatten() {
            if draw[0] == wire::SURFACE_IMAGE {
                self.images.unmap(draw[2])?;
            }
        }
        result
    }
}
