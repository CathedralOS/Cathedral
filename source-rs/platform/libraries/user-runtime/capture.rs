//! Capture recipient. Receiver: the matrix's separately supplied capture endpoint.
//! Results are sealed and already masked; no unmasked backing is accepted here.
use crate::{Error, abi, connection::Client, memory::Object, services::Pair, time};
use cathedral_platform_contracts::{capture as wire, matrix};
pub struct Capture(Client);
pub struct Image {
    pub width: u64,
    pub height: u64,
    pub stride: u64,
    pub object: Object,
}
impl Capture {
    /// Takes custody of the separately granted capture connection.
    pub fn connect(pair: Pair) -> Result<Self, Error> {
        Ok(Self(Client::connect(pair, time::after(300)?)?))
    }
    pub fn request(&mut self, words: matrix::Packet) -> Result<matrix::Packet, Error> {
        let (bytes, len) = self.0.call_retained(&matrix::encode(words))?;
        matrix::decode(&bytes[..len]).ok_or_else(|| {
            self.0.retire();
            Error(abi::IO_ERROR as i64)
        })
    }
    pub fn begin(&mut self) -> Result<u64, Error> {
        let reply = self.request([wire::BEGIN, 0, 0, 0, 0, 0, 0, 0])?;
        if reply[0] != 0 {
            return Err(Error(reply[0] as i64));
        }
        if reply[1] == 0 || reply[2..] != [0; 6] {
            return Err(Error(abi::IO_ERROR as i64));
        }
        Ok(reply[1])
    }
    pub fn take(&mut self, ticket: u64) -> Result<Option<Image>, Error> {
        let (bytes, len, object) = self.0.call_retained_memory(
            &matrix::encode([wire::TAKE, ticket, 0, 0, 0, 0, 0, 0]),
            None,
        )?;
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
            _ => {
                self.0.retire();
                Err(Error(abi::IO_ERROR as i64))
            }
        }
    }
}
