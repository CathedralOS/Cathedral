//! Capture recipient. Receiver: the matrix's separately supplied capture endpoint.
//! Results are sealed and already masked; no unmasked backing is accepted here.
use crate::{Error, abi, memory::Object, services::Pair};
use cathedral_platform_contracts::{capture as wire, matrix};
pub struct Capture(pub Pair);
pub struct Image {
    pub width: u64,
    pub height: u64,
    pub stride: u64,
    pub object: Object,
}
impl Capture {
    pub fn begin(&self) -> Result<u64, Error> {
        let reply = crate::matrix::upstream::call(self.0, [wire::BEGIN, 0, 0, 0, 0, 0, 0, 0])?;
        if reply[0] != 0 {
            return Err(Error(reply[0] as i64));
        }
        if reply[1] == 0 || reply[2..] != [0; 6] {
            return Err(Error(abi::IO_ERROR as i64));
        }
        Ok(reply[1])
    }
    pub fn take(&self, ticket: u64) -> Result<Option<Image>, Error> {
        self.0
            .0
            .send(&matrix::encode([wire::TAKE, ticket, 0, 0, 0, 0, 0, 0]))?;
        let mut bytes = [0; 64];
        let (len, object) = self.0.1.receive_memory(&mut bytes)?;
        match (matrix::decode(&bytes[..len]), object) {
            (Some([0, t, w, h, stride, 0, 0, 0]), Some(object))
                if t == ticket
                    && w > 0
                    && h > 0
                    && w.checked_mul(h).is_some_and(|n| n <= 32_768)
                    && stride == w
                    && object.len() <= 131_072
                    && cathedral_platform_contracts::display::composition::valid_image(
                        w,
                        h,
                        stride,
                        object.len(),
                    ) =>
            {
                Ok(Some(Image {
                    width: w,
                    height: h,
                    stride,
                    object,
                }))
            }
            (Some([wire::PENDING, 0, 0, 0, 0, 0, 0, 0]), None) => Ok(None),
            (Some([code, 0, 0, 0, 0, 0, 0, 0]), None) if code != 0 => Err(Error(code as i64)),
            _ => Err(Error(abi::IO_ERROR as i64)),
        }
    }
}
